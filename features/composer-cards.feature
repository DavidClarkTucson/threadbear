Feature: The open card in the composer
  As someone an agent has finished for, or is waiting on
  I want its asks and follow-ups in the box I answer in
  So answering is one move, not a hunt through the transcript

  Background:
    Given the app is connected to a runner with the demo threads

  @card.needsyou
  Scenario: A card of asks sits in the composer
    When I open the thread "Ship the beta to TestFlight"
    Then the card in the composer says "Needs you"
    And the card in the composer asks "1. Accept the new agreement in App Store Connect"
    And the card in the composer asks "2. Check who gets the beta"

  @card.answers
  Scenario: A picked answer waits, then goes with the message
    When I open the thread "Ship the beta to TestFlight"
    And I pick answer 1 of ask 1 on the card
    Then ask 1 on the card is answered "done"
    When I send my picks
    Then the transcript shows my message "1. Accept the new agreement in App Store Connect"

  @card.somethingelse @card.answers
  Scenario: Something else takes my own words for an ask
    When I open the thread "Ship the beta to TestFlight"
    And I write "Just the internal testers" as my own answer to ask 2
    Then ask 2 on the card is answered "something else: \"Just the internal testers\""
    When I send my picks
    Then the transcript shows my message "2. Check who gets the beta"

  @card.returnflow
  Scenario: Return with picks and an ask still open goes no further
    When I open the thread "Ship the beta to TestFlight"
    And I pick answer 1 of ask 1 on the card
    And I press Return
    Then the card in the composer asks "1. Accept the new agreement in App Store Connect"
    And the transcript doesn't show my message "1. Accept the new agreement in App Store Connect"

  @card.returnflow
  Scenario: Return with every ask answered sends them
    When I open the thread "Ship the beta to TestFlight"
    And I pick answer 1 of ask 1 on the card
    And I write "Just the internal testers" as my own answer to ask 2
    And I press Return
    Then the transcript shows my message "1. Accept the new agreement in App Store Connect"

  @card.collapse
  Scenario: The card can fold to its heading
    When I open the thread "Ship the beta to TestFlight"
    And I press "collapse" on the card
    Then the card in the composer says "Needs you (collapsed)"

  @card.done
  Scenario: A done card offers the agent's follow-ups
    When I open the thread "Tidy the imports in the uploader"
    Then the card in the composer says "Done"
    And the card offers the follow-up "Turn on the lint rule that sorts imports"

  @card.done @card.answers
  Scenario: Ticked follow-ups go when I send
    When I open the thread "Tidy the imports in the uploader"
    And I tick follow-up 1 on the card
    Then the follow-up "Turn on the lint rule that sorts imports" is ticked
    When I press Return
    Then the transcript shows my message "Turn on the lint rule that sorts imports"

  @card.review @swift-pending
  Scenario: Another lab's findings show in the transcript, not on the done card
    When I open the thread "Tidy the imports in the uploader"
    And the agent there finishes a turn another lab reviewed
    Then the card in the composer says "Done"
    And the transcript's review line says "Reviewed by Codex: 1 open"
    And the transcript's review line says "#1 fixed: Drops the session on refresh."
    And the card doesn't show "Reviewed by"

  @card.collapse
  Scenario: Folded, the card's picks show as chips over the box, and one can be taken back
    When I open the thread "Ship the beta to TestFlight"
    And I pick answer 1 of ask 1 on the card
    And I press "collapse" on the card
    Then the box shows the picked answers "1: done"
    When I take the picked answer back
    Then the box shows no picked answers
    And the transcript doesn't show my message "1. Accept the new agreement in App Store Connect"

  @card.somethingelse
  Scenario: A picture put in a Something else box goes with that answer
    When I open the thread "Ship the beta to TestFlight"
    And I write "Like this one" as my own answer to ask 2
    And I put the picture in my own answer to ask 2
    Then ask 2 on the card is answered with 1 picture
    When I send my picks
    Then the transcript shows my message "2. Check who gets the beta" with the chip "picture: Picture 1"

  # Through the answer's own button, which moves a paged card on to the
  # next ask, so Return finds the one still open already in front.
  @card.returnflow
  Scenario: Return beeps when the ask still open is the one in front
    When I open the thread "Ship the beta to TestFlight"
    And I press answer 1 of ask 1 on the card
    And the card has had a moment to move on
    And I press Return
    Then the app beeps
