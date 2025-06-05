Feature: Smart Mode
  Scenario: interactive shell
    Given a command router
    When I route "python"
    Then the route should spawn "python"
