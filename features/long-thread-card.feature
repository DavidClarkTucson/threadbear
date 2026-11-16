Feature: The long thread card
  As someone whose thread has grown long
  I want to be offered a compact or a fresh thread
  So replies stay cheap and the agent keeps track of the early details

  Background:
    Given the app is connected to a runner with the demo threads

  # The mock grows a thread past the runner's contextNudge levels when it's
  # sent "mock long thread" (firm) or "mock gentle thread". The Swift app has
  # this card (LongThreadOffer) but its dump doesn't say what's over the box.
  @card.longthread @swift-pending
  Scenario: A long thread offers Compact and Continue in a new thread
    When I open the thread "Morning brief"
    And I send "mock gentle thread"
    Then the box has the long thread card saying "This conversation is long (162k tokens)."
    When I send "mock long thread"
    Then the box has the long thread card saying "This conversation is very long (262k tokens)."

  @card.longthread @swift-pending
  Scenario: Compact sends /compact and the card goes
    When I open the thread "Morning brief"
    And I send "mock long thread"
    And I press "Compact" on the long thread card
    Then the transcript shows my message "/compact"
    And the box has no long thread card

  @card.longthread @swift-pending
  Scenario: Not now hides the card until the thread is longer still
    When I open the thread "Morning brief"
    And I send "mock gentle thread"
    And I press "Not now" on the long thread card
    Then the box has no long thread card
    When I send "mock long thread"
    Then the box has the long thread card saying "This conversation is very long"

  @card.longthread @swift-pending
  Scenario: Continue in a new thread carries on in one, and Open goes there
    When I open the thread "Morning brief"
    And I send "mock long thread"
    And I press "Continue in a new thread" on the long thread card
    Then the box has the long thread card saying "Carried on in “Morning brief (continued)”."
    When I press "Open" on the long thread card
    Then the open thread is "Morning brief (continued)"
