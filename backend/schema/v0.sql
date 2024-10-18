-- Create app_user table
CREATE TABLE IF NOT EXISTS app_user (
  id SERIAL PRIMARY KEY,
  name VARCHAR(255) NOT NULL UNIQUE,
  pubkey VARCHAR(66) NOT NULL UNIQUE,
  last_tweak BIGINT NOT NULL,
  relays TEXT [] NOT NULL,
  federation_ids TEXT [] NOT NULL
);
-- Create invoice table
CREATE TABLE IF NOT EXISTS invoice (
  id SERIAL PRIMARY KEY,
  federation_id VARCHAR(255) NOT NULL,
  op_id VARCHAR(255) NOT NULL UNIQUE,
  app_user_id INTEGER NOT NULL REFERENCES app_user(id),
  bolt11 TEXT NOT NULL,
  amount BIGINT NOT NULL,
  state INTEGER NOT NULL,
  tweak BIGINT NOT NULL
);
-- Create index on invoice.state for faster queries on pending invoices
CREATE INDEX IF NOT EXISTS idx_invoice_state ON invoice(state);
-- Create index on invoice.app_user_id for faster joins
CREATE INDEX IF NOT EXISTS idx_invoice_app_user_id ON invoice(app_user_id);
