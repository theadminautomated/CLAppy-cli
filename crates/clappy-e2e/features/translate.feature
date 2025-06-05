Feature: Translate
  Scenario: simple
    Given a command router
    When I translate "list files"
    Then the command should be "echo list files"
