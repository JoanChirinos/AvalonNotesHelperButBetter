use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

pub type DbPool = r2d2::Pool<ConnectionManager<SqliteConnection>>;

pub fn init_pool(database_url: &str) -> DbPool {
    let manager = ConnectionManager::<SqliteConnection>::new(database_url);
    let pool = r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create DB pool");

    // Run migrations
    let mut conn = pool.get().expect("Failed to get DB connection");
    conn.run_pending_migrations(MIGRATIONS)
        .expect("Failed to run migrations");

    // Enable WAL mode and foreign keys
    diesel::sql_query("PRAGMA journal_mode=WAL")
        .execute(&mut conn)
        .ok();
    diesel::sql_query("PRAGMA foreign_keys=ON")
        .execute(&mut conn)
        .ok();

    pool
}
