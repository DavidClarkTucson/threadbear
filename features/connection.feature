Feature: Staying connected to the runner
  As someone whose runner restarts when it updates
  I want the app to reconnect by itself
  So I never have to do anything about it

  Background:
    Given the app is connected to a runner with the demo threads

  @foundation.rpc
  Scenario: The app reconnects when the runner restarts
    When I open the thread "Morning brief"
    And the runner restarts
    Then the app is connected again
    And the transcript still shows the thread "Morning brief"

  @banner.connection
  Scenario: The bar says when the runner can't be reached, and goes once it's back
    When the runner goes away without a word for 8 seconds
    Then the bar says "Can't reach the runner."
    And the app is connected again
    And there's no banner once the runner is back

  @thread.loadfailure
  Scenario: A thread that can't be read says so, and Try again reads it
    Given the thread "Morning brief" can't be read once
    When I open the thread "Morning brief", which fails to load
    Then the transcript says it couldn't be loaded
    When I press Try again on the transcript
    Then the transcript shows the thread "Morning brief"
