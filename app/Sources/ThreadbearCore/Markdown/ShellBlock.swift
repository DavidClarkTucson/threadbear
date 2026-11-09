import Foundation

/// A ```sh, ```bash, ```zsh or ```shell block in a reply: a command the agent
/// is handing over, often one only Mike can run. The Mac app draws a play
/// icon on it that types the command into the thread's Terminal, and stops
/// short of Return, so a person always reads what's about to run (Yuriy on
/// X, 10 Nov: copying "the command only you can run on prod" by hand).
///
/// What's in a reply can come from anything the agent read, so this only
/// takes text that means what it looks like. A control character could end
/// a bracketed paste early (ESC [ 201 ~) and let the next line run by
/// itself, and invisible or bidi characters could show a different command
/// from the one typed. A block holding any of those gets no play icon.
public struct ShellBlock: Hashable, Sendable {
    public let command: String

    public init(command: String) {
        self.command = command
    }

    static let languages: Set<String> = ["sh", "bash", "zsh", "shell"]

    /// Past this it's a script, not a command to hand someone.
    public static let maxSource = 8_000

    /// The command for a fence's info string (only the first word counts),
    /// or nil for any other fence, or one that can't safely be typed.
    public static func from(language: String?, code: String) -> ShellBlock? {
        guard let word = language?.split(whereSeparator: { $0 == " " || $0 == "\t" || $0 == "{" }).first?.lowercased(),
              languages.contains(word) else { return nil }
        var source = code.replacingOccurrences(of: "\r\n", with: "\n")
        while source.hasSuffix("\n") { source.removeLast() }
        guard !source.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty, source.count <= maxSource, isPlain(source) else { return nil }
        return ShellBlock(command: withoutPrompts(source))
    }

    /// Whether a block of a reply's text could hold one, so only those are
    /// checked for a closed fence.
    public static func mayHold(_ text: String) -> Bool {
        guard text.contains("```") || text.contains("~~~") else { return false }
        let lowered = text.lowercased()
        return lowered.contains("sh") // sh, bash, zsh and shell all have it.
    }

    /// Whether it has to go in as a paste: in a shell that can't take one,
    /// a newline would run what's before it and a tab would complete it.
    public var needsPaste: Bool {
        command.contains("\n") || command.contains("\t")
    }

    /// Only ordinary visible text, spaces, tabs and newlines.
    private static func isPlain(_ text: String) -> Bool {
        text.unicodeScalars.allSatisfy { scalar in
            if scalar == "\n" || scalar == "\t" { return true }
            switch scalar.properties.generalCategory {
            case .control, .format, .lineSeparator, .paragraphSeparator, .privateUse, .surrogate, .unassigned: return false
            default: return true
            }
        }
    }

    /// Agents sometimes write `$ npm test`. The "$ " only goes when every
    /// line has one, so a line that really starts with `$` is left alone.
    private static func withoutPrompts(_ text: String) -> String {
        let lines = text.split(separator: "\n", omittingEmptySubsequences: false)
        let written = lines.filter { !$0.allSatisfy(\.isWhitespace) }
        guard written.allSatisfy({ $0.hasPrefix("$ ") }) else { return text }
        return lines.map { $0.hasPrefix("$ ") ? String($0.dropFirst(2)) : String($0) }.joined(separator: "\n")
    }
}

/// What putting a command in a shell should do now, worked out from what
/// the terminal says about itself, each time it's asked.
public enum TerminalPut {
    public enum Step: Equatable, Sendable {
        /// The shell is still starting; ask again in a moment.
        case wait
        case type
        /// The last command put here is still at the prompt, unrun, and
        /// this is a different one: it goes in place of that one, as run
        /// together they'd make a third command nobody wrote.
        case replaceLast
        /// It's already sitting at the prompt (a second click): only the
        /// keyboard needs to go there.
        case alreadyThere
        /// Something else has the terminal, or it can't take this safely.
        case busy
    }

    /// - shellInFront: the shell itself owns the terminal, not a program
    ///   it's running (vim, a dev server, a command still going).
    /// - bracketedPaste: the terminal is in bracketed paste mode, which zsh,
    ///   fish and modern bash switch on only while they wait at a prompt.
    /// - starting: the shell has only just been started.
    /// - screen: what the terminal shows, read only if it's needed.
    /// - lastPut: the command this last put in this shell, if any.
    public static func step(for block: ShellBlock, shellInFront: Bool, bracketedPaste: Bool, starting: Bool, lastPut: String? = nil, screen: () -> String) -> Step {
        guard shellInFront else { return starting ? .wait : .busy }
        if !bracketedPaste {
            if starting { return .wait }
            // A shell that never does bracketed paste, like macOS's bash 3.
            if block.needsPaste { return .busy }
        }
        let shown = screen()
        if endsWith(shown, block.command) { return .alreadyThere }
        if let lastPut, endsWith(shown, lastPut) { return .replaceLast }
        return .type
    }

    /// Compared without any whitespace: the terminal wraps a long command
    /// and pads its lines, and a shell draws a pasted newline its own way.
    public static func endsWith(_ screen: String, _ command: String) -> Bool {
        let wanted = command.filter { !$0.isWhitespace }
        return !wanted.isEmpty && screen.filter { !$0.isWhitespace }.hasSuffix(wanted)
    }
}
