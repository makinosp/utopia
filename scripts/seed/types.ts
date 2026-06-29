// Shared TypeScript interfaces for seed data structures.
// Used by both the seed generator and k6 test scripts for type consistency.

export interface AccountSeed {
  name: string;
  type: string;
  currency_code: string;
  active: boolean;
  include_net_worth: boolean;
  current_balance?: string;
  account_role?: string | null;
  iban?: string | null;
  bic?: string | null;
  account_number?: string | null;
  notes?: string | null;
  liability_type?: string | null;
  liability_direction?: string | null;
  interest?: string | null;
  interest_period?: string | null;
}

export interface TransactionSeed {
  group_id: string;
  transaction_type: "withdrawal" | "deposit" | "transfer";
  description: string;
  amount: string;
  currency_code: string;
  date: string;
  source_name?: string | null;
  destination_name?: string | null;
  category_name?: string | null;
  notes?: string | null;
  reconciled?: boolean;
}

export interface UserSeed {
  email: string;
  password: string;
  primary_currency_code: string;
}

export interface SeedConfig {
  user: UserSeed;
  accounts: AccountSeed[];
  transactions: TransactionSeed[];
}
