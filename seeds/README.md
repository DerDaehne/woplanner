# Database Seeds

This directory contains sample data for development and demos.

## How it works

Seeds are automatically loaded when the application starts, unless explicitly disabled.

## Usage

### Development (default)
```bash
cargo run
# Seeds are loaded automatically
```

### Production (disable seeds)
```bash
SEED_DATABASE=false cargo run
# or
export SEED_DATABASE=false
./woplanner
```

## Files

Seeds are executed in alphabetical order:

1. **01_users.sql** - Demo user accounts
2. **02_exercises.sql** - Basic exercises (Bench Press, Squat, Deadlift, Pull-up)
3. **03_workouts.sql** - Sample workout plans (Push/Pull/Legs split)

## Adding new seeds

1. Create a new file: `seeds/04_your_seed.sql`
2. Add SQL with `INSERT OR IGNORE` for idempotency
3. Register in `src/database.rs` in the `seed_files` vector

## Notes

- All seeds use `INSERT OR IGNORE` to prevent duplicates
- Seeds are safe to run multiple times
- Seeds depend on migrations being run first
- Foreign key constraints are enforced
