Feature: Files opened in the side pane
  As someone reviewing what an agent made
  I want a file to open beside the conversation, drawn for what it is
  So I can read it without leaving the thread

  Background:
    Given the app is connected to a runner with the demo threads

  @file.host
  Scenario: A click on a file opens it over Highlights, and Back comes back
    When I open the thread "Rebuild the three landing pages"
    And I click the highlight "copy-review.md"
    Then the pane is showing the file "docs/copy-review.md"
    When I press Back in the pane
    Then the pane shows Highlights

  @file.markdown
  Scenario: A Markdown file is drawn as a page
    When I open the thread "Rebuild the three landing pages"
    And I click the highlight "copy-review.md"
    Then the pane shows a "markdown" page with the words "Copy review: home, pricing, docs"
    And the pane's page has the words "Nine words, down from twenty."

  @file.markdown
  Scenario: A Markdown file's menu shows its own text, and the next one opens the same way
    When I open the thread "Rebuild the three landing pages"
    And I click the highlight "copy-review.md"
    Then the pane's Markdown menu says "preview"
    When I pick "markdown" from the pane's Markdown menu
    Then the pane shows a "text" page with the words "# Copy review: home, pricing, docs"
    And the pane's Markdown menu says "markdown"
    When I press Back in the pane
    And I click the highlight "copy-review.md"
    Then the pane shows a "text" page with the words "## Home"
    When I pick "preview" from the pane's Markdown menu
    Then the pane shows a "markdown" page with the words "Nine words, down from twenty."

  @file.find
  Scenario: The find bar finds words on a drawn page, counts them and steps through them
    When I open the thread "Rebuild the three landing pages"
    And I click the highlight "copy-review.md"
    And I look for "the" in the pane's file
    Then the find bar says "the: 1 of 8"
    When I find the next match
    And I find the next match
    Then the find bar says "the: 3 of 8"
    When I find the match before
    Then the find bar says "the: 2 of 8"
    When I look for "zzz" in the pane's file
    Then the find bar says "zzz: not found"
    When I close the find bar
    Then the find bar is gone

  @file.find
  Scenario: The find bar finds words in code whatever their case, and goes round
    When I open the thread "Rebuild the three landing pages"
    And I open "src/hero.ts" in the thread's folder in the pane
    And I look for "WORDS" in the pane's file
    Then the find bar says "WORDS: 1 of 2"
    When I find the next match
    Then the find bar says "WORDS: 2 of 2"
    When I find the next match
    Then the find bar says "WORDS: 1 of 2"

  @file.textview
  Scenario: A code file is shown as selectable, coloured text
    When I open the thread "Rebuild the three landing pages"
    And I open "src/hero.ts" in the thread's folder in the pane
    Then the pane shows a "code" page with the words "export function hero"

  @file.textview
  Scenario: A plain text file is shown as it is
    When I open the thread "Rebuild the three landing pages"
    And I open "logs/docs-build.txt" in the thread's folder in the pane
    Then the pane shows a "text" page with the words "Could not resolve image"

  # A video plays in the system's own player over the pane, its first frame
  # showing until it's played, sized to the film. The GPUI app does this on a
  # Mac; on Windows and Linux it still opens in its own app.
  @file.video @windows-pending @linux-pending
  Scenario: A video opens in the pane in a player, at its own size
    When I open the thread "Rebuild the three landing pages"
    And I open "screens/hero-walkthrough.mp4" in the thread's folder in the pane
    Then the pane is showing the file "screens/hero-walkthrough.mp4"
    And the pane shows a video 1600 by 1058

  # Quick Look draws a PDF in the Swift app; PDFKit's view does in the GPUI
  # app on a Mac.
  @file.quicklook @windows-pending @linux-pending
  Scenario: A PDF opens in the pane
    When I open the thread "Rebuild the three landing pages"
    And I open "docs/launch-brief.pdf" in the thread's folder in the pane
    Then the pane shows the PDF

  @file.picture
  Scenario: A picture is drawn in the pane, and Copy Image copies the picture itself
    When I open the thread "Rebuild the three landing pages"
    And I click the highlight "pricing-before.png"
    Then the pane shows a picture
    When I copy the picture in the pane
    Then it was copied as a picture
