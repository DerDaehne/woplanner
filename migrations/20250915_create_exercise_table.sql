-- Create exercises table
CREATE TABLE IF NOT EXISTS exercises (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    instructions TEXT NOT NULL,
    created_at TEXT NOT NULL
);
