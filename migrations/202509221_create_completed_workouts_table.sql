-- Tabelle für abgeschlossene Trainings
CREATE TABLE IF NOT EXISTS completed_workouts (
    id TEXT PRIMARY KEY NOT NULL,
    user_id TEXT NOT NULL,
    workout_id TEXT NOT NULL,
    started_at TEXT NOT NULL,
    completed_at TEXT NOT NULL,
    total_duration_minutes INTEGER NOT NULL,
    total_sets INTEGER NOT NULL,
    total_volume_kg REAL NOT NULL,
    notes TEXT,
    created_at TEXT NOT NULL,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (workout_id) REFERENCES workouts(id) ON DELETE RESTRICT
);

-- Indexes für bessere Performance
CREATE INDEX IF NOT EXISTS idx_completed_workouts_user_id ON completed_workouts(user_id);
CREATE INDEX IF NOT EXISTS idx_completed_workouts_date ON completed_workouts(user_id, completed_at);
CREATE INDEX IF NOT EXISTS idx_completed_workouts_workout ON completed_workouts(workout_id);
