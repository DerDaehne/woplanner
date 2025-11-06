-- Table for tracking personal records per exercise
CREATE TABLE IF NOT EXISTS personal_records (
    id TEXT PRIMARY KEY NOT NULL,
    user_id TEXT NOT NULL,
    exercise_id TEXT NOT NULL,
    record_type TEXT NOT NULL,  -- 'max_weight', 'max_reps', 'max_volume'
    weight REAL,                -- Weight used for the PR
    reps INTEGER,               -- Reps achieved for the PR
    volume_kg REAL,             -- Total volume (weight * reps)
    completed_set_id TEXT NOT NULL, -- Reference to the set that achieved this PR
    achieved_at TEXT NOT NULL,  -- When this PR was achieved
    created_at TEXT NOT NULL,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (exercise_id) REFERENCES exercises(id) ON DELETE CASCADE,
    FOREIGN KEY (completed_set_id) REFERENCES completed_sets(id) ON DELETE CASCADE,
    UNIQUE(user_id, exercise_id, record_type)  -- One PR per type per exercise per user
);

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_personal_records_user_exercise ON personal_records(user_id, exercise_id);
CREATE INDEX IF NOT EXISTS idx_personal_records_achieved_at ON personal_records(user_id, achieved_at);
