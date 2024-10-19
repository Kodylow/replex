-- Create users table
CREATE TABLE IF NOT EXISTS users (
  id SERIAL PRIMARY KEY,
  name VARCHAR(255) NOT NULL UNIQUE,
  pubkey VARCHAR(66) NOT NULL UNIQUE,
  last_tweak BIGINT NOT NULL,
  relays TEXT [] NOT NULL,
  federation_ids TEXT [] NOT NULL
);
-- Create invoices table
CREATE TABLE IF NOT EXISTS invoices (
  id SERIAL PRIMARY KEY,
  federation_id VARCHAR(255) NOT NULL,
  op_id VARCHAR(255) NOT NULL UNIQUE,
  user_id INTEGER NOT NULL REFERENCES users(id),
  bolt11 TEXT NOT NULL,
  amount BIGINT NOT NULL,
  state INTEGER NOT NULL,
  tweak BIGINT NOT NULL
);
-- Create index on invoices.state for faster queries on pending invoices
CREATE INDEX IF NOT EXISTS idx_invoice_state ON invoices(state);
-- Create index on invoices.user_id for faster joins
CREATE INDEX IF NOT EXISTS idx_invoice_user_id ON invoices(user_id);
