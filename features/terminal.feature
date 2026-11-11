Feature: The Terminal
  As someone working with an agent in a folder
  I want a real shell in that folder under the thread
  So I can look at its work and run what only I can, without leaving the app

  # A shell here always runs in a temp folder the scenario makes: the demo
  # threads' own folders are Mike's, and a shell never starts in anyone's.

  Background:
    Given the app is connected to a runner with the demo threads

  @terminal.panel
  Scenario: ⌘J brings up a shell in the thread's folder, with the keyboard
    Given the thread "Rename the Settings tabs" works in a folder of its own
    When I open the thread "Rename the Settings tabs"
    And I press ⌘J
    Then the Terminal is showing a shell in the thread's folder
    And the Terminal has the keyboard
    When I type "pwd" and press Return in the Terminal
    Then the Terminal shows the thread's folder

  @terminal.panel
  Scenario: ⌘J again puts it away, and the shell keeps running
    Given the thread "Rename the Settings tabs" works in a folder of its own
    When I open the thread "Rename the Settings tabs"
    And I press ⌘J
    And I press ⌘J
    Then the Terminal is put away
    And the thread's shell is still running

  @terminal.panel
  Scenario: What a program prints shows in the Terminal
    Given the thread "Rename the Settings tabs" works in a folder of its own
    When I open the thread "Rename the Settings tabs"
    And I press ⌘J
    And I type "printf '\033[31mred\033[0m and plain\n'" and press Return in the Terminal
    Then the Terminal shows the line "red and plain"

  @terminal.panel
  Scenario: What's scrolled off the top can be scrolled back to
    Given the thread "Rename the Settings tabs" works in a folder of its own
    When I open the thread "Rename the Settings tabs"
    And I press ⌘J
    And I type "seq 1 400" and press Return in the Terminal
    Then the Terminal shows the line "400"
    When I scroll the Terminal back 300 lines
    Then the Terminal shows earlier lines and not the line "400"

  @terminal.panel
  Scenario: Text copies out of the Terminal and pastes into it
    Given the thread "Rename the Settings tabs" works in a folder of its own
    When I open the thread "Rename the Settings tabs"
    And I press ⌘J
    And I type "echo copy-me" and press Return in the Terminal
    Then the Terminal shows the line "copy-me"
    When I select everything in the Terminal and copy it
    Then the clipboard holds "copy-me" among other lines
    When I paste "echo pasted-in" into the Terminal and press Return
    Then the Terminal shows the line "pasted-in"

  @terminal.panel
  Scenario: The shell is as wide as the panel
    Given the thread "Rename the Settings tabs" works in a folder of its own
    When I open the thread "Rename the Settings tabs"
    And I press ⌘J
    And I note how many columns the Terminal has
    And the window is made 300 points wider
    Then the Terminal has more columns than before
    When I type "tput cols" and press Return in the Terminal
    Then the Terminal shows its own number of columns

  @terminal.panel
  Scenario: A thread whose folder isn't on this Mac says where its Terminal is
    When I open the thread "Export last quarter's invoices as CSV"
    And I bring up the Terminal without starting a shell
    Then the Terminal says "This thread's folder isn't on this Mac any more, so this terminal is in your home folder."
    And the Terminal offers to start a shell

  @terminal.lifecycle
  Scenario: A shell starts only when one's asked for
    Given the thread "Rename the Settings tabs" works in a folder of its own
    When I open the thread "Rename the Settings tabs"
    And I bring up the Terminal without starting a shell
    Then the Terminal offers to start a shell
    And no shell is running
    When I start a shell from the empty Terminal
    Then the Terminal is showing a shell in the thread's folder

  @terminal.lifecycle
  Scenario: Each thread keeps its own shell while I look at others
    Given the thread "Rename the Settings tabs" works in a folder of its own
    And the thread "Tidy the imports in the uploader" works in a folder of its own
    When I open the thread "Rename the Settings tabs"
    And I press ⌘J
    And I open the thread "Tidy the imports in the uploader"
    Then the Terminal offers to start a shell
    When I open the thread "Rename the Settings tabs"
    Then the Terminal is showing a shell in the thread's folder
    And 1 shell is running

  @terminal.lifecycle
  Scenario: Archiving a thread stops its shell
    Given the thread "Rename the Settings tabs" works in a folder of its own
    When I open the thread "Rename the Settings tabs"
    And I press ⌘J
    Then the thread's shell is still running
    When I archive the open thread
    Then the shell of "Rename the Settings tabs" has stopped

  @terminal.lifecycle
  Scenario: A shell that has exited can be started again
    Given the thread "Rename the Settings tabs" works in a folder of its own
    When I open the thread "Rename the Settings tabs"
    And I press ⌘J
    And I type "exit" and press Return in the Terminal
    Then the Terminal says the shell has exited
    When I press New Shell in the Terminal
    Then the Terminal is showing a shell in the thread's folder
