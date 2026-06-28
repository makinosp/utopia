/**
 * Seed data generator for Utopia compatibility verification suite.
 *
 * Truncates all tables, then inserts deterministic test fixtures for
 * accounts, transactions, and a test user. Uses the `pg` library for
 * PostgreSQL connectivity.
 *
 * Usage: bun run scripts/seed/index.ts
 * Environment: DATABASE_URL (e.g., postgres://utopia:utopia@localhost:5432/utopia)
 */

import "dotenv/config";
import { Client } from "pg";
import { SEED_ACCOUNTS } from "./accounts";
import { SEED_TRANSACTIONS } from "./transactions";
import type { AccountSeed, TransactionSeed, UserSeed } from "./types";

const BOOTSTRAP_KEY = process.env.BOOTSTRAP_KEY || "replace-me-with-long-random-bootstrap-secret";

const SEED_USER: UserSeed = {
  email: "test-compat@utopia.local",
  password: "CompatTestPassword2026!",
  primary_currency_code: "JPY",
};

async function seed(): Promise<void> {
  const databaseUrl = process.env.DATABASE_URL;
  if (!databaseUrl) {
    console.error("ERROR: DATABASE_URL environment variable is required");
    process.exit(1);
  }

  const client = new Client({ connectionString: databaseUrl });

  try {
    await client.connect();
    console.log("Connected to database");

    // Begin transaction
    await client.query("BEGIN");

    // Step 1: Truncate all tables (CASCADE handles FK constraints)
    console.log("Truncating all tables...");
    await client.query("TRUNCATE TABLE transaction_journals CASCADE");
    await client.query("TRUNCATE TABLE accounts CASCADE");
    await client.query("TRUNCATE TABLE personal_access_tokens CASCADE");
    await client.query("TRUNCATE TABLE bootstrap_key_usage CASCADE");
    await client.query("TRUNCATE TABLE users CASCADE");

    // Step 2: Insert test user
    console.log(`Inserting test user: ${SEED_USER.email}`);
    const userResult = await client.query(
      `INSERT INTO users (email, blocked, primary_currency_code)
       VALUES ($1, $2, $3)
       RETURNING id`,
      [SEED_USER.email, false, SEED_USER.primary_currency_code],
    );
    const userId: string = userResult.rows[0].id;
    console.log(`  User ID: ${userId}`);

    // Step 3: Insert accounts and build name-to-id map
    console.log(`Inserting ${SEED_ACCOUNTS.length} accounts...`);
    const accountNameToId = new Map<string, string>();
    for (const account of SEED_ACCOUNTS) {
      const result = await client.query(
        `INSERT INTO accounts (
          user_id, account_type, name, current_balance, currency_code,
          active, include_net_worth, account_role, iban, bic, account_number,
          notes, liability_type, liability_direction, interest, interest_period
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16)
        RETURNING id`,
        [
          userId,
          account.type,
          account.name,
          account.current_balance ?? "0.00",
          account.currency_code,
          account.active,
          account.include_net_worth,
          account.account_role ?? null,
          account.iban ?? null,
          account.bic ?? null,
          account.account_number ?? null,
          account.notes ?? null,
          account.liability_type ?? null,
          account.liability_direction ?? null,
          account.interest ?? null,
          account.interest_period ?? null,
        ],
      );
      accountNameToId.set(account.name, result.rows[0].id);
      console.log(`  Account: ${account.name} -> ${result.rows[0].id}`);
    }

    // Step 4: Insert transactions (resolve account names to IDs)
    console.log(`Inserting ${SEED_TRANSACTIONS.length} transactions...`);
    for (const tx of SEED_TRANSACTIONS) {
      const sourceId = tx.source_name ? accountNameToId.get(tx.source_name) ?? null : null;
      const destinationId = tx.destination_name
        ? accountNameToId.get(tx.destination_name) ?? null
        : null;

      await client.query(
        `INSERT INTO transaction_journals (
          user_id, group_id, transaction_type, description, amount,
          currency_code, date, source_id, destination_id, category_name,
          notes, reconciled
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)`,
        [
          userId,
          tx.group_id,
          tx.transaction_type,
          tx.description,
          tx.amount,
          tx.currency_code,
          tx.date,
          sourceId,
          destinationId,
          tx.category_name ?? null,
          tx.notes ?? null,
          tx.reconciled ?? false,
        ],
      );
      console.log(`  Transaction: ${tx.description} (${tx.amount} ${tx.currency_code})`);
    }

    // Commit transaction
    await client.query("COMMIT");

    console.log("\n=== Seed Summary ===");
    console.log(`  User: 1 (${SEED_USER.email})`);
    console.log(`  Accounts: ${SEED_ACCOUNTS.length}`);
    console.log(`  Transactions: ${SEED_TRANSACTIONS.length}`);
    console.log("Seed complete.");
  } catch (err) {
    await client.query("ROLLBACK");
    console.error("Seed failed:", err);
    process.exit(1);
  } finally {
    await client.end();
  }
}

seed();
