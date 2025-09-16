-- Create workouts table 
CREATE TABLE IF NOT EXISTS workouts (
    id TEXT PRIMARY KEY NOT NULL,
    user_id TEXT NOT NULL,
    name TEXT NOT NULL,
    description TEXT,
    is_active BOOLEAN DEFAULT TRUE,
    schedule_type TEXT DEFAULT 'manual',
    schedule_day INTEGER,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

-- Create workout_exercises table
CREATE TABLE IF NOT EXISTS workout_exercises (
    id TEXT PRIMARY KEY NOT NULL,
    workout_id TEXT NOT NULL,
    exercise_id TEXT NOT NULL,
    position INTEGER NOT NULL,
    target_sets INTEGER NOT NULL DEFAULT 3,
    target_weight REAL,
    notes TEXT,  
    created_at TEXT NOT NULL,
    FOREIGN KEY (workout_id) REFERENCES workouts(id) ON DELETE CASCADE,
    FOREIGN KEY (exercise_id) REFERENCES exercises(id) ON DELETE CASCADE
);

-- index creation
CREATE INDEX idx_workouts_user_id ON workouts(user_id);
CREATE INDEX idx_workouts_active ON workouts(user_id, is_active);
CREATE INDEX idx_workout_exercises_workout_id ON workout_exercises(workout_id);
CREATE INDEX idx_workout_exercises_position ON workout_exercises(workout_id, position);

-- sample data
INSERT INTO workouts (id, user_id, name, description, is_active, schedule_type, created_at, updated_at) 
SELECT 
    'wo-push-001',
    id,
    'Push Day',
    'Brust, Schultern, Trizeps',
    TRUE,
    'rotation',
    '2024-12-03T10:00:00Z',
    '2024-12-03T10:00:00Z'
FROM users LIMIT 1;

INSERT INTO workouts (id, user_id, name, description, is_active, schedule_type, created_at, updated_at) 
SELECT 
    'wo-pull-001',
    id,
    'Pull Day', 
    'Rücken, Bizeps',
    TRUE,
    'rotation',
    '2024-12-03T10:00:00Z',
    '2024-12-03T10:00:00Z'
FROM users LIMIT 1;

INSERT INTO workouts (id, user_id, name, description, is_active, schedule_type, schedule_day, created_at, updated_at) 
SELECT 
    'wo-legs-001',
    id,
    'Leg Day',
    'Beine und Core - Samstags',
    TRUE,
    'weekly',
    6,  -- Samstag
    '2024-12-03T10:00:00Z',
    '2024-12-03T10:00:00Z'
FROM users LIMIT 1;

INSERT INTO workout_exercises (id, workout_id, exercise_id, position, target_sets, target_weight, notes, created_at)
VALUES 
    ('we-001', 'wo-push-001', 'ex-bench-press-001', 1, 4, 80.0,'Aufwärmen mit 60kg', '2024-12-03T10:00:00Z'),
    ('we-002', 'wo-push-001', 'ex-bench-press-001', 2, 3, 60.0,'Incline Bench', '2024-12-03T10:00:00Z');

INSERT INTO workout_exercises (id, workout_id, exercise_id, position, target_sets, target_weight, notes, created_at)
VALUES 
    ('we-003', 'wo-pull-001', 'ex-pullup-001', 1, 4, NULL, 'Bis Muskelversagen', '2024-12-03T10:00:00Z'),
    ('we-004', 'wo-pull-001', 'ex-deadlift-001', 2, 3, 120.0, 'Kreuzheben schwer', '2024-12-03T10:00:00Z');

INSERT INTO workout_exercises (id, workout_id, exercise_id, position, target_sets, target_weight, notes, created_at)
VALUES 
    ('we-005', 'wo-legs-001', 'ex-squat-001', 1, 4, 100.0, 'ATG - tief runter!', '2024-12-03T10:00:00Z'),
    ('we-006', 'wo-legs-001', 'ex-deadlift-001', 2, 3, 140.0, 'Romanian Deadlifts', '2024-12-03T10:00:00Z');
