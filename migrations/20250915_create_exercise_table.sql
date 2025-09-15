-- Create exercises table

CREATE TABLE IF NOT EXISTS exercises (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    instructions TEXT NOT NULL,
    created_at TEXT NOT NULL
);

-- Insert some basic exercises for testing
INSERT INTO exercises (id, name, instructions, created_at) VALUES 
    ('ex-bench-press-001', 'Bench Press', 'Lie on bench, lower bar to chest, press up explosively. Keep feet planted and core tight.', '2024-12-03T10:00:00Z'),
    ('ex-squat-001', 'Squat', 'Stand with feet shoulder-width apart. Lower until thighs parallel to floor, drive through heels to stand.', '2024-12-03T10:15:00Z'),
    ('ex-deadlift-001', 'Deadlift', 'Stand with bar over mid-foot. Hinge at hips, grab bar, drive through heels to lift. Keep back straight.', '2024-12-03T10:30:00Z'),
    ('ex-pullup-001', 'Pull-up', 'Hang from bar with overhand grip. Pull body up until chin clears bar. Lower with control.', '2024-12-03T10:45:00Z');
