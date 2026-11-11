Feature: Drafts in a reply
  As someone an agent writes messages for
  I want a Slack message or an email it drafted shown the way it'll go out
  So I can read it through and copy it straight into Slack or Mail

  Background:
    Given the app is connected to a runner with the demo threads

  @rich.draft
  Scenario: A Slack draft keeps every line where the agent put it, in Slack's formatting
    When I open the thread "Morning brief"
    And a reply with a Slack draft streams in
    Then the reply shows a "Slack draft" box with the lines "Threadbear, 6 Nov", "• one link" and "• two"
    And in the draft "Threadbear, 6 Nov" is bold and "link" links to "https://x.dev"

  @rich.draft
  Scenario: An email draft keeps its blank lines and links its addresses
    When I open the thread "Morning brief"
    And a reply with an email draft streams in
    Then the reply shows an "Email draft" box with the lines "Hi Sam,", "", "See https://x.dev/a for *this*.", "" and "Mike"
    And in the draft "https://x.dev/a" links to "https://x.dev/a"

  @rich.draft
  Scenario: A draft whose fence hasn't closed yet stays code
    When I open the thread "Morning brief"
    And a reply with an unfinished Slack draft streams in
    Then the reply shows no draft box

  @rich.draft.copy
  Scenario: Copy puts the draft on the clipboard with its bold and links, and says so
    When I open the thread "Morning brief"
    And a reply with a Slack draft streams in
    And I press the draft's Copy
    Then the draft's Copy reads "Copied"
    And the clipboard holds the lines "*Threadbear, 6 Nov*", "• one link (https://x.dev)" and "• two"
    And the clipboard's HTML has '<b>Threadbear, 6 Nov</b><br>• one <a href="https://x.dev">link</a><br>• two'
