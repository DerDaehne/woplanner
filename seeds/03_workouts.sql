-- Sample workouts for development and demos
INSERT OR IGNORE INTO workouts (id, user_id, name, description, is_active, schedule_type, schedule_day, created_at, updated_at)
VALUES
    ('wo-push-001', 'user-demo-001', 'Push Day', 'Brust, Schultern, Trizeps', TRUE, 'rotation', NULL, '2024-12-03T10:00:00Z', '2024-12-03T10:00:00Z'),
    ('wo-pull-001', 'user-demo-001', 'Pull Day', 'Rücken, Bizeps', TRUE, 'rotation', NULL, '2024-12-03T10:00:00Z', '2024-12-03T10:00:00Z'),
    ('wo-legs-001', 'user-demo-001', 'Leg Day', 'Beine und Core - Samstags', TRUE, 'weekly', 6, '2024-12-03T10:00:00Z', '2024-12-03T10:00:00Z');

INSERT OR IGNORE INTO workout_exercises (id, workout_id, exercise_id, position, target_sets, target_weight, notes, created_at)
VALUES
    ('we-001', 'wo-push-001', 'ex-bench-press-001', 1, 4, 80.0,'Aufwärmen mit 60kg', '2024-12-03T10:00:00Z'),
    ('we-002', 'wo-push-001', 'ex-bench-press-001', 2, 3, 60.0,'Incline Bench', '2024-12-03T10:00:00Z'),
    ('we-003', 'wo-pull-001', 'ex-pullup-001', 1, 4, NULL, 'Bis Muskelversagen', '2024-12-03T10:00:00Z'),
    ('we-004', 'wo-pull-001', 'ex-deadlift-001', 2, 3, 120.0, 'Kreuzheben schwer', '2024-12-03T10:00:00Z'),
    ('we-005', 'wo-legs-001', 'ex-squat-001', 1, 4, 100.0, 'ATG - tief runter!', '2024-12-03T10:00:00Z'),
    ('we-006', 'wo-legs-001', 'ex-deadlift-001', 2, 3, 140.0, 'Romanian Deadlifts', '2024-12-03T10:00:00Z');
