CREATE TABLE IF NOT EXISTS accounts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    account_type TEXT NOT NULL,
    name TEXT NOT NULL,
    current_balance NUMERIC(20, 8) NOT NULL DEFAULT 0,
    currency_code TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_accounts_user_id_name
    ON accounts (user_id, name, id);

CREATE INDEX IF NOT EXISTS idx_accounts_user_id_type
    ON accounts (user_id, account_type);
