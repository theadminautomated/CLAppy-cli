Feature: Model
  Scenario: switch
    Given a command router
    When I route "/model llama2"
    Then the route should be Switch "llama2"
