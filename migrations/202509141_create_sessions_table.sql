-- create sessions table for tower-sessions

CREATE TABLE IF NOT EXISTS sessions (
  id TEXT PRIMARY KEY NOT NULL,
  data BLOB NOT NULL,
  expiry_data INTEGER NOT NULL
);
