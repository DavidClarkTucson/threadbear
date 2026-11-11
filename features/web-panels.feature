Feature: Web pages in a thread's panels
  As someone whose project has a dev server or a dashboard
  I want its page beside the thread, kept as I left it, with its links to
  other sites opening outside it
  So I can look at it without leaving the thread

  # The pages are little sites the steps serve on this machine
  # (support/web-pages.ts). The Swift app's web views can't be clicked into
  # from a script, and its dump doesn't read a page's title or the Convex
  # panel's state yet, so these run on the GPUI app (@swift-pending). The
  # GPUI app needs its web runtime for them: @web-runtime scenarios are
  # skipped without TB_WEB_RUNTIME_ARCHIVE (support/web-runtime.ts).

  Background:
    Given the app is connected to a runner with the demo threads

  @panel.web @web-runtime @swift-pending
  Scenario: A url panel shows its page, and keeps it between visits
    Given the web runtime is installed
    When I open the thread "Add dark mode to the dashboard"
    And the thread's url panel shows the page server
    And I show the "mock.page" section of the pane
    Then the panel shows the page "Panel home"
    When I show the "threadbear.highlights" section of the pane
    Then the panel's page is out of sight and its sound is off
    When I show the "mock.page" section of the pane
    Then the panel shows the page "Panel home" without loading it again

  @panel.web @panel.bar @web-runtime @swift-pending
  Scenario: A link to another site opens in the browser, and the panel keeps its page
    Given the web runtime is installed
    When I open the thread "Add dark mode to the dashboard"
    And the thread's url panel shows the page server
    And I show the "mock.page" section of the pane
    Then the panel shows the page "Panel home"
    And the panel's bar has no "Back"
    When I click the panel page's link "#other"
    Then the link to the other site opened in the browser
    And the panel shows the page "Panel home"
    When I click the panel page's link "#next"
    Then the panel shows the page "Second page"
    And the panel's bar has "Back"
    When I press the control "Back"
    Then the panel shows the page "Panel home"

  @panel.web @web-runtime @swift-pending
  Scenario: A new window on the panel's own site opens in the panel, and one to another site outside it
    Given the web runtime is installed
    When I open the thread "Add dark mode to the dashboard"
    And the thread's url panel shows the page server
    And I show the "mock.page" section of the pane
    Then the panel shows the page "Panel home"
    When I click the panel page's link "#other-window"
    Then the link to the other site opened in the browser
    When I click the panel page's link "#next-window"
    Then the panel shows the page "Second page"

  # Only a person's click sends a page to the browser or opens a new window,
  # and at most one a second (Codex review, 11 Nov: a script could open a
  # browser tab a second).
  @panel.web @web-runtime @swift-pending
  Scenario: A page's own scripts can't open windows, in the browser or the panel
    Given the web runtime is installed
    When I open the thread "Add dark mode to the dashboard"
    And the thread's url panel shows the page server
    And I show the "mock.page" section of the pane
    Then the panel shows the page "Panel home"
    When the panel's page opens a window to the other site by itself
    And the panel's page opens a window on its own site by itself
    Then nothing was opened in the browser
    And the panel shows the page "Panel home"

  @panel.web @web-runtime @swift-pending
  Scenario: Two clicks in a moment on a link to another site open it once
    Given the web runtime is installed
    When I open the thread "Add dark mode to the dashboard"
    And the thread's url panel shows the page server
    And I show the "mock.page" section of the pane
    Then the panel shows the page "Panel home"
    When I click the panel page's link "#other-window" twice in a moment
    Then the link to the other site opened in the browser once

  # Only web addresses load in a panel: a file from this Mac or a page made
  # of data doesn't, by a link in a new window or dropped from the Finder.
  @panel.web @web-runtime @swift-pending
  Scenario: A file, or a page made of data, never loads in a panel
    Given the web runtime is installed
    When I open the thread "Add dark mode to the dashboard"
    And the thread's url panel shows the page server
    And I show the "mock.page" section of the pane
    Then the panel shows the page "Panel home"
    When I click the panel page's link "#file-window"
    And I click the panel page's link "#data-window"
    And I drop a file from the Finder on the panel's page
    Then nothing was opened in the browser
    And the panel shows the page "Panel home"

  @panel.convex @web-runtime @swift-pending
  Scenario: The dashboard's own scripts can't take it, or the browser, anywhere
    Given the web runtime is installed
    When I open the thread "Add dark mode to the dashboard"
    And the thread has the Convex panel, with a deploy key
    And I show the "convex.dashboard" section of the pane
    Then the panel shows the page "Credentials for happy-otter-123 with a key"
    When the panel's page sends itself to the other site by itself
    And the panel's page opens a window to the other site by itself
    Then nothing was opened in the browser
    And the panel shows the page "Credentials for happy-otter-123 with a key"

  # The app's own key bindings see ⌘A first; with the page holding the
  # keyboard it's the page's Select All, as the Swift app's Edit menu sends it.
  @panel.web @web-runtime @swift-pending
  Scenario: ⌘A in a page that has the keyboard selects the page's own text
    Given the web runtime is installed
    When I open the thread "Add dark mode to the dashboard"
    And the thread's url panel shows the page server
    And I show the "mock.page" section of the pane
    Then the panel shows the page "Panel home"
    When the panel's page has the keyboard
    And I press ⌘A
    Then the page's own text is selected

  @panel.bar @web-runtime @swift-pending
  Scenario: Reload goes back to the panel's own page, and Open in browser opens it
    Given the web runtime is installed
    When I open the thread "Add dark mode to the dashboard"
    And the thread's url panel shows the page server
    And I show the "mock.page" section of the pane
    Then the panel shows the page "Panel home"
    When I click the panel page's link "#next"
    Then the panel shows the page "Second page"
    When I press the control "Reload the panel's own page"
    Then the panel shows the page "Panel home"
    When I press the control "Open in browser"
    Then the page server's page opened in the browser

  # The Swift app has WebKit, so it never asks.
  @panel.web @web-prompt @swift-pending
  Scenario: Without the web runtime a url panel asks for it, and still opens in the browser
    When I open the thread "Add dark mode to the dashboard"
    And the thread's url panel shows the page server
    And I show the "mock.page" section of the pane
    Then the panel asks to install the web runtime
    When I say Not now to the web runtime
    Then the panel says "This page shows here once the web runtime is installed (Settings > General)."
    When I press the control "Open in browser"
    Then the page server's page opened in the browser

  @panel.convex @web-runtime @swift-pending
  Scenario: The Convex panel says what the runner couldn't do
    Given the web runtime is installed
    When I open the thread "Add dark mode to the dashboard"
    And the thread has the Convex panel
    And I show the "convex.dashboard" section of the pane
    Then the Convex panel is for the project "walkies", showing "Development · happy-otter-123"
    And the Convex panel's menu offers "Development · happy-otter-123 | Production · busy-bear-456"
    And the Convex panel says what the runner said: "The demo runner has no Convex deploy keys."

  @panel.convex @panel.bar @web-runtime @swift-pending
  Scenario: The dashboard gets its deployment's key, a new one when asked, and its links open outside it
    Given the web runtime is installed
    When I open the thread "Add dark mode to the dashboard"
    And the thread has the Convex panel, with a deploy key
    And I show the "convex.dashboard" section of the pane
    Then the panel shows the page "Credentials for happy-otter-123 with a key"
    When I choose the deployment "busy-bear-456" in the Convex panel
    Then the panel shows the page "Credentials for busy-bear-456 with a key"
    When I press the control "Make a new deploy key for busy-bear-456, if the old one was revoked"
    Then the runner was asked for a new deploy key for "busy-bear-456"
    And the panel shows the page "Credentials for busy-bear-456 with a key"
    When I click the panel page's link "#docs"
    Then Convex's docs opened in the browser, not in the panel
    And the panel shows the page "Credentials for busy-bear-456 with a key"
    When I press the control "Open busy-bear-456 in browser"
    Then "https://dashboard.convex.dev/d/busy-bear-456" opened in the browser

  @panel.convex @web-runtime @swift-pending
  Scenario: Signed out of Convex, the panel signs the runner in and then shows the dashboard
    Given the web runtime is installed
    When I open the thread "Add dark mode to the dashboard"
    And the thread has the Convex panel, signed out of Convex
    And I show the "convex.dashboard" section of the pane
    Then the Convex panel offers to sign in to Convex
    When I press the control "Sign the runner's Convex CLI in, in your browser"
    Then Convex's sign-in page opened in the browser
    And the Convex panel waits for the sign-in with the code "MOCK-1234"
    And the panel shows the page "Credentials for happy-otter-123 with a key"
