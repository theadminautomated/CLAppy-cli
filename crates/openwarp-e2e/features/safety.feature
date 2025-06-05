Feature: Safety
  Scenario: dangerous command
    Given a command router
    When safety scan "rm -rf /"
    Then scan result is false
