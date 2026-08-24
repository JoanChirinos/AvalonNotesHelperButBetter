//! Migration-safety tests.
//!
//! Migrations run automatically on backend startup against the LIVE avalon.db,
//! so a migration that works on an empty schema but breaks on real data would
//! panic the backend in production. These tests apply the embedded migrations to
//! a *copy* of a real DB and assert they succeed and leave the DB consistent.

use avalon_notes::db::MIGRATIONS;
use diesel::connection::SimpleConnection;
use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use diesel_migrations::MigrationHarness;

/// Run all embedded migrations against a fresh empty DB and integrity-check it.
/// This is the baseline: migrations must at least apply cleanly from scratch.
#[test]
fn migrations_apply_to_empty_db() {
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("empty.db");
    let mut conn = SqliteConnection::establish(db_path.to_str().unwrap()).unwrap();
    conn.batch_execute("PRAGMA foreign_keys=ON").unwrap();

    conn.run_pending_migrations(MIGRATIONS)
        .expect("migrations should apply to an empty DB");

    assert_integrity_ok(&mut conn);
}

/// Apply migrations to a copy of the live production DB, if present on this host.
/// Skips (does not fail) when the live DB isn't there — e.g. fresh checkout, CI
/// on a different machine — so the suite stays portable.
#[test]
fn migrations_apply_to_copy_of_live_db() {
    // ANH_LIVE_DB lets us point at a copy of prod data on a machine that doesn't
    // host the live app (CI leaves it unset and uses the on-host path).
    let live = std::env::var("ANH_LIVE_DB").unwrap_or_else(|_| {
        format!(
            "{}/AvalonNotesHelperButBetter/backend/avalon.db",
            std::env::var("HOME").unwrap_or_default()
        )
    });
    if !std::path::Path::new(&live).exists() {
        eprintln!("skipping: live DB not found at {live}");
        return;
    }

    let dir = tempfile::tempdir().unwrap();
    let copy = dir.path().join("live_copy.db");
    std::fs::copy(&live, &copy).expect("copy live DB");

    let mut conn = SqliteConnection::establish(copy.to_str().unwrap()).unwrap();
    conn.batch_execute("PRAGMA foreign_keys=ON").unwrap();

    conn.run_pending_migrations(MIGRATIONS)
        .expect("migrations should apply cleanly to real data");

    assert_integrity_ok(&mut conn);
    assert_no_foreign_key_violations(&mut conn);
}

#[derive(QueryableByName)]
struct TextRow {
    #[diesel(sql_type = diesel::sql_types::Text)]
    value: String,
}

#[derive(QueryableByName)]
struct CountRow {
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    value: i64,
}

fn assert_integrity_ok(conn: &mut SqliteConnection) {
    let rows: Vec<TextRow> =
        diesel::sql_query("SELECT integrity_check AS value FROM pragma_integrity_check()")
            .load(conn)
            .expect("integrity_check should run");
    assert_eq!(rows.len(), 1, "integrity_check returned multiple problems");
    assert_eq!(rows[0].value, "ok", "integrity_check reported problems");
}

fn assert_no_foreign_key_violations(conn: &mut SqliteConnection) {
    // Table-valued pragma so we can count violations directly; mapping the raw
    // multi-column PRAGMA foreign_key_check output by name is fragile.
    let rows: Vec<CountRow> =
        diesel::sql_query("SELECT count(*) AS value FROM pragma_foreign_key_check()")
            .load(conn)
            .expect("foreign_key_check should run");
    assert_eq!(rows[0].value, 0, "foreign_key_check found violations");
}
