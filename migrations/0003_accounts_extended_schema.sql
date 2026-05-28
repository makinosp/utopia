-- Migration 0003: Accounts Extended Schema
-- 
-- Prerequisites: 0001_initial_schema.sql, 0002_accounts_schema.sql
-- 
-- This migration adds columns and indexes required by the Accounts Module
-- functional design (§1 - expanded attributes) and NFR design patterns.

-- ============================================================
-- 1. Users table: add primary currency preference
-- ============================================================
ALTER TABLE users ADD COLUMN IF NOT EXISTS primary_currency_code TEXT NOT NULL DEFAULT 'JPY';

-- ============================================================
-- 2. Accounts table: add extended attribute columns
-- ============================================================

-- Account status and ordering
ALTER TABLE accounts ADD COLUMN IF NOT EXISTS active BOOLEAN NOT NULL DEFAULT true;
ALTER TABLE accounts ADD COLUMN IF NOT EXISTS "order" INTEGER;

-- Balance-related columns (Pattern DI-01)
ALTER TABLE accounts ADD COLUMN IF NOT EXISTS initial_balance NUMERIC(20, 8) NOT NULL DEFAULT 0;
ALTER TABLE accounts ADD COLUMN IF NOT EXISTS initial_balance_date TIMESTAMPTZ;
ALTER TABLE accounts ADD COLUMN IF NOT EXISTS virtual_balance NUMERIC(20, 8) NOT NULL DEFAULT 0;

-- Soft delete (Pattern SEC-ACCT-02)
ALTER TABLE accounts ADD COLUMN IF NOT EXISTS deleted_at TIMESTAMPTZ;

-- Financial identifiers
ALTER TABLE accounts ADD COLUMN IF NOT EXISTS iban TEXT;
ALTER TABLE accounts ADD COLUMN IF NOT EXISTS bic TEXT;
ALTER TABLE accounts ADD COLUMN IF NOT EXISTS account_number TEXT;

-- Metadata
ALTER TABLE accounts ADD COLUMN IF NOT EXISTS notes TEXT;
ALTER TABLE accounts ADD COLUMN IF NOT EXISTS include_net_worth BOOLEAN NOT NULL DEFAULT true;

-- Asset account role
ALTER TABLE accounts ADD COLUMN IF NOT EXISTS account_role TEXT;

-- Liability-specific fields
ALTER TABLE accounts ADD COLUMN IF NOT EXISTS liability_type TEXT;
ALTER TABLE accounts ADD COLUMN IF NOT EXISTS liability_direction TEXT;
ALTER TABLE accounts ADD COLUMN IF NOT EXISTS interest TEXT;
ALTER TABLE accounts ADD COLUMN IF NOT EXISTS interest_period TEXT;

-- Credit card-specific fields
ALTER TABLE accounts ADD COLUMN IF NOT EXISTS cc_type TEXT;
ALTER TABLE accounts ADD COLUMN IF NOT EXISTS cc_monthly_payment_date TEXT;

-- Opening balance
ALTER TABLE accounts ADD COLUMN IF NOT EXISTS opening_balance_date TIMESTAMPTZ;

-- ============================================================
-- 3. Indexes
-- ============================================================

-- Partial unique index for active account names (Pattern DI-02)
-- Prevents duplicate active account names per user while allowing
-- soft-deleted accounts to retain their original name.
CREATE UNIQUE INDEX IF NOT EXISTS idx_accounts_user_id_name_active
    ON accounts (user_id, LOWER(name))
    WHERE deleted_at IS NULL;

-- Index for efficient soft-delete filtering
CREATE INDEX IF NOT EXISTS idx_accounts_user_id_deleted_at
    ON accounts (user_id, deleted_at);

-- Index for current_balance range queries (future balance_min/max filters)
CREATE INDEX IF NOT EXISTS idx_accounts_user_id_current_balance
    ON accounts (user_id, current_balance);

-- Index for account_role filtering
CREATE INDEX IF NOT EXISTS idx_accounts_user_id_account_role
    ON accounts (user_id, account_role)
    WHERE account_role IS NOT NULL;

-- ============================================================
-- 4. Trigger: auto-update updated_at timestamp
-- ============================================================
CREATE OR REPLACE FUNCTION update_accounts_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS trg_accounts_updated_at ON accounts;
CREATE TRIGGER trg_accounts_updated_at
    BEFORE UPDATE ON accounts
    FOR EACH ROW
    EXECUTE FUNCTION update_accounts_updated_at();
