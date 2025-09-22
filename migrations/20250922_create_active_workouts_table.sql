-- migrations/20241222_create_active_workouts_table.sql

-- Tabelle für aktive (laufende) Trainings
CREATE TABLE IF NOT EXISTS active_workouts (
    id TEXT PRIMARY KEY NOT NULL,
    user_id TEXT NOT NULL,
    workout_id TEXT NOT NULL,
    started_at TEXT NOT NULL,
    -- completed_at kommt später wenn Training beendet wird
    created_at TEXT NOT NULL,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (workout_id) REFERENCES workouts(id) ON DELETE CASCADE
);

-- Tabelle für einzelne Sets die durchgeführt wurden
CREATE TABLE IF NOT EXISTS completed_sets (
    id TEXT PRIMARY KEY NOT NULL,
    active_workout_id TEXT NOT NULL,
    exercise_id TEXT NOT NULL,
    set_number INTEGER NOT NULL,        -- 1., 2., 3. Set...
    weight REAL,                        -- NULL für Bodyweight exercises
    reps INTEGER NOT NULL,              -- Tatsächliche Wiederholungen
    notes TEXT,                         -- Optional: "zu schwer", "form war schlecht" etc.
    completed_at TEXT NOT NULL,         -- Zeitstempel wann Set gemacht wurde
    created_at TEXT NOT NULL,
    FOREIGN KEY (active_workout_id) REFERENCES active_workouts(id) ON DELETE CASCADE,
    FOREIGN KEY (exercise_id) REFERENCES exercises(id) ON DELETE RESTRICT
);

-- Indexes für bessere Performance
CREATE INDEX IF NOT EXISTS idx_active_workouts_user_id ON active_workouts(user_id);
CREATE INDEX IF NOT EXISTS idx_active_workouts_started_at ON active_workouts(user_id, started_at);
CREATE INDEX IF NOT EXISTS idx_completed_sets_active_workout ON completed_sets(active_workout_id);
CREATE INDEX IF NOT EXISTS idx_completed_sets_exercise ON completed_sets(active_workout_id, exercise_id);

-- Testdaten: Ein aktives Training für den ersten User (falls vorhanden)
INSERT OR IGNORE INTO active_workouts (id, user_id, workout_id, started_at, created_at)
SELECT 
    'active-workout-test-001',
    users.id,
    workouts.id,
    datetime('now'),
    datetime('now')
FROM users 
CROSS JOIN workouts 
WHERE workouts.name = 'Push Day'
LIMIT 1;

-- Testdaten: Ein paar completed sets
INSERT OR IGNORE INTO completed_sets (id, active_workout_id, exercise_id, set_number, weight, reps, completed_at, created_at)
SELECT 
    'set-001',
    'active-workout-test-001',
    exercises.id,
    1,
    80.0,
    8,
    datetime('now', '-5 minutes'),
    datetime('now', '-5 minutes')
FROM exercises 
WHERE exercises.name = 'Bench Press'
LIMIT 1;

INSERT OR IGNORE INTO completed_sets (id, active_workout_id, exercise_id, set_number, weight, reps, notes, completed_at, created_at)
SELECT 
    'set-002',
    'active-workout-test-001',
    exercises.id,
    2,
    80.0,
    6,
    'Letztes Rep war schwer',
    datetime('now', '-2 minutes'),
    datetime('now', '-2 minutes')
FROM exercises 
WHERE exercises.name = 'Bench Press'
LIMIT 1;
