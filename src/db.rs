// use sqlx::{postgres::PgPoolOptions, migrate::MigrateDatabase, Pool, Postgres};
// use tracing::{info, error, instrument};
// use std::path::Path;
//
// #[instrument(skip(database_url))]
// pub async fn create_pool(database_url: &str) -> Result<Pool<Postgres>, sqlx::Error> {
//
//     let db_exists = Postgres::database_exists(database_url).await?;
//
//     if !db_exists {
//         info!("Database does not exist, creating...");
//         Postgres::create_database(database_url).await?;
//         info!("Database created successfully");
//     } else {
//         info!("Database already exists");
//     }
//
//     let pool = PgPoolOptions::new()
//         .max_connections(10)
//         .connect(database_url)
//         .await?;
//
//     info!("Database connection pool created successfully");
//     Ok(pool)
// }
//
// #[instrument(skip(pool))]
// pub async fn run_migrations(pool: &Pool<Postgres>) -> Result<(), sqlx::Error> {
//     info!("Running database migrations...");
//
//     let migrations = sqlx::migrate::Migrator::new(Path::new("./migrations"))
//         .await?
//         .run(pool)
//         .await;
//
//     match migrations {
//         Ok(_) => {
//             info!("Migrations completed successfully");
//             Ok(())
//         },
//         Err(e) => {
//             error!("Migration error: {:?}", e);
//             Err(e.into())
//         }
//     }
// }
use sqlx::{sqlite::SqlitePoolOptions, migrate::MigrateDatabase, Pool, Sqlite};
use tracing::{info, error, instrument};
use std::path::Path;

#[instrument(skip(database_url))]
pub async fn create_pool(database_url: &str) -> Result<Pool<Sqlite>, sqlx::Error> {
    // Check if the database exists. SQLite creates the database file if it doesn't exist.
    let db_exists = Path::new(database_url).exists();

    if !db_exists {
        info!("Database does not exist, creating...");
        // SQLite creates the database when you connect to it, no need to explicitly create it
        info!("Database created successfully");
    } else {
        info!("Database already exists");
    }

    // Create the connection pool for SQLite
    let pool = SqlitePoolOptions::new()
        .max_connections(10)
        .connect(database_url)
        .await?;

    info!("Database connection pool created successfully");
    Ok(pool)
}

#[instrument(skip(pool))]
pub async fn run_migrations(pool: &Pool<Sqlite>) -> Result<(), sqlx::Error> {
    info!("Running database migrations...");

    let migrations = sqlx::migrate::Migrator::new(Path::new("./migrations"))
        .await?
        .run(pool)
        .await;

    match migrations {
        Ok(_) => {
            info!("Migrations completed successfully");
            Ok(())
        },
        Err(e) => {
            error!("Migration error: {:?}", e);
            Err(e.into())
        }
    }
}
