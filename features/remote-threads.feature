Feature: Other runners' threads
  As someone with threads running on other machines
  I want them listed beside this Mac's, and to work with them here
  So it doesn't matter where a thread runs

  Background:
    Given the app is connected to a runner with the demo threads

  @sidebar.row.remote @remote-threads
  Scenario: Another runner's threads are listed with its machine
    Then the row "Check the photo library backup" is on the runner "On bruce"
    And the row "Check the photo library backup" has the machine tile
    And the row "Morning brief" isn't on another runner

  @sidebar.row.remote @remote-threads
  Scenario: Opening one shows its transcript
    When I open the thread "Tidy the media drive"
    Then the transcript shows my message "Find duplicate videos on the media drive."
    And the transcript shows a reply starting "Found 14 duplicates"
    And the composer is on the runner "bruce"

  @sidebar.row.remote @remote-threads
  Scenario: It takes a message like any other thread
    When I open the thread "Tidy the media drive"
    And I send "Keep the copies in Inbox for now"
    Then the transcript shows my message "Keep the copies in Inbox for now"

  @composer.runner @remote-threads
  Scenario: The new-thread page offers each machine, with the default starred
    When I start a new thread
    Then the "runner" menu has a row "[x] This Mac *"
    And the "runner" menu has a row "[ ] bruce"

  @composer.runner @remote-threads
  Scenario: Picking a machine is where the thread will start
    When I start a new thread
    And I pick "bruce" from the "runner" menu
    Then the composer is on the runner "bruce"
    And the note under the box says "Starts in misc on bruce, with mike"
    And the "runner" menu has a row "[x] bruce"

  @composer.runner @remote-threads
  Scenario: A new thread started on another machine is listed as its
    When I start a new thread
    And I pick "bruce" from the "runner" menu
    And I send "Check the backup drive's health"
    Then a thread called "Check the backup drive's health" opens
    And the transcript shows my message "Check the backup drive's health"
    And the row "Check the backup drive's health" is on the runner "On bruce"

  @composer.runner @remote-threads
  Scenario: The star sets where new threads start
    When I start a new thread
    And I make "bruce" the default runner
    Then the "runner" menu has a row "[x] bruce *"
    And the composer is on the runner "bruce"
    When I make "This Mac" the default runner
    Then the "runner" menu has a row "[x] This Mac *"
