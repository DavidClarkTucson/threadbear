Feature: The panels round the thread
  As someone with several threads going at once
  I want to fold the thread list away and keep a thread open beside another
  So the window shows what I'm working on and nothing else

  Background:
    Given the app is connected to a runner with the demo threads

  @menu.view
  Scenario: Collapse Threads folds the list away, and ⌘/ brings it back as it was
    Then the "View" menu has "Collapse Threads" on "⌘/"
    When I choose "Collapse Threads" from the menu bar
    Then the Threads panel is folded away
    And the panels are laid out as "row[threads (no title) 29% | thread 47% | highlights 24%]"
    When I press "/" with "cmd"
    Then the Threads panel is showing

  # A scripted Swift app's menu items keep their titles from the last time
  # the menu was opened, so only the GPUI app can say them as they change.
  @menu.view @swift-pending
  Scenario: The View menu's items say what they'll do once the list is folded away
    When I choose "Collapse Threads" from the menu bar
    Then the "View" menu has "Expand Threads" on "⌘/"
    And the "View" menu has "Show Threads"
    When I choose "Expand Threads" from the menu bar
    Then the Threads panel is showing
    And the "View" menu has "Hide Threads"

  @menu.view
  Scenario: ⌘/ folds the list away, and the activity bar's Threads button brings it back
    When I press "/" with "cmd"
    Then the Threads panel is folded away
    When I click "Threads" in the activity bar
    Then the Threads panel is showing

  @menu.view
  Scenario: The folded list stays folded when the app is opened again
    When I press "/" with "cmd"
    Then the Threads panel is folded away
    When the app is quit and opened again
    Then the Threads panel is folded away
