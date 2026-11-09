import AppKit
import SwiftUI
import SwiftTerm
import ThreadbearCore

/// The Terminal panel (⌘J, View > Show Terminal): a real terminal in the
/// dock, as VS Code and T3 Code have under their editor. SwiftTerm draws it
/// and runs your login shell in a pty, so colours, the cursor keys and
/// full-screen programs (vim, htop, claude) work as in Terminal.
///
/// Each thread has a shell of its own, started in the thread's folder (its
/// worktree when it has one) and kept running while you look at other
/// threads, so switching back finds it as you left it. A thread on another
/// Mac or the cloud gets a shell on this Mac in your home folder, with a
/// line saying so, as its folder isn't here. With no thread open it's a
/// shell in your home folder too.
///
/// A shell only starts when you ask for one: ⌘J, clicking the empty
/// panel, or a ```bash block's play icon. Clicking through threads with the panel open, or launching with
/// it left open, starts nothing, as each login shell reads your whole
/// profile. A thread's shell stops when the thread is archived or deleted
/// (a dev server left running there would otherwise keep going in a
/// worktree the next thread gets), and every shell stops when the app quits.
@MainActor
@Observable
final class Terminals {
    static let shared = Terminals()

    @ObservationIgnored private var sessions: [String: TerminalSession] = [:]
    /// Bumped to ask the showing terminal to take the keyboard.
    private(set) var focusTick = 0

    /// Bumped when a shell starts or stops, so the panel redraws (the
    /// sessions themselves aren't observed).
    private(set) var generation = 0

    /// The target's shell, if one has been started.
    func existing(for target: TerminalTarget) -> TerminalSession? {
        _ = generation
        return sessions[target.key]
    }

    /// The target's shell, starting one if it has none, and gives it the
    /// keyboard: ⌘J and a click on the empty panel.
    @discardableResult
    func open(_ target: TerminalTarget) -> TerminalSession {
        defer { focusTick += 1 }
        if let s = sessions[target.key] { return s }
        let s = TerminalSession(target: target)
        sessions[target.key] = s
        generation += 1
        return s
    }

    /// Stops a thread's shell (the thread was archived or deleted).
    func stop(threadId: String) {
        guard let s = sessions.removeValue(forKey: threadId) else { return }
        s.stop()
        generation += 1
    }

    /// Called whenever the app's thread list changes: stops the shells of
    /// threads that have been archived. Threads missing from the list are
    /// left alone here, as the list can be briefly incomplete; a deleted
    /// thread's shell is stopped from `forget`, or from `stopAll(except:)`
    /// once the runner's full list comes back after a reconnect.
    func stopArchived(in threads: [String: ThreadSummary]) {
        for key in sessions.keys where threads[key]?.isArchived == true { stop(threadId: key) }
    }

    /// The runner's full thread list has come back (on connecting): any
    /// thread's shell not in it belongs to a thread deleted while the app
    /// wasn't listening, so it stops. The shared home shell stays.
    func stopAll(except threadIds: some Collection<String>) {
        let keep = Set(threadIds)
        for key in sessions.keys where key != "home" && !keep.contains(key) { stop(threadId: key) }
    }

    enum PutOutcome { case typed, alreadyThere, busy }

    /// Threads with a command on its way, so a second click while the shell
    /// is still starting doesn't queue another.
    @ObservationIgnored private var putting: Set<String> = []

    /// Types `block` at the target's prompt, starting a shell if it needs
    /// one, and gives it the keyboard. Never presses Return: the command
    /// came out of a reply, so a person gets to read it first.
    func put(_ block: ShellBlock, in target: TerminalTarget, done: @escaping @MainActor (PutOutcome) -> Void) {
        guard putting.insert(target.key).inserted else { return }
        if sessions[target.key]?.exited == true { restart(target.key) }
        let session = open(target)
        Task { @MainActor [weak self] in
            defer { self?.putting.remove(target.key) }
            while true {
                switch session.putStep(for: block) {
                case .wait:
                    try? await Task.sleep(for: .milliseconds(50))
                case .type:
                    session.put(block.command)
                    return done(.typed)
                case .replaceLast:
                    session.clearLastPut()
                    session.put(block.command)
                    return done(.typed)
                case .alreadyThere:
                    return done(.alreadyThere)
                case .busy:
                    return done(.busy)
                }
            }
        }
    }

    /// A new shell in place of one that has exited.
    func restart(_ key: String) {
        guard let old = sessions[key] else { return }
        let fresh = TerminalSession(target: old.target)
        sessions[key] = fresh
        old.stop()
        generation += 1
        focusTick += 1
    }

    /// For the automation and `dump`: what's running where.
    var debugSummary: [String] {
        sessions.values.sorted { $0.target.key < $1.target.key }.map { "\($0.target.key) \($0.exited ? "exited" : "running") in \($0.target.cwd)" }
    }
}

/// Where a terminal runs: its key (the thread, or "home") and folder.
struct TerminalTarget: Equatable {
    var key: String
    var cwd: String
    /// Said above the terminal when its folder isn't the thread's.
    var note: String?

    @MainActor
    static func `for`(_ thread: ThreadSummary?, runnerName: (ThreadSummary) -> String?) -> TerminalTarget {
        let home = NSHomeDirectory()
        guard let thread else { return TerminalTarget(key: "home", cwd: home) }
        // An archived thread's worktree may have been handed to a newer
        // thread since (archive keeps it as a spare), so its folder could be
        // someone else's checkout now. Its own shell was stopped on archive.
        if thread.isArchived {
            return TerminalTarget(key: "home", cwd: home, note: "This thread is archived, so this terminal is in your home folder.")
        }
        if thread.isRemote {
            let where_ = runnerName(thread).map { "on \($0)" } ?? "on another machine"
            return TerminalTarget(key: "home", cwd: home, note: "This thread runs \(where_), so this terminal is on this Mac, in your home folder.")
        }
        var isDir: ObjCBool = false
        guard !thread.cwd.isEmpty, FileManager.default.fileExists(atPath: thread.cwd, isDirectory: &isDir), isDir.boolValue else {
            return TerminalTarget(key: "home", cwd: home, note: "This thread's folder isn't on this Mac any more, so this terminal is in your home folder.")
        }
        return TerminalTarget(key: thread.id, cwd: thread.cwd)
    }
}

/// One shell in one pty, and the view that draws it.
@MainActor
@Observable
final class TerminalSession: NSObject, LocalProcessTerminalViewDelegate {
    let target: TerminalTarget
    @ObservationIgnored let view: LocalProcessTerminalView
    private(set) var exited = false
    private(set) var title = ""
    @ObservationIgnored let startedAt = Date()
    /// How long a new shell may take to reach its first prompt. A slower
    /// one is treated as having no bracketed paste: one line still goes
    /// in, and the tty holds it until the shell reads it.
    static let startingFor: TimeInterval = 6
    /// What a play icon last put at this shell's prompt.
    @ObservationIgnored private var lastPut: String?
    /// A command is waiting at the prompt for Return, so the panel says so.
    private(set) var waitingForReturn = false
    @ObservationIgnored private var returnWatch: Task<Void, Never>?

    init(target: TerminalTarget) {
        self.target = target
        view = LocalProcessTerminalView(frame: NSRect(x: 0, y: 0, width: 600, height: 240))
        super.init()
        view.processDelegate = self
        view.font = TerminalStyle.font
        TerminalStyle.apply(to: view)
        start()
    }

    private func start() {
        let shell = ProcessInfo.processInfo.environment["SHELL"].flatMap { $0.isEmpty ? nil : $0 } ?? "/bin/zsh"
        // A login shell ("-zsh" as its name), so it reads your profile and
        // has the PATH Terminal gives you, which an app's own doesn't.
        let name = "-" + (shell as NSString).lastPathComponent
        view.startProcess(executable: shell, args: [], environment: Self.environment(), execName: name, currentDirectory: target.cwd)
    }

    static func environment(from base: [String: String] = ProcessInfo.processInfo.environment) -> [String] {
        var env = base
        env["TERM"] = "xterm-256color"
        env["COLORTERM"] = "truecolor"
        env["TERM_PROGRAM"] = "Threadbear"
        if (env["LANG"] ?? "").isEmpty { env["LANG"] = "en_US.UTF-8" }
        // Never a thread id: the runner reads it as "this is that thread's
        // own agent" (runner/src/cli.ts), so `tb` typed here would refuse to
        // archive the thread with a message meant for agents, and log what
        // you do as the agent's doing. The app may have inherited either
        // name if it was started from inside a thread.
        env.removeValue(forKey: "THREADBEAR_THREAD_ID")
        env.removeValue(forKey: "THREADBARE_THREAD_ID")
        env.removeValue(forKey: "THREADBEAR_THREAD_KEY")
        return env.map { "\($0.key)=\($0.value)" }
    }

    /// Ends the shell as closing a Terminal window does: SIGHUP, which an
    /// interactive shell passes on to its jobs (SIGTERM, LocalProcess's
    /// own terminate, is ignored by an interactive zsh). If it's still
    /// there once the session is let go, LocalProcess's deinit kills it.
    func stop() {
        let pid = view.process?.shellPid ?? 0
        if pid > 0, !exited { kill(pid, SIGHUP) }
        view.removeFromSuperview()
    }

    /// What putting `block` in this shell should do now (TerminalPut).
    func putStep(for block: ShellBlock, now: Date = Date()) -> TerminalPut.Step {
        let state = view.terminalStateSnapshot()
        let pid = view.process?.shellPid ?? 0
        let fd = view.process?.childfd ?? -1
        // The foreground job of the shell's terminal: the shell itself, or
        // whatever it's running (vim, a dev server, a command still going).
        let inFront = pid > 0 && fd >= 0 && tcgetpgrp(fd) == pid
        return TerminalPut.step(for: block, shellInFront: inFront, bracketedPaste: state.bracketedPasteMode,
                                starting: now.timeIntervalSince(startedAt) < Self.startingFor, lastPut: lastPut,
                                screen: { state.visibleRows.map(\.text).joined(separator: "\n") })
    }

    /// Pasted as the terminal's own paste is: bracketed where the shell
    /// asked for that, so a newline in it doesn't run anything.
    func put(_ text: String) {
        lastPut = text
        view.pasteText(text)
        watchForReturn(text)
    }

    /// Shows the hint until the command has gone from the end of the
    /// screen: run, edited, or cleared. Stops by itself after a while, so a
    /// command left there overnight costs nothing.
    private func watchForReturn(_ text: String) {
        returnWatch?.cancel()
        waitingForReturn = true
        returnWatch = Task { @MainActor [weak self] in
            // Time for the shell to echo it before it's looked for.
            try? await Task.sleep(for: .milliseconds(400))
            for _ in 0..<600 {
                guard let self, !Task.isCancelled else { return }
                let screen = self.view.terminalStateSnapshot().visibleRows.map(\.text).joined(separator: "\n")
                if !TerminalPut.endsWith(screen, text) { break }
                try? await Task.sleep(for: .milliseconds(500))
            }
            guard !Task.isCancelled else { return }
            self?.waitingForReturn = false
        }
    }

    /// Deletes the last command put here, which is known to be everything
    /// on the prompt (TerminalPut.replaceLast). Ctrl-U clears a line in
    /// zsh, bash and fish, but zsh and fish take it a line of a multi-line
    /// paste at a time, and fish needs a second go to remove each newline;
    /// any extra ones land on an empty prompt and do nothing.
    func clearLastPut() {
        let lines = (lastPut ?? "").split(separator: "\n", omittingEmptySubsequences: false).count
        view.send(txt: String(repeating: "\u{15}", count: lines * 2))
    }

    /// Text typed into the shell, for the automation.
    func type(_ text: String) {
        view.send(txt: text)
    }

    // MARK: LocalProcessTerminalViewDelegate

    nonisolated func sizeChanged(source: LocalProcessTerminalView, newCols: Int, newRows: Int) {}

    nonisolated func setTerminalTitle(source: LocalProcessTerminalView, title: String) {
        MainActor.assumeIsolated { self.title = title }
    }

    nonisolated func hostCurrentDirectoryUpdate(source: TerminalView, directory: String?) {}

    nonisolated func processTerminated(source: TerminalView, exitCode: Int32?) {
        MainActor.assumeIsolated { self.exited = true }
    }
}

/// The terminal's font and colours: the app's code font, on the transcript's
/// own background, light or dark with the app.
enum TerminalStyle {
    @MainActor static var font: NSFont { .monospacedSystemFont(ofSize: Theme.codeSize + 1, weight: .regular) }

    @MainActor
    static func apply(to view: TerminalView) {
        let dark = view.effectiveAppearance.bestMatch(from: [.darkAqua, .aqua]) == .darkAqua
        view.nativeBackgroundColor = dark ? NSColor(srgbRed: 0.11, green: 0.11, blue: 0.12, alpha: 1) : NSColor(white: 1, alpha: 1)
        view.nativeForegroundColor = dark ? NSColor(white: 0.9, alpha: 1) : NSColor(white: 0.12, alpha: 1)
        view.caretColor = .controlAccentColor
        view.selectedTextBackgroundColor = NSColor.selectedTextBackgroundColor
    }
}

struct TerminalPanel: View {
    @Environment(AppModel.self) private var app

    var body: some View {
        let target = TerminalTarget.for(app.selectedThread) { app.runnerName(of: $0) }
        let session = Terminals.shared.existing(for: target)
        VStack(spacing: 0) {
            if let note = target.note {
                Text(note)
                    .font(.system(size: TypeScale.detail))
                    .foregroundStyle(.secondary)
                    .lineLimit(2)
                    .frame(maxWidth: .infinity, alignment: .leading)
                    .padding(.horizontal, zoomed(10))
                    .padding(.vertical, zoomed(5))
                    .tip(note)
            }
            if let session {
                ZStack(alignment: .bottom) {
                    TerminalHost(session: session, focusTick: Terminals.shared.focusTick)
                        .dockDragDisabled()
                    if session.waitingForReturn {
                        Text("Return runs it")
                            .font(.system(size: TypeScale.detail))
                            .foregroundStyle(.secondary)
                            .padding(.horizontal, zoomed(10))
                            .padding(.vertical, zoomed(6))
                            .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topTrailing)
                            .allowsHitTesting(false)
                    }
                    if session.exited {
                    HStack(spacing: zoomed(8)) {
                        Text("The shell has exited.").foregroundStyle(.secondary)
                        Button("New Shell") { Terminals.shared.restart(target.key) }
                            .tip("Start a new shell in \((target.cwd as NSString).abbreviatingWithTildeInPath)")
                    }
                    .font(.system(size: TypeScale.label))
                    .padding(zoomed(8))
                    .background(.regularMaterial, in: RoundedRectangle(cornerRadius: 8))
                    .padding(zoomed(10))
                    }
                }
            } else {
                // No shell for this thread yet: one starts on a click here
                // or ⌘J, not just from looking.
                Button { Terminals.shared.open(target) } label: {
                    Label("Start a shell", systemImage: WindowDock.terminalSymbol)
                        .font(.system(size: TypeScale.label))
                        .foregroundStyle(.secondary)
                        .frame(maxWidth: .infinity, maxHeight: .infinity)
                        .contentShape(Rectangle())
                }
                .buttonStyle(.plain)
                .tip("Start your login shell in \((target.cwd as NSString).abbreviatingWithTildeInPath)")
            }
        }
    }
}

/// Hosts the session's view, swapping it when the thread changes, and
/// gives it the keyboard when asked (⌘J).
private struct TerminalHost: NSViewRepresentable {
    let session: TerminalSession
    let focusTick: Int

    /// Holds the terminal a little in from the panel's edges, on the
    /// terminal's own background.
    final class Container: NSView {
        var focusTick = 0
        static var inset: NSSize { NSSize(width: zoomed(10), height: zoomed(6)) }

        override init(frame: NSRect) {
            super.init(frame: frame)
            wantsLayer = true
        }

        required init?(coder: NSCoder) { fatalError() }

        override func layout() {
            super.layout()
            let inset = Self.inset
            for view in subviews { view.frame = bounds.insetBy(dx: inset.width, dy: inset.height) }
        }

        override func viewDidChangeEffectiveAppearance() {
            super.viewDidChangeEffectiveAppearance()
            for case let view as TerminalView in subviews {
                TerminalStyle.apply(to: view)
                layer?.backgroundColor = view.nativeBackgroundColor.cgColor
            }
        }
    }

    func makeNSView(context: Context) -> Container {
        let c = Container()
        c.focusTick = focusTick - 1
        return c
    }

    func updateNSView(_ container: Container, context: Context) {
        let view = session.view
        if view.superview !== container {
            container.subviews.forEach { $0.removeFromSuperview() }
            view.removeFromSuperview()
            view.autoresizingMask = []
            container.addSubview(view)
            container.needsLayout = true
            TerminalStyle.apply(to: view)
            container.layer?.backgroundColor = view.nativeBackgroundColor.cgColor
        }
        let font = TerminalStyle.font
        if view.font.pointSize != font.pointSize { view.font = font }
        if container.focusTick != focusTick {
            container.focusTick = focusTick
            DispatchQueue.main.async { view.window?.makeFirstResponder(view) }
        }
    }

    static func dismantleNSView(_ container: Container, coordinator: ()) {
        // The session keeps the view (and its shell) for next time.
        container.subviews.forEach { $0.removeFromSuperview() }
    }
}
