Feature: Design reviews
  As someone an agent shows design options to (its show_options)
  I want them in a window of their own, big enough to compare
  So I can step through them, tick one or several, say why and send it

  # The demo thread's review has four options, each one picture: A, B and C to
  # pick from, C the agent's pick, and D for reference only.
  Background:
    Given the app is connected to a runner with the demo threads
    When I open the thread "Compare account picker designs"

  @review.window @transcript.review
  Scenario: The card opens the review on its first option, one at a time
    When I press "Compare 4 images"
    Then the review window is open on option "A", one at a time

  @header.pendingreviews
  Scenario: The bar over the thread says how many reviews are waiting
    Then the bar over the thread says "Review design options (1)"

  # The Swift bar's list is a popover, a window of its own that the Swift
  # app's `click` doesn't look in.
  @header.pendingreviews @swift-pending
  Scenario: The bar's list opens a review that's waiting
    When I press the bar's "Review design options (1)"
    And I press the review "Account picker" in the bar's list
    Then the review window is open on option "A", one at a time

  @review.picks
  Scenario: Ticking options says what Send will send
    When I open the review
    And I tick "B" in the review
    And I tick "C" in the review
    Then the review's Send says "Send B + C"
    When I choose None in the review
    Then the review's Send says "Send none of them"
    And no option is ticked in the review

  @review.picks
  Scenario: An option there for reference can't be ticked
    When I open the review
    Then "D" has no tick in the review

  @review.picks
  Scenario: Closing the window keeps the draft
    When I open the review
    And I tick "B" in the review
    And I write "Bigger, please" in the review
    And I close the review
    And I open the review
    Then the review has "B" ticked and the reply "Bigger, please"

  @review.picks
  Scenario: Sending answers the agent, and the review can't be changed after
    When I open the review
    And I tick "C" in the review
    And I write "C, please." in the review
    And I press Send in the review
    Then the review window has closed
    And the transcript shows my message "Review: Account picker"
    When I press "Open review"
    Then the review window shows what was sent: "C" and "C, please."
    And nothing can be changed in the review
