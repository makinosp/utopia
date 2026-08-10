# Unit of Work Story Map

## Mapping Rule
Each story has one primary owning unit. Secondary units are listed as dependencies when shared contracts are required.

## Story-to-Unit Mapping

| Story ID | Story Name | Primary Unit | Secondary Dependencies |
|---|---|---|---|
| US-001 | List Accounts | UOW-02 Accounts | UOW-01 Core |
| US-002 | Get Single Account | UOW-02 Accounts | UOW-01 Core |
| US-003 | Create Account | UOW-02 Accounts | UOW-01 Core |
| US-004 | Update Account | UOW-02 Accounts | UOW-01 Core |
| US-005 | Delete Account | UOW-02 Accounts | UOW-01 Core |
| US-006 | List Transactions | UOW-03 Transactions | UOW-01 Core, UOW-02 Accounts |
| US-007 | Get Single Transaction | UOW-03 Transactions | UOW-01 Core, UOW-02 Accounts |
| US-008 | Create Transaction | UOW-03 Transactions | UOW-01 Core, UOW-02 Accounts |
| US-009 | Update Transaction | UOW-03 Transactions | UOW-01 Core, UOW-02 Accounts |
| US-010 | Delete Transaction | UOW-03 Transactions | UOW-01 Core, UOW-02 Accounts |
| US-011 | List Account Transactions | UOW-03 Transactions | UOW-01 Core, UOW-02 Accounts |
| US-012 | List Budgets | UOW-04 Budgets | UOW-01 Core |
| US-013 | Get Single Budget | UOW-04 Budgets | UOW-01 Core |
| US-014 | Create Budget | UOW-04 Budgets | UOW-01 Core |
| US-015 | Update Budget | UOW-04 Budgets | UOW-01 Core |
| US-016 | Delete Budget | UOW-04 Budgets | UOW-01 Core |
| US-017 | List Budget Limits | UOW-04 Budgets | UOW-01 Core |
| US-018 | List Currencies | UOW-05 Metadata | UOW-01 Core |
| US-019 | Get System Preferences/User Profile | UOW-05 Metadata | UOW-01 Core |
| US-020 | Get Server About Info | UOW-05 Metadata | UOW-01 Core |
| US-021 | Obtain OAuth2/Personal Access Token | UOW-01 Core | None |
| US-022 | Reject Unauthenticated Requests | UOW-01 Core | UOW-02 Accounts, UOW-03 Transactions, UOW-04 Budgets, UOW-05 Metadata |

## Coverage Validation
- Total approved stories mapped: 22
- Unmapped stories: 0
- Multi-unit stories use primary ownership with explicit dependency references.
