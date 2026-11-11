Feature: Editing a message
  As someone who sent a message I'd put differently
  I want to change it and carry on from there
  So the thread goes the way I meant

  Background:
    Given the app is connected to a runner with the demo threads

  @transcript.edit @composer.editing
  Scenario: The pencil puts a message in the box, and sending it goes back to there
    When I open the thread "Morning brief"
    And I edit my first message
    Then the composer holds my first message to edit
    When I press Escape
    Then the composer isn't editing

  @transcript.edit @composer.editing
  Scenario: Sending an edit replaces the message and carries on from it
    When I open the thread "Morning brief"
    And I edit my first message
    And I replace what's in the composer with "Only the calendar, please"
    And I press Return
    Then the composer isn't editing
    And the transcript has my message "Only the calendar, please"

  # Daybreak finding 86 (11 Nov): with one approval waiting, Return in an
  # empty box allows it, and the GPUI app did that mid-edit too.
  @transcript.edit @composer.editing @transcript.approval
  Scenario: Return in an emptied edit allows nothing
    When I open the thread "Make uploads resume after a crash"
    Then an approval card asks to run "rm -rf .build DerivedData"
    When I edit my first message
    And I replace what's in the composer with ""
    And I press Return
    Then the composer is still editing my first message
    And the approval for "rm -rf .build DerivedData" is still waiting
