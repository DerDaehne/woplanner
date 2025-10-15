use anyhow::Result;
use sqlx::{SqlitePool, migrate::MigrateDatabase};

pub async fn create_database_pool(database_url: &str) -> Result<SqlitePool> {
    if !sqlx::Sqlite::database_exists(database_url)
        .await
        .unwrap_or(false)
    {
        println!("Creating database {}", database_url);
        match sqlx::Sqlite::create_database(database_url).await {
            Ok(_) => println!("Created db successfully"),
            Err(error) => panic!("error {}", error),
        }
    } else {
        println!("Database already created");
    }

    let pool = SqlitePool::connect(database_url).await?;

    println!("Running database migrations...");
    sqlx::migrate!("./migrations").run(&pool).await?;

    // Run seeds if SEED_DATABASE=true (default in development)
    let should_seed = std::env::var("SEED_DATABASE")
        .unwrap_or_else(|_| "true".to_string())
        .parse::<bool>()
        .unwrap_or(true);

    if should_seed {
        println!("Running database seeds...");
        run_seeds(&pool).await?;
    }

    println!("Database setup complete!");
    Ok(pool)
}

async fn run_seeds(pool: &SqlitePool) -> Result<()> {
    let seed_files = vec![
        ("seeds/01_users.sql", include_str!("../seeds/01_users.sql")),
        ("seeds/02_exercises.sql", include_str!("../seeds/02_exercises.sql")),
        ("seeds/03_workouts.sql", include_str!("../seeds/03_workouts.sql")),
    ];

    for (name, sql) in seed_files {
        match sqlx::query(sql).execute(pool).await {
            Ok(_) => println!("  ✓ Seeded {}", name),
            Err(e) => eprintln!("  ✗ Failed to seed {}: {}", name, e),
        }
    }

    Ok(())
}
