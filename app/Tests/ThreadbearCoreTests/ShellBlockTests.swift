import XCTest
@testable import ThreadbearCore

/// ```bash blocks get a play icon that types the command into the Terminal.
final class ShellBlockTests: XCTestCase {
    func testOnlyShellFencesCount() {
        for language in ["sh", "bash", "zsh", "shell", "Bash title=x"] {
            XCTAssertEqual(ShellBlock.from(language: language, code: "ls\n")?.command, "ls", language)
        }
        for language in ["python", "bashful", "console", "", nil] as [String?] {
            XCTAssertNil(ShellBlock.from(language: language, code: "ls"), language ?? "nil")
        }
        XCTAssertNil(ShellBlock.from(language: "bash", code: " \n\n"))
    }

    func testPromptsComeOffOnlyWhenEveryLineHasOne() {
        XCTAssertEqual(ShellBlock.from(language: "sh", code: "$ a\n\n$ b\n")?.command, "a\n\nb")
        XCTAssertEqual(ShellBlock.from(language: "sh", code: "$ a\nb")?.command, "$ a\nb")
        XCTAssertEqual(ShellBlock.from(language: "sh", code: "$HOME/x")?.command, "$HOME/x")
    }

    // Each of these could make what runs differ from what's on screen, or
    // run a line without Return being pressed.
    func testRefusesWhatCantBeTypedSafely() {
        let bad = [
            "echo hi\u{1b}[201~\nrm -rf ~", // ends a bracketed paste early
            "echo\u{07}", "a\rb", "a\u{7f}b", "a\u{9b}201~",
            "rm\u{200b} x", // zero width
            "echo \u{202e}cod", // right-to-left override
            "a\u{2028}b",
        ]
        for code in bad { XCTAssertNil(ShellBlock.from(language: "bash", code: code), code.debugDescription) }
        XCTAssertNotNil(ShellBlock.from(language: "bash", code: "echo 'héllo 🐻'\n\tls"))
        XCTAssertNil(ShellBlock.from(language: "bash", code: String(repeating: "a", count: ShellBlock.maxSource + 1)))
    }

    func testMayHold() {
        XCTAssertTrue(ShellBlock.mayHold("x\n```bash\nls\n```"))
        XCTAssertFalse(ShellBlock.mayHold("no fence, just bash"))
        XCTAssertFalse(ShellBlock.mayHold("```python\nx\n```"))
    }

    private func step(_ command: String = "ls", front: Bool = true, bracketed: Bool = true, starting: Bool = false, screen: String = "~ $") -> TerminalPut.Step {
        TerminalPut.step(for: ShellBlock(command: command), shellInFront: front, bracketedPaste: bracketed, starting: starting, screen: { screen })
    }

    func testTypesAtAPrompt() {
        XCTAssertEqual(step(), .type)
        XCTAssertEqual(step("a\nb"), .type)
    }

    func testNeverTypesIntoARunningProgram() {
        // vim turns bracketed paste on too, so that alone isn't enough.
        XCTAssertEqual(step(front: false, bracketed: true), .busy)
    }

    func testWaitsForAShellThatsStarting() {
        XCTAssertEqual(step(bracketed: false, starting: true), .wait)
        XCTAssertEqual(step(front: false, starting: true), .wait)
        XCTAssertEqual(step(starting: true), .type)
    }

    func testAShellWithoutBracketedPasteOnlyGetsOneLine() {
        XCTAssertEqual(step(bracketed: false), .type)
        XCTAssertEqual(step("a\nb", bracketed: false), .busy)
        XCTAssertEqual(step("a\tb", bracketed: false), .busy)
    }

    func testADifferentBlockReplacesTheOneStillAtThePrompt() {
        func put(_ command: String, screen: String) -> TerminalPut.Step {
            TerminalPut.step(for: ShellBlock(command: command), shellInFront: true, bracketedPaste: true, starting: false, lastPut: "npm test", screen: { screen })
        }
        XCTAssertEqual(put("ls", screen: "~ $ npm test   \n"), .replaceLast)
        XCTAssertEqual(put("npm test", screen: "~ $ npm test"), .alreadyThere)
        // It was run: there's a new prompt, so nothing's left to replace.
        XCTAssertEqual(put("ls", screen: "~ $ npm test\npassed\n~ $ "), .type)
        // Someone's own half-typed line isn't this feature's to delete.
        XCTAssertEqual(put("ls", screen: "~ $ git sta"), .type)
    }

    func testASecondClickDoesntTypeItAgain() {
        XCTAssertEqual(step("npx convex run --prod x", screen: "~ $ npx convex\n run --prod x   \n\n"), .alreadyThere)
        XCTAssertEqual(step("ls", screen: "~ $ ls\nfile\n~ $ "), .type)
    }
}
