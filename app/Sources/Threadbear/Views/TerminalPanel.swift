import AppKit
import SwiftTerm
import SwiftUI
import ThreadbearCore

/// The built-in Terminal panel (docs/protocol.md, "The Terminal panel"): a
/// shell in the thread's folder, running on the runner that has the thread,
/// drawn by SwiftTerm. One per thread (`.id(thread.id)`); the shell keeps
/// going when the panel goes, and its replay redraws the screen next time.
struct TerminalPanel: View {
    @Environment(AppModel.self) private var app
    @Environment(\.isDockGhost) private var isGhost
    let thread: ThreadSummary
    @State private var model: TerminalPanelModel

    init(thread: ThreadSummary) {
        self.thread = thread
        _model = State(initialValue: TerminalPanelModel(threadId: thread.id))
    }

    var body: some View {
        if isGhost {
            // The dock's drag picture: a second terminal would attach and
            // take the shell's output from the real one.
            Color(nsColor: .textBackgroundColor)
        } else {
            content
                .onAppear { model.appear(app: app) }
                .onDisappear { model.disappear() }
        }
    }

    @ViewBuilder
    private var content: some View {
        switch model.phase {
        case .tooOld:
            PaneNote(text: TerminalText.tooOld)
        case .failed(let message):
            VStack(spacing: zoomed(10)) {
                PaneNote(text: message).fixedSize(horizontal: false, vertical: true)
                Button("Try again") { model.open() }
                    .tip("Start the shell again, or attach to the one that's running")
            }
            .frame(maxWidth: .infinity, maxHeight: .infinity)
        case .opening, .live, .ended:
            VStack(spacing: 0) {
                TerminalScreen(model: model)
                    .padding(.leading, zoomed(6))
                    .background(Color(nsColor: .textBackgroundColor))
                    .overlay {
                        if case .opening = model.phase {
                            VStack(spacing: zoomed(8)) {
                                SpinnerMark(color: .secondaryLabelColor).frame(width: zoomed(16), height: zoomed(16))
                                Text(TerminalText.starting)
                                    .font(.system(size: TypeScale.label))
                                    .foregroundStyle(.secondary)
                            }
                            .frame(maxWidth: .infinity, maxHeight: .infinity)
                            .background(Color(nsColor: .textBackgroundColor))
                        }
                    }
                if case .ended(let why) = model.phase {
                    endedBar(why)
                }
            }
        }
    }

    private func endedBar(_ why: String) -> some View {
        HStack(spacing: zoomed(8)) {
            Text(why)
                .font(.system(size: TypeScale.label))
                .foregroundStyle(.secondary)
                .lineLimit(2)
            Spacer(minLength: zoomed(8))
            Button("New shell") { model.restart() }
                .tip("Start a new shell in this thread's folder")
                .busy(app.isBusy(BusyKey.terminalRestart(thread.id)))
        }
        .padding(.horizontal, zoomed(10))
        .padding(.vertical, zoomed(6))
        .background(.bar)
        .overlay(alignment: .top) { Rectangle().fill(.separator).frame(height: 0.5) }
    }
}

// MARK: - The model

/// One panel's shell: opening or attaching, the bytes both ways, and why
/// it ended. Output comes through `TerminalHub`, by thread.
@MainActor
@Observable
final class TerminalPanelModel {
    enum Phase: Equatable {
        case opening
        case live
        /// Why, in plain words, with New shell under it.
        case ended(String)
        case tooOld
        case failed(String)
    }

    let threadId: String
    private(set) var phase: Phase = .opening

    @ObservationIgnored private weak var app: AppModel?
    @ObservationIgnored weak var view: ThreadTerminalView? {
        didSet { if view != nil, let pending = pendingScreen { pendingScreen = nil; draw(pending) } }
    }
    /// The shell this panel shows, and the connection it was opened on.
    @ObservationIgnored private var terminalId: String?
    @ObservationIgnored private var client: RPCClient?
    @ObservationIgnored private var showing = false
    @ObservationIgnored private var opening = false
    /// Output that came while the open was on its way, fed after the replay.
    @ObservationIgnored private var early: [(terminalId: String, data: Data)] = []
    /// A replay with no view yet to draw it in.
    @ObservationIgnored private var pendingScreen: (replay: Data, after: [Data])?

    @ObservationIgnored private var size = TerminalSize.fallback
    @ObservationIgnored private var sizeKnown = false
    @ObservationIgnored private var waitingForSize: Task<Void, Never>?
    @ObservationIgnored private var resizeTask: Task<Void, Never>?

    @ObservationIgnored private var input = TerminalInputBuffer()
    @ObservationIgnored private var sending = false

    init(threadId: String) {
        self.threadId = threadId
    }

    // MARK: Showing

    func appear(app: AppModel) {
        self.app = app
        showing = true
        TerminalHub.shared.register(self)
        // Open once the view has said its size, so the shell starts the
        // width it's drawn at; a moment at most.
        if sizeKnown {
            open()
        } else {
            waitingForSize?.cancel()
            waitingForSize = Task { [weak self] in
                try? await Task.sleep(for: .milliseconds(250))
                guard let self, !Task.isCancelled, self.terminalId == nil, !self.opening else { return }
                self.open()
            }
        }
    }

    func disappear() {
        showing = false
        waitingForSize?.cancel()
        resizeTask?.cancel()
        TerminalHub.shared.unregister(self)
        guard let terminalId, let client else { return }
        let rest = input.take()
        self.terminalId = nil
        self.client = nil
        Task {
            for piece in rest { _ = try? await client.terminalInput(terminalId: terminalId, data: piece) }
            do {
                try await client.terminalDetach(terminalId: terminalId)
            } catch {
                AppLog.write("terminal", "detach failed: \(error.localizedDescription)")
            }
        }
    }

    // MARK: Opening

    /// Starts the thread's shell, or attaches to the one that's running.
    /// Shows "Starting the shell" with a spinner until it answers, except
    /// `quietly`, attaching again after a reconnect, where the screen stays
    /// as it was until the replay redraws it.
    func open(quietly: Bool = false) {
        guard let app, showing, !opening else { return }
        waitingForSize?.cancel()
        opening = true
        if !quietly { phase = .opening }
        early = []
        Task {
            defer { opening = false }
            do {
                let client = try await app.openClient(for: threadId)
                let session = try await client.terminalOpen(threadId: threadId, cols: size.cols, rows: size.rows)
                attach(session, on: client)
            } catch {
                fail(error)
            }
        }
    }

    /// New shell: the runner ends the old one first. The button spins
    /// until it lands.
    func restart() {
        guard let app else { return }
        let size = size
        app.whileBusy(BusyKey.terminalRestart(threadId)) { [weak self, threadId] in
            do {
                let client = try await app.openClient(for: threadId)
                let session = try await client.terminalRestart(threadId: threadId, cols: size.cols, rows: size.rows)
                self?.attach(session, on: client)
            } catch {
                self?.fail(error)
            }
        }
    }

    /// The connection came back (this Mac's runner, or the thread's own on
    /// the direct link): attach again, if it's a new one.
    func reconnected() {
        guard showing, !opening, let app else { return }
        guard terminalId != nil || phase.isFailed else { return }
        guard app.client(for: threadId) !== client else { return }
        terminalId = nil
        client = nil
        open(quietly: !phase.isFailed)
    }

    private func attach(_ session: TerminalSession, on client: RPCClient) {
        guard showing else {
            Task { try? await client.terminalDetach(terminalId: session.terminalId) }
            return
        }
        terminalId = session.terminalId
        self.client = client
        let after = early.filter { $0.terminalId == session.terminalId }.map(\.data)
        early = []
        draw((session.replayBytes, after))
        phase = session.alive ? .live : .ended(TerminalText.ended(reason: session.reason, exitCode: session.exitCode))
        if session.alive { view?.focusIfInFront() }
        if !input.isEmpty { Task { await flush() } }
    }

    private func draw(_ screen: (replay: Data, after: [Data])) {
        guard let view else { pendingScreen = screen; return }
        view.resetToInitialState()
        if !screen.replay.isEmpty { view.feed(byteArray: ArraySlice([UInt8](screen.replay))) }
        for data in screen.after { view.feed(byteArray: ArraySlice([UInt8](data))) }
    }

    private func fail(_ error: Error) {
        guard showing else { return }
        if TerminalText.runnerTooOld(error) {
            phase = .tooOld
        } else {
            AppLog.write("terminal", "couldn't open the shell: \(error.localizedDescription)")
            phase = .failed(error.localizedDescription)
        }
    }

    // MARK: From the runner (TerminalHub)

    func output(terminalId: String, data: Data) {
        if opening, self.terminalId == nil {
            early.append((terminalId, data))
            return
        }
        guard terminalId == self.terminalId else { return }
        view?.feed(byteArray: ArraySlice([UInt8](data)))
    }

    func exited(_ exit: TerminalExit) {
        guard exit.terminalId == terminalId else { return }
        input = TerminalInputBuffer()
        phase = .ended(TerminalText.ended(reason: exit.reason, exitCode: exit.exitCode))
    }

    // MARK: From the view

    /// Keys and pastes, a few ms' worth at a time, and while one send is on
    /// its way the next waits for it, so they reach the shell in order.
    func type(_ bytes: Data) {
        guard phase == .live || opening else { return }
        guard input.append(bytes), !sending else { return }
        Task {
            try? await Task.sleep(for: .milliseconds(4))
            await flush()
        }
    }

    private func flush() async {
        guard !sending, let terminalId, let client else { return }
        let pieces = input.take()
        guard !pieces.isEmpty else { return }
        sending = true
        for piece in pieces {
            do {
                try await client.terminalInput(terminalId: terminalId, data: piece)
            } catch {
                AppLog.write("terminal", "typing didn't reach the shell: \(error.localizedDescription)")
                break
            }
        }
        sending = false
        if !input.isEmpty, self.terminalId == terminalId { await flush() }
    }

    func resized(cols: Int, rows: Int) {
        let next = TerminalSize.clamped(cols: cols, rows: rows)
        let first = !sizeKnown
        sizeKnown = true
        guard next != size || first else { return }
        size = next
        if first, showing, terminalId == nil, !opening, waitingForSize != nil {
            return open()
        }
        resizeTask?.cancel()
        resizeTask = Task { [weak self] in
            try? await Task.sleep(for: .milliseconds(100))
            guard let self, !Task.isCancelled, let terminalId = self.terminalId, let client = self.client else { return }
            do {
                try await client.terminalResize(terminalId: terminalId, cols: self.size.cols, rows: self.size.rows)
            } catch {
                AppLog.write("terminal", "resize failed: \(error.localizedDescription)")
            }
        }
    }
}

private extension TerminalPanelModel.Phase {
    var isFailed: Bool { if case .failed = self { true } else { false } }
}

// MARK: - Routing the runner's notifications

/// Hands `terminal.output` and `terminal.exited` to the panel showing that
/// thread's shell, from any runner connection, and tells panels when a
/// connection comes back.
@MainActor
final class TerminalHub {
    static let shared = TerminalHub()

    private final class Weak { weak var model: TerminalPanelModel?; init(_ m: TerminalPanelModel) { model = m } }
    private var models: [String: Weak] = [:]

    func register(_ model: TerminalPanelModel) { models[model.threadId] = Weak(model) }

    func unregister(_ model: TerminalPanelModel) {
        if models[model.threadId]?.model === model { models[model.threadId] = nil }
    }

    func output(terminalId: String, threadId: String, data: Data) {
        models[threadId]?.model?.output(terminalId: terminalId, data: data)
    }

    func exited(_ exit: TerminalExit) {
        models[exit.threadId]?.model?.exited(exit)
    }

    func reconnected() {
        for entry in models.values { entry.model?.reconnected() }
    }
}

// MARK: - The screen

/// SwiftTerm's AppKit view. It takes the keyboard when clicked, so typing
/// goes to the shell and not the composer.
final class ThreadTerminalView: TerminalView {
    // Clicks count while Threadbear isn't the app in front (FirstClick.swift).
    override func acceptsFirstMouse(for event: NSEvent?) -> Bool { true }

    override func mouseDown(with event: NSEvent) {
        if window?.firstResponder !== self { window?.makeFirstResponder(self) }
        super.mouseDown(with: event)
    }

    /// After a shell starts, if nothing else in the window has the keyboard
    /// that matters (the composer keeps it when Mike is typing there).
    func focusIfInFront() {
        guard let window, window.isKeyWindow else { return }
        if window.firstResponder === window || window.firstResponder == nil { window.makeFirstResponder(self) }
    }
}

private struct TerminalScreen: NSViewRepresentable {
    let model: TerminalPanelModel

    func makeCoordinator() -> Coordinator { Coordinator(model: model) }

    func makeNSView(context: Context) -> ThreadTerminalView {
        let view = ThreadTerminalView(frame: .zero, font: Theme.codeFont)
        view.terminalDelegate = context.coordinator
        // Follow light and dark: dynamic colours, which SwiftTerm resolves
        // again when the appearance changes.
        view.nativeForegroundColor = .textColor
        view.nativeBackgroundColor = .textBackgroundColor
        view.selectedTextBackgroundColor = .selectedTextBackgroundColor
        view.caretColor = .controlAccentColor
        // Option types the Mac's characters (# on a UK keyboard, accents).
        view.optionAsMetaKey = false
        model.view = view
        return view
    }

    func updateNSView(_ view: ThreadTerminalView, context: Context) {
        context.coordinator.model = model
        if model.view !== view { model.view = view }
        // The app's text size follows its zoom.
        let font = Theme.codeFont
        if view.font.pointSize != font.pointSize { view.font = font }
    }

    @MainActor
    final class Coordinator: NSObject, TerminalViewDelegate {
        var model: TerminalPanelModel

        init(model: TerminalPanelModel) { self.model = model }

        func sizeChanged(source: TerminalView, newCols: Int, newRows: Int) {
            model.resized(cols: newCols, rows: newRows)
        }

        func send(source: TerminalView, data: ArraySlice<UInt8>) {
            model.type(Data(data))
        }

        func setTerminalTitle(source: TerminalView, title: String) {}
        func hostCurrentDirectoryUpdate(source: TerminalView, directory: String?) {}
        func scrolled(source: TerminalView, position: Double) {}
        func rangeChanged(source: TerminalView, startY: Int, endY: Int) {}

        /// A program copying (OSC 52) puts it on the pasteboard, as Terminal does.
        func clipboardCopy(source: TerminalView, content: Data) {
            guard let text = String(data: content, encoding: .utf8) else { return }
            NSPasteboard.general.clearContents()
            NSPasteboard.general.setString(text, forType: .string)
        }
    }
}
