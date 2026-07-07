// Transaction fixture definitions and generation logic.

import type { TransactionSeed } from './types';

/**
 * Deterministic transaction fixtures for compatibility testing.
 * These reference account names that will be resolved to UUIDs at seed time.
 */
export const SEED_TRANSACTIONS: TransactionSeed[] = [
  {
    group_id: '00000000-0000-0000-0000-000000000001',
    transaction_type: 'deposit',
    description: 'Monthly salary',
    amount: '350000.00',
    currency_code: 'JPY',
    date: '2026-01-15T09:00:00Z',
    source_name: 'Revenue Account',
    destination_name: 'Default Account',
    category_name: 'Salary',
    notes: 'January salary payment',
    reconciled: true,
  },
  {
    group_id: '00000000-0000-0000-0000-000000000002',
    transaction_type: 'withdrawal',
    description: 'Grocery shopping',
    amount: '8500.00',
    currency_code: 'JPY',
    date: '2026-01-16T18:30:00Z',
    source_name: 'Default Account',
    destination_name: 'Expense Account',
    category_name: 'Groceries',
    notes: 'Weekly grocery run',
    reconciled: true,
  },
  {
    group_id: '00000000-0000-0000-0000-000000000003',
    transaction_type: 'transfer',
    description: 'Monthly savings transfer',
    amount: '50000.00',
    currency_code: 'JPY',
    date: '2026-01-20T10:00:00Z',
    source_name: 'Default Account',
    destination_name: 'Savings Account',
    category_name: 'Savings',
    notes: 'Automatic monthly transfer',
    reconciled: true,
  },
  {
    group_id: '00000000-0000-0000-0000-000000000004',
    transaction_type: 'withdrawal',
    description: 'Credit card payment',
    amount: '25000.00',
    currency_code: 'JPY',
    date: '2026-01-25T12:00:00Z',
    source_name: 'Default Account',
    destination_name: 'Credit Card',
    category_name: 'Debt',
    notes: 'Credit card bill payment',
    reconciled: false,
  },
  {
    group_id: '00000000-0000-0000-0000-000000000005',
    transaction_type: 'deposit',
    description: 'Cash withdrawal',
    amount: '10000.00',
    currency_code: 'JPY',
    date: '2026-01-28T14:00:00Z',
    source_name: 'Default Account',
    destination_name: 'Cash Account',
    category_name: 'Cash',
    notes: 'ATM withdrawal',
    reconciled: true,
  },
];
