Feature: Plugin Bus
  Scenario: openurl
    Given a command router
    When plugin bus processes "openurl https://example.com"
    Then plugin output should be "https://example.com"
