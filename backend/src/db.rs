use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager, CustomizeConnection};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

pub type DbPool = r2d2::Pool<ConnectionManager<SqliteConnection>>;

#[derive(Debug)]
struct SqliteConnectionCustomizer;

impl CustomizeConnection<SqliteConnection, diesel::r2d2::Error> for SqliteConnectionCustomizer {
    fn on_acquire(&self, conn: &mut SqliteConnection) -> Result<(), diesel::r2d2::Error> {
        diesel::sql_query("PRAGMA foreign_keys=ON")
            .execute(conn)
            .expect("Failed to enable foreign keys");
        Ok(())
    }
}

pub fn init_pool(database_url: &str) -> DbPool {
    let manager = ConnectionManager::<SqliteConnection>::new(database_url);
    let pool = r2d2::Pool::builder()
        .connection_customizer(Box::new(SqliteConnectionCustomizer))
        .build(manager)
        .expect("Failed to create DB pool");

    let mut conn = pool.get().expect("Failed to get DB connection");
    conn.run_pending_migrations(MIGRATIONS)
        .expect("Failed to run migrations");
    diesel::sql_query("PRAGMA journal_mode=WAL")
        .execute(&mut conn)
        .ok();

    pool
}
