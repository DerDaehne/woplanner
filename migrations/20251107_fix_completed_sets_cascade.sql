-- Fix completed_sets foreign key to prevent cascade deletion when active_workout is finished
-- This ensures training history is preserved for progression tracking

-- SQLite doesn't support DROP FOREIGN KEY, so we need to recreate the table

-- Step 1: Create new table with corrected foreign key
CREATE TABLE completed_sets_new (
    id TEXT PRIMARY KEY NOT NULL,
    active_workout_id TEXT NOT NULL,  -- Will reference completed_workouts after workout is finished
    exercise_id TEXT NOT NULL,
    set_number INTEGER NOT NULL,
    weight REAL,
    reps INTEGER NOT NULL,
    notes TEXT,
    completed_at TEXT NOT NULL,
    created_at TEXT NOT NULL,
    -- Remove CASCADE so sets persist when active_workout is deleted
    FOREIGN KEY (exercise_id) REFERENCES exercises(id) ON DELETE RESTRICT
);

-- Step 2: Copy existing data (if any)
INSERT INTO completed_sets_new
SELECT id, active_workout_id, exercise_id, set_number, weight, reps, notes, completed_at, created_at
FROM completed_sets;

-- Step 3: Drop old table
DROP TABLE completed_sets;

-- Step 4: Rename new table
ALTER TABLE completed_sets_new RENAME TO completed_sets;

-- Step 5: Recreate indexes
CREATE INDEX idx_completed_sets_active_workout ON completed_sets(active_workout_id);
CREATE INDEX idx_completed_sets_exercise ON completed_sets(active_workout_id, exercise_id);
CREATE INDEX idx_completed_sets_exercise_user ON completed_sets(exercise_id, completed_at);
