Feature: Sending a message
  As someone talking to an agent
  I want what I type to go straight in and the reply to stream back
  So it feels like a conversation

  Background:
    Given the app is connected to a runner with the demo threads
    And I open the thread "Morning brief"

  @composer.return @composer.send
  Scenario: Return sends what I typed
    When I type "Can you check the backups?" in the composer
    And I press Return
    Then the composer is empty
    And the transcript shows my message "Can you check the backups?"
    And the reply streams in and the thread goes back to idle

  @composer.return
  Scenario: Shift-Return starts a new line instead of sending
    When I type "first line" in the composer
    And I press Shift-Return
    Then the composer holds a new line after "first line"
    And the transcript doesn't show my message "first line"

  @composer.listcontinue
  Scenario: Shift-Return carries a numbered list on
    When I type "1. first" in the composer
    And I press Shift-Return
    Then the composer says the lines "1. first\n2. "

  @composer.listcontinue
  Scenario: Shift-Return on an empty list item ends the list
    When I type "1. first" in the composer
    And I press Shift-Return
    And I press Shift-Return
    Then the composer says the lines "1. first\n"

  @composer.textbox
  Scenario: Pasting puts the words where the cursor is
    When I type "Hello " in the composer
    And I paste "there\nworld" into the composer
    Then the composer says the lines "Hello there\nworld"

  @composer.textbox
  Scenario: What I've typed stays with its thread
    When I type "half a thought" in the composer
    And I open the thread "Rename the Settings tabs"
    Then the composer is empty
    When I open the thread "Morning brief"
    Then the composer says "half a thought"

  @composer.send
  Scenario: Stop ends a turn that's running
    When I send "Please keep working on this for a while"
    And I stop the turn
    Then the thread isn't running any more
