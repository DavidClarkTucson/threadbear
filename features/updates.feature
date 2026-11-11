Feature: The Updates panel
  As someone whose runner and agent CLIs keep themselves up to date
  I want one place that says what's on its way and what's waiting for me
  So I can see an update coming, and act on the few that need me

  Background:
    Given the app is connected to a runner with the demo threads

  @updates.panel
  Scenario: It says how the runner's update is going and what each CLI has
    When the runner starts building main at "abc1234" for "Fix the thing"
    And I open the Updates panel
    Then the Updates panel lists "Building the runner from main at abc1234: Fix the thing."
    And the Updates panel lists "Claude Code 2.1.311, 2.1.312 available (automatic) [Update]"
    And the Updates panel lists "Codex 0.165.0 is held back because the app-server protocol changed (automatic) [Update anyway] [Go back to 0.163.0]"
    And the Updates panel lists "  Missing: thread/backgroundTerminals/list"

  @updates.panel
  Scenario: Update installs a CLI's new version, saying so while it runs
    When I open the Updates panel
    And I press "Install the new version now" in the Updates panel
    Then the Updates panel lists "Claude Code 2.1.311, 2.1.312 available (automatic): Updating Claude Code to 2.1.312"
    And the Updates panel lists "Claude Code 2.1.312, up to date (automatic)"

  @updates.panel
  Scenario: Mike's release says what's new, and Bring Them In starts a thread for it
    When Mike has a release of 10 Nov with 3 new changes
    And I open the Updates panel
    Then the Updates panel lists "Mike's release of 10 Nov has 3 new changes."
    And the Updates panel lists "  Runners in the app"
    When I press "Start a thread that merges Mike's release into your main" in the Updates panel
    Then a thread called "Bring in Mike's release of 10 Nov" opens

  # The app's own update is made up (`updates` with `preview`): a scripted
  # app never follows main, as only the installed one does.
  @system.selfupdate
  Scenario: The app's next version waiting says so in the Updates panel
    When the app's next version, def5678, is ready
    And I open the Updates panel
    Then the Updates panel lists "The app's next version (def5678) is ready. It switches in when you're not using the app, or now."

  # The Swift app's menu bar didn't show the made-up update's item within 5 s
  # of a script making it up (its SwiftUI commands, read by `menus`), so this
  # is only checked on the GPUI app for now.
  @system.selfupdate @swift-pending
  Scenario: Restart to Update is in the app menu once the next version is ready
    When the app's next version, def5678, is ready
    Then the app menu has "Restart to Update to def5678"

  # The Swift app builds where it is, so it never waits for a Mac.
  @system.selfupdate @swift-pending
  Scenario: A new version with no Mac to build it on says it's waiting for one
    When the app's next version is waiting for a Mac to build it on
    And I open the Updates panel
    Then the Updates panel lists "The app has a new version on main, waiting for a Mac to build it on."
