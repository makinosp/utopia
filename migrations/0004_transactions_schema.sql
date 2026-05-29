-- Migration 0004: Transactions Schema
--
-- Prerequisites: 0001_initial_schema.sql, 0002_accounts_schema.sql, 0003_accounts_extended_schema.sql
--
-- This migration creates the transaction_journals table required by the
-- Transactions Module (UOW-03) functional design.

-- ============================================================
-- 1. Transaction Journals table
-- ============================================================
CREATE TABLE IF NOT EXISTS transaction_journals (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    group_id UUID NOT NULL,
    transaction_type TEXT NOT NULL CHECK (transaction_type IN ('withdrawal', 'deposit', 'transfer')),
    description TEXT NOT NULL,
    amount NUMERIC(20, 8) NOT NULL,
    currency_code TEXT NOT NULL,
    date TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    source_id UUID REFERENCES accounts(id) ON DELETE SET NULL,
    destination_id UUID REFERENCES accounts(id) ON DELETE SET NULL,
    category_name TEXT,
    notes TEXT,
    reconciled BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- ============================================================
-- 2. Indexes
-- ============================================================

-- Primary listing: by user with date ordering
CREATE INDEX IF NOT EXISTS idx_tj_user_id_date
    ON transaction_journals (user_id, date DESC);

-- Per-account listing: find all transactions involving a specific account as source
CREATE INDEX IF NOT EXISTS idx_tj_source_id
    ON transaction_journals (user_id, source_id, date DESC);

-- Per-account listing: find all transactions involving a specific account as destination
CREATE INDEX IF NOT EXISTS idx_tj_destination_id
    ON transaction_journals (user_id, destination_id, date DESC);

-- Group lookup
CREATE INDEX IF NOT EXISTS idx_tj_group_id
    ON transaction_journals (user_id, group_id);

-- ============================================================
-- 3. Trigger: auto-update updated_at timestamp
-- ============================================================
CREATE OR REPLACE FUNCTION update_transaction_journals_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS trg_transaction_journals_updated_at ON transaction_journals;
CREATE TRIGGER trg_transaction_journals_updated_at
    BEFORE UPDATE ON transaction_journals
    FOR EACH ROW
    EXECUTE FUNCTION update_transaction_journals_updated_at();
