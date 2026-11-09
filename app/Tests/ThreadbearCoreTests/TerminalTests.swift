import XCTest
@testable import ThreadbearCore

/// The Terminal panel's side of docs/protocol.md, "The Terminal panel".
final class TerminalTests: XCTestCase {
    private func answering(_ result: [String: Any]) -> FakeTransport {
        let transport = FakeTransport()
        let body = E.json(result)
        transport.onSend = { request in [#"{"jsonrpc":"2.0","id":\#(request["id"] as! Int),"result":\#(body)}"#] }
        return transport
    }

    func testHelloSaysTheAppHasATerminalPanel() async throws {
        let transport = answering(["runnerVersion": "0.1.0", "protocol": 1, "accounts": [], "defaults": ["cwd": "/w", "accountId": "a"]])
        let client = RPCClient(transport: transport)
        try await client.start()
        _ = try await client.hello(version: "1.0")
        let params = transport.sent[0]["params"] as! [String: Any]
        XCTAssertEqual(params["terminalPanel"] as? Bool, true)
        XCTAssertEqual(params["browserPanel"] as? Bool, true)
    }

    func testOpenSendsTheSizeAndDecodesTheReplay() async throws {
        let replay = Data("$ ls\r\nREADME.md\r\n".utf8)
        let transport = answering(["terminalId": "term1", "replay": replay.base64EncodedString(), "alive": true,
                                   "startedAt": "2026-11-09T08:00:00Z", "extra": 1])
        let client = RPCClient(transport: transport)
        try await client.start()
        let opened = try await client.terminalOpen(threadId: "th1", cols: 120, rows: 40)
        XCTAssertEqual(opened.terminalId, "term1")
        XCTAssertTrue(opened.alive)
        XCTAssertNil(opened.exitCode)
        XCTAssertEqual(opened.replayBytes, replay)
        let sent = transport.sent[0]
        XCTAssertEqual(sent["method"] as? String, "terminal.open")
        let params = sent["params"] as! [String: Any]
        XCTAssertEqual(params["threadId"] as? String, "th1")
        XCTAssertEqual(params["cols"] as? Int, 120)
        XCTAssertEqual(params["rows"] as? Int, 40)
    }

    func testAnEndedShellComesBackWithItsExitCode() async throws {
        let transport = answering(["terminalId": "term1", "replay": "", "alive": false, "startedAt": "2026-11-09T08:00:00Z", "exitCode": 130, "reason": "archived"])
        let client = RPCClient(transport: transport)
        try await client.start()
        let opened = try await client.terminalRestart(threadId: "th1", cols: 80, rows: 24)
        XCTAssertEqual(transport.sent[0]["method"] as? String, "terminal.restart")
        XCTAssertFalse(opened.alive)
        XCTAssertEqual(opened.exitCode, 130)
        XCTAssertEqual(opened.reason, .archived)
        XCTAssertEqual(opened.replayBytes, Data())
    }

    func testInputResizeDetachAndKill() async throws {
        let transport = answering([:])
        let client = RPCClient(transport: transport)
        try await client.start()
        try await client.terminalInput(terminalId: "term1", data: Data("ls\r".utf8))
        try await client.terminalResize(terminalId: "term1", cols: 100, rows: 30)
        try await client.terminalDetach(terminalId: "term1")
        try await client.terminalKill(terminalId: "term1")
        XCTAssertEqual(transport.sent.map { $0["method"] as? String }, ["terminal.input", "terminal.resize", "terminal.detach", "terminal.kill"])
        let input = transport.sent[0]["params"] as! [String: Any]
        XCTAssertEqual(input["terminalId"] as? String, "term1")
        XCTAssertEqual(input["data"] as? String, Data("ls\r".utf8).base64EncodedString())
        let resize = transport.sent[1]["params"] as! [String: Any]
        XCTAssertEqual(resize["cols"] as? Int, 100)
        XCTAssertEqual(resize["rows"] as? Int, 30)
    }

    func testOutputAndExitNotificationsAreDecoded() async throws {
        let transport = FakeTransport()
        let client = RPCClient(transport: transport)
        try await client.start()
        let bytes = Data([0x1b, 0x5b, 0x48, 0xff, 0x00])
        transport.push(E.json(["jsonrpc": "2.0", "method": "terminal.output", "params": ["terminalId": "term1", "threadId": "th1", "data": bytes.base64EncodedString()]]))
        transport.push(E.json(["jsonrpc": "2.0", "method": "terminal.exited", "params": ["terminalId": "term1", "threadId": "th1", "exitCode": NSNull(), "reason": "archived"]]))
        transport.push(E.json(["jsonrpc": "2.0", "method": "terminal.exited", "params": ["terminalId": "term2", "threadId": "th2", "exitCode": 1, "reason": "something new"]]))
        // Not base64: dropped rather than fed to the screen as nothing.
        transport.push(E.json(["jsonrpc": "2.0", "method": "terminal.output", "params": ["terminalId": "term1", "threadId": "th1", "data": "%%%"]]))
        transport.push(E.json(["jsonrpc": "2.0", "method": "thread.deleted", "params": ["threadId": "th9"]]))
        var iterator = client.notifications.makeAsyncIterator()
        guard case .terminalOutput(let id, let threadId, let data)? = await iterator.next() else { return XCTFail("no output") }
        XCTAssertEqual(id, "term1")
        XCTAssertEqual(threadId, "th1")
        XCTAssertEqual(data, bytes)
        guard case .terminalExited(let exit)? = await iterator.next() else { return XCTFail("no exit") }
        XCTAssertEqual(exit, TerminalExit(terminalId: "term1", threadId: "th1", exitCode: nil, reason: .archived))
        guard case .terminalExited(let odd)? = await iterator.next() else { return XCTFail("no second exit") }
        XCTAssertEqual(odd.reason, .unknown)
        XCTAssertEqual(odd.exitCode, 1)
        guard case .threadDeleted? = await iterator.next() else { return XCTFail("the bad output wasn't dropped") }
    }

    func testWhyTheShellEndedInPlainWords() {
        XCTAssertEqual(TerminalText.ended(reason: .exited, exitCode: 0), "The shell ended.")
        XCTAssertEqual(TerminalText.ended(reason: .exited, exitCode: nil), "The shell ended.")
        XCTAssertEqual(TerminalText.ended(reason: .exited, exitCode: 2), "The shell ended with code 2.")
        XCTAssertEqual(TerminalText.ended(reason: .runner, exitCode: nil), "The shell ended when the runner restarted.")
        XCTAssertEqual(TerminalText.ended(reason: .archived, exitCode: nil), "The thread was archived, so its shell ended.")
        XCTAssertEqual(TerminalText.ended(reason: .deleted, exitCode: nil), "The thread was deleted, so its shell ended.")
        XCTAssertEqual(TerminalText.ended(reason: .idle, exitCode: nil), "The shell ended after 8 hours with nobody looking at it.")
        XCTAssertEqual(TerminalText.ended(reason: .evicted, exitCode: nil), "The shell ended to make room, as too many were running.")
        XCTAssertEqual(TerminalText.ended(reason: .killed, exitCode: nil), "The shell was stopped.")
        XCTAssertEqual(TerminalText.ended(reason: .unknown, exitCode: 1), "The shell ended with code 1.")
        // An `alive: false` open says no reason.
        XCTAssertEqual(TerminalText.ended(reason: nil, exitCode: 0), "The shell ended.")
        for text in [TerminalText.tooOld, TerminalText.starting, TerminalText.ended(reason: .runner, exitCode: nil)] {
            XCTAssertFalse(text.contains("\u{2014}"), "no em dashes in UI copy")
        }
    }

    func testAnOldRunnerIsToldApartFromOtherErrors() {
        XCTAssertTrue(TerminalText.runnerTooOld(RPCError.server(code: -32601, message: "Method not found")))
        XCTAssertFalse(TerminalText.runnerTooOld(RPCError.server(code: -32000, message: "Open it on the Mac that has it")))
        XCTAssertFalse(TerminalText.runnerTooOld(RPCError.disconnected(nil)))
        XCTAssertEqual(TerminalText.tooOld, "This runner is too old for a terminal. Update it to use one.")
    }

    func testTypingIsCoalescedAndSentInPiecesTheRunnerTakes() {
        var buffer = TerminalInputBuffer()
        XCTAssertTrue(buffer.isEmpty)
        XCTAssertEqual(buffer.take(), [])
        XCTAssertTrue(buffer.append(Data("l".utf8)), "the first byte starts the wait")
        XCTAssertFalse(buffer.append(Data("s".utf8)), "already waiting")
        XCTAssertFalse(buffer.append(Data("\r".utf8)))
        XCTAssertEqual(buffer.take(), [Data("ls\r".utf8)])
        XCTAssertTrue(buffer.isEmpty)
        XCTAssertFalse(buffer.append(Data()), "nothing to send")

        // A big paste goes in pieces of at most 64 KB, in order.
        let paste = Data((0..<(TerminalInputBuffer.maxChunk * 2 + 10)).map { UInt8($0 % 251) })
        XCTAssertTrue(buffer.append(paste))
        let pieces = buffer.take()
        XCTAssertEqual(pieces.map(\.count), [TerminalInputBuffer.maxChunk, TerminalInputBuffer.maxChunk, 10])
        XCTAssertEqual(pieces.reduce(Data(), +), paste)
        XCTAssertEqual(TerminalInputBuffer.maxChunk, 64 * 1024)
    }

    func testSizesAreKeptToWhatTheRunnerTakes() {
        XCTAssertEqual(TerminalSize.clamped(cols: 0, rows: -3), TerminalSize(cols: 1, rows: 1))
        XCTAssertEqual(TerminalSize.clamped(cols: 5000, rows: 24), TerminalSize(cols: 1000, rows: 24))
        XCTAssertEqual(TerminalSize.clamped(cols: 80, rows: 24), TerminalSize(cols: 80, rows: 24))
    }
}
