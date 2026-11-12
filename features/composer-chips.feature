Feature: The chips under the message box
  As someone choosing how a thread runs
  I want the model, access and folder right where I type
  So I can change them without leaving the box

  Background:
    Given the app is connected to a runner with the demo threads
    And I open the thread "Morning brief"

  @composer.permission
  Scenario: The permission chip says how the thread runs and changes it
    Then the "permission" menu shows "Full access" ticked
    When I pick "Ask before edits" from the "permission" menu
    Then the composer's permission mode is "ask"
    And the "permission" menu shows "Ask before edits" ticked

  @composer.modelpicker
  Scenario: The model picker lists the models, marking the default
    Then the "model" menu has a row "[x] Fable 5.1 *"
    And the "model" menu has a row "[ ] Opus 5.5"
    And the model chip says "Claude · Fable 5.1"

  @composer.modelpicker
  Scenario: Picking a model changes the chip
    When I pick "Opus 5.5" from the "model" menu
    Then the model chip says "Claude · Opus 5.5"

  @composer.modelpicker
  Scenario: Picking an effort level changes the chip
    When I pick "High" from the "model" menu
    Then the model chip says "Claude · Fable 5.1 · High"

  @composer.modelpicker
  Scenario: A thread pinned to a login says so in the model picker, and Unpin hands it back to a pool
    When I open the thread "Compare SQLite and plain files for the runner"
    Then the "model" menu has a row "# Pinned to mike+3"
    And the "model" menu has a row "--- Pinned to mike+3 · Unpin"
    When I pick "Unpin" from the "model" menu
    Then the "model" menu has no row "--- Pinned to"

  # Without pools there's a login chip: another Mac's thread, as bruce has none.
  @composer.account @remote-threads
  Scenario: The login picker groups the logins by provider, with the room each has left
    When I open the thread "Check the photo library backup"
    Then the "account" menu has a row "# Claude"
    And the "account" menu has a row "[x] mike@example.com - week 78%, resets"
    And the "account" menu has a row "# Codex"
    And the "account" menu has a row "[ ] mike@example.com - week 64%, resets"
    And the "account" menu has a row "--- Usage from Token Stats at"

  @composer.schedulefromsend
  Scenario: Right-clicking Send offers to send the message later
    When I type "Check the backups tonight" in the composer
    Then right-clicking Send offers "Schedule…"
    When I right-click Send and choose "Schedule…"
    Then the schedule editor is open on a new schedule that sends "Check the backups tonight"

  @composer.schedulefromsend @remote-threads
  Scenario: Another Mac's thread has no Schedule… on Send, as its schedules are made there
    When I open the thread "Check the photo library backup"
    And I type "Check it again" in the composer
    Then right-clicking Send doesn't offer "Schedule…"

  @composer.contextring
  Scenario: The context ring says how full the thread is
    When I open the thread "Make uploads resume after a crash"
    Then the "context" menu says "642k of 1M tokens" and "64%"

  @composer.contextring
  Scenario: The context ring's popover says what fills a Claude thread's context
    When I open the thread "Make uploads resume after a crash"
    And I open the context ring's popover
    Then the context popover shows "64% used 642k / 1M"
    And the context popover shows "Messages 610k 61%"
    And the context popover shows "MCP tools (deferred) 173k"
    And the context popover shows "Breakdown from just now"

  # A picture's chip among the words, and its thumbnail over them, open the
  # picture as one in the transcript does: Quick Look in the Swift app, the
  # app's own viewer in GPUI, stepping through the box's other pictures. The
  # GPUI app's click on a thumbnail also checks its × is in its top right
  # corner while the pointer's on it (12 Nov).
  @composer.chiphover
  Scenario: Clicking a picture's chip opens it, among the box's other pictures
    When I paste the picture into the composer
    And I paste the picture into the composer
    And the composer holds 2 pictures
    Then clicking chip 2 opens picture 2 of 2

  @composer.chiphover
  Scenario: Clicking a picture's thumbnail over the box opens it
    When I paste the picture into the composer
    And I paste the picture into the composer
    And the composer holds 2 pictures
    Then clicking the thumbnail of picture 1 opens picture 1 of 2
