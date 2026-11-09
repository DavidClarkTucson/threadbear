import Foundation

// The Terminal panel (docs/protocol.md, "The Terminal panel"): a shell
// beside each thread, on the runner that has the thread. What goes either
// way is bytes, sent as base64.

/// `terminal.open` and `terminal.restart`'s answer.
public struct TerminalSession: Decodable, Sendable, Equatable {
    public var terminalId: String
    /// The last of what the shell wrote, to redraw the screen with.
    public var replay: String
    /// False with `exitCode` when the thread's last shell has ended and
    /// nothing new was started.
    public var alive: Bool
    public var startedAt: String?
    public var exitCode: Int?
    /// With `alive` false: why it ended, as in `terminal.exited`.
    public var reason: TerminalExitReason?

    public init(terminalId: String, replay: String = "", alive: Bool = true, startedAt: String? = nil, exitCode: Int? = nil, reason: TerminalExitReason? = nil) {
        self.terminalId = terminalId
        self.replay = replay
        self.alive = alive
        self.startedAt = startedAt
        self.exitCode = exitCode
        self.reason = reason
    }

    public init(from decoder: Decoder) throws {
        let c = try decoder.container(keyedBy: CodingKeys.self)
        terminalId = try c.decode(String.self, forKey: .terminalId)
        replay = (try? c.decodeIfPresent(String.self, forKey: .replay)) ?? ""
        alive = (try? c.decodeIfPresent(Bool.self, forKey: .alive)) ?? true
        startedAt = try? c.decodeIfPresent(String.self, forKey: .startedAt)
        exitCode = try? c.decodeIfPresent(Int.self, forKey: .exitCode)
        reason = (try? c.decodeIfPresent(String.self, forKey: .reason)).flatMap { $0.flatMap(TerminalExitReason.init(rawValue:)) ?? .unknown }
    }

    private enum CodingKeys: String, CodingKey { case terminalId, replay, alive, startedAt, exitCode, reason }

    /// The replay as bytes; nothing if it isn't base64.
    public var replayBytes: Data { Data(base64Encoded: replay) ?? Data() }
}

/// Why a shell ended, from `terminal.exited`. A word this app doesn't know
/// is `unknown`, so a newer runner's reasons still end the shell here.
public enum TerminalExitReason: String, Sendable, Equatable {
    case exited, archived, deleted, idle, runner, evicted, killed, unknown
}

/// `terminal.exited`.
public struct TerminalExit: Sendable, Equatable {
    public var terminalId: String
    public var threadId: String
    public var exitCode: Int?
    public var reason: TerminalExitReason

    public init(terminalId: String, threadId: String, exitCode: Int?, reason: TerminalExitReason) {
        self.terminalId = terminalId
        self.threadId = threadId
        self.exitCode = exitCode
        self.reason = reason
    }
}

struct TerminalOutputParams: Decodable {
    var terminalId: String
    var threadId: String
    var data: String
}

struct TerminalExitedParams: Decodable {
    var terminalId: String
    var threadId: String
    var exitCode: Int?
    var reason: String?

    var exit: TerminalExit {
        TerminalExit(terminalId: terminalId, threadId: threadId, exitCode: exitCode,
                     reason: reason.flatMap(TerminalExitReason.init(rawValue:)) ?? .unknown)
    }
}

/// A terminal's size in character cells, as the runner takes it.
public struct TerminalSize: Equatable, Sendable {
    public var cols: Int
    public var rows: Int

    public init(cols: Int, rows: Int) {
        self.cols = cols
        self.rows = rows
    }

    /// 1 to 1000 each.
    public static func clamped(cols: Int, rows: Int) -> TerminalSize {
        TerminalSize(cols: min(max(cols, 1), 1000), rows: min(max(rows, 1), 1000))
    }

    /// Before the view has said how big it is.
    public static let fallback = TerminalSize(cols: 80, rows: 24)
}

/// Keystrokes waiting to go to the shell. Typing fast, or a paste, lands
/// as one `terminal.input` instead of one a key; a big paste goes in
/// pieces the runner takes (64 KB at most a call).
public struct TerminalInputBuffer: Sendable {
    public static let maxChunk = 64 * 1024

    private var pending = Data()

    public init() {}

    public var isEmpty: Bool { pending.isEmpty }

    /// Adds bytes. True when the buffer was empty, so the caller starts
    /// its short wait before sending.
    @discardableResult
    public mutating func append(_ bytes: Data) -> Bool {
        guard !bytes.isEmpty else { return false }
        let wasEmpty = pending.isEmpty
        pending.append(bytes)
        return wasEmpty
    }

    /// Everything waiting, in order, in pieces of at most `maxChunk`.
    public mutating func take() -> [Data] {
        guard !pending.isEmpty else { return [] }
        var pieces: [Data] = []
        var start = pending.startIndex
        while start < pending.endIndex {
            let end = min(start + Self.maxChunk, pending.endIndex)
            pieces.append(Data(pending[start..<end]))
            start = end
        }
        pending = Data()
        return pieces
    }
}

/// The panel's words.
public enum TerminalText {
    public static let starting = "Starting the shell\u{2026}"
    public static let tooOld = "This runner is too old for a terminal. Update it to use one."

    /// Why the shell ended. `reason` is nil from a runner that doesn't say.
    public static func ended(reason: TerminalExitReason?, exitCode: Int?) -> String {
        switch reason {
        case .runner: return "The shell ended when the runner restarted."
        case .archived: return "The thread was archived, so its shell ended."
        case .deleted: return "The thread was deleted, so its shell ended."
        case .idle: return "The shell ended after 8 hours with nobody looking at it."
        case .evicted: return "The shell ended to make room, as too many were running."
        case .killed: return "The shell was stopped."
        case .exited, .unknown, nil:
            if let exitCode, exitCode != 0 { return "The shell ended with code \(exitCode)." }
            return "The shell ended."
        }
    }

    /// An older runner, without the terminal's methods.
    public static func runnerTooOld(_ error: Error) -> Bool {
        if case RPCError.server(let code, _) = error, code == -32601 { return true }
        return false
    }
}
