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

    println!("Database setup complete!");
    Ok(pool)
}
