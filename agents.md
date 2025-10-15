# Agent Instructions for WOPlanner

## Project Overview

WOPlanner is a Progressive Web App (PWA) for tracking strength training workouts. Built with Rust, Axum, HTMX, and SQLite, it focuses on mobile-first design and simplicity.

**Primary Goal:** Learning Rust and modern web development patterns while building a useful fitness tracking tool.

**Key Philosophy:** Simple, type-safe, mobile-first, no complex JavaScript.

## Technology Stack

### Backend
- **Language:** Rust
- **Web Framework:** Axum 0.8.4
- **Database:** SQLite with SQLx
- **Sessions:** tower-sessions with SQLite backend
- **Template Engine:** Askama
- **Error Handling:** anyhow

### Frontend
- **Interactivity:** HTMX 1.9.12
- **Styling:** TailwindCSS (CDN)
- **Design System:** Glassmorphism with dark theme
- **Icons:** Emoji-based for simplicity

### Development
- **Build Tool:** Cargo
- **Dev Environment:** Nix Flakes
- **Hot Reload:** cargo-watch

## Architecture Patterns

### Project Structure
```
src/
├── main.rs              # Entry point, router composition
├── database.rs          # Connection pool & migrations
├── handlers/            # HTTP request handlers (one per domain)
└── models/              # Data structures (one per domain)

templates/               # Askama HTML templates
├── base.html           # Base layout with navigation
└── feature/            # Feature-specific templates

migrations/             # SQL migration files (chronological, schema only)
seeds/                  # Sample data for development and demos
static/                 # CSS, PWA manifest, icons
```


## Critical Implementation Rules

### Askama Template Limitations
**Problem:** Askama has very limited filter support

**Forbidden Patterns:**
```jinja2
{{ value | round }}        ❌ No round filter
{{ value | min(100) }}     ❌ No min/max
{{ list | sum }}           ❌ No aggregate filters
{% set_global var = x %}   ❌ No variable mutation
```

**Correct Patterns:**
```jinja2
{{ value as i32 }}         ✅ Type casting
{% if value > 100 %}       ✅ Inline conditionals
{% for item in list %}     ✅ Simple loops
{% match option %}         ✅ Pattern matching
```

**Rule:** Compute in Rust, display in templates. Keep template logic minimal.

### Session Management
```rust
async fn get_current_user(session: &Session, pool: &SqlitePool) -> Option<User> {
    if let Ok(Some(user_id)) = session.get::<String>("current_user_id").await {
        sqlx::query_as!(User, "SELECT * FROM users WHERE id = ?", user_id)
            .fetch_optional(pool)
            .await
            .ok()
            .flatten()
    } else {
        None
    }
}
```

### HTMX Redirects
```rust
use axum::http::{HeaderMap, HeaderValue};

let mut headers = HeaderMap::new();
headers.insert("HX-Redirect", HeaderValue::from_static("/path"));
(headers, Html("Message".to_string())).into_response()
```

## Design System

### CSS Classes (Glassmorphism)
```css
.glass              /* Subtle glass effect */
.glass-card         /* Prominent card with glass effect */
.btn-primary        /* Netflix red button */
.btn-secondary      /* Glass button */
.input-glass        /* Glass input field */
.dock               /* Fixed bottom navigation */
```

### Component Guidelines
- **Cards:** Use `.glass-card` with `rounded-2xl` or `rounded-3xl`
- **Buttons:** Minimum height 44px for touch targets
- **Icons:** Emoji-based (🏋️💪📊🔥📈) for consistency
- **Spacing:** Use Tailwind scale (4, 6, 8 for gaps)
- **Hover Effects:** Subtle `hover:scale-[1.02]` on cards

## Mobile-First PWA Considerations

### iOS Safari Quirks
```css
/* Safe area support */
padding-bottom: calc(80px + env(safe-area-inset-bottom));

/* Fixed positioning fix */
.dock {
    position: fixed;
    -webkit-transform: translate3d(0, 0, 0);
    transform: translate3d(0, 0, 0);
    -webkit-backface-visibility: hidden;
}
```

### Viewport Meta Tag
```html
<meta name="viewport" content="width=device-width, initial-scale=1.0, viewport-fit=cover, user-scalable=no">
```

### Input Font Size (Prevent Zoom)
```css
input, textarea, select {
    font-size: 16px !important; /* Prevents iOS auto-zoom */
}
```

## Database Schema

### Core Tables
```sql
users                       -- User accounts
├── workouts                -- Workout templates
│   └── workout_exercises   -- Exercises in workout
│       └── exercises       -- Exercise library
├── active_workouts         -- Currently training
│   └── completed_sets      -- Sets in active session
└── completed_workouts      -- Finished trainings
```

### Key Relationships
- One User → Many Workouts
- One Workout → Many WorkoutExercises → Many Exercises
- One User → One ActiveWorkout (at most)
- One ActiveWorkout → Many CompletedSets
- One ActiveWorkout → One CompletedWorkout (after finish)

## Common Tasks

### Adding a New Feature
1. **Database:** Create migration in `migrations/`
2. **Models:** Define structs in `src/models/feature.rs`
3. **Handler:** Create `src/handlers/feature.rs` with router
4. **Templates:** Add templates in `templates/feature/`
5. **Integration:** 
   - Add to `src/handlers/mod.rs`
   - Merge router in `src/main.rs`
   - Update navigation (dock or dashboard)
6. **Testing:** Test on mobile PWA (iOS + Android)

### Creating a Migration
```sql
-- migrations/YYYYMMDD_description.sql
-- Migrations should ONLY contain schema changes (tables, indexes, constraints)
-- NO sample data in migrations!
CREATE TABLE IF NOT EXISTS table_name (
    id TEXT PRIMARY KEY NOT NULL,
    field TEXT NOT NULL,
    created_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_name ON table(field);
```

### Creating Seeds (Sample Data)
```sql
-- seeds/XX_description.sql
-- Seeds are for development and demos only
-- Use INSERT OR IGNORE for idempotency
INSERT OR IGNORE INTO table_name (id, field, created_at) VALUES
    ('sample-id-001', 'Sample Value', '2024-12-03T10:00:00Z');
```

**Important:**
- Seeds run automatically in development (`SEED_DATABASE=true`, default)
- Disable for production: `SEED_DATABASE=false`
- Seeds must be registered in `src/database.rs`

### Writing SQLx Queries
```rust
// Fetch one
let model = sqlx::query_as!(
    Model,
    "SELECT * FROM table WHERE id = ?",
    id
).fetch_one(&pool).await?;

// Fetch optional
let model = sqlx::query_as!(
    Model,
    "SELECT * FROM table WHERE id = ?",
    id
).fetch_optional(&pool).await?;

// Fetch many
let models = sqlx::query_as!(
    Model,
    "SELECT * FROM table ORDER BY created_at DESC"
).fetch_all(&pool).await?;
```

## Best Practices

### Code Quality
- Run `cargo fmt` before committing
- Address `cargo clippy` warnings
- Use descriptive variable names
- Add comments for complex logic
- Keep functions focused and small

### Performance
- Use database indexes for frequent queries
- Add `LIMIT` to unbounded queries
- Compute aggregates in SQL when possible
- Keep templates simple (logic in Rust)

### Security
- Never trust user input
- Use SQLx parameterized queries (prevents SQL injection)
- Validate form data before processing
- Keep sessions secure (HTTPOnly, Secure in production)

### User Experience
- Mobile-first responsive design
- Touch-friendly (large targets)
- Immediate visual feedback
- Loading states for async operations
- Error messages that help users

## Resources

- **Rust Book:** https://doc.rust-lang.org/book/
- **Axum Docs:** https://docs.rs/axum/
- **SQLx Docs:** https://docs.rs/sqlx/
- **HTMX Docs:** https://htmx.org/docs/
- **Askama Docs:** https://docs.rs/askama/
- **PWA Guide:** https://web.dev/progressive-web-apps/

## Project Status

### Completed Features
- User management with sessions
- Exercise library (CRUD)
- Workout planning with scheduling
- Live training with guided flow
- Dashboard with real stats
- PWA manifest and iOS fixes

### In Progress
- Training history with progression tracking

### Planned
- Exercise progression charts
- Body measurements tracking
- Rest timer between sets
- Personal records (PRs) detection
- Workout templates
- Full offline support
