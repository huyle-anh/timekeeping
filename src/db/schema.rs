use anyhow::{Context, Result};
use rusqlite::Connection;

/// SQL statements to create all tables in the correct dependency order.
const CREATE_EMPLOYEES: &str = "
CREATE TABLE IF NOT EXISTS employees (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    name        TEXT NOT NULL,
    role        TEXT NOT NULL CHECK(role IN ('Admin', 'Manager', 'Staff')),
    device_id   TEXT UNIQUE,
    hourly_rate TEXT NOT NULL,
    created_at  TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at  TEXT NOT NULL DEFAULT (datetime('now'))
);
";

const CREATE_ATTENDANCE_LOGS: &str = "
CREATE TABLE IF NOT EXISTS attendance_logs (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    employee_id     INTEGER NOT NULL REFERENCES employees(id),
    event_type      TEXT NOT NULL CHECK(event_type IN ('check_in', 'check_out')),
    timestamp       TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    ip_address      TEXT,
    device_id       TEXT,
    correlation_id  TEXT NOT NULL,
    created_at      TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);

CREATE INDEX IF NOT EXISTS idx_attendance_employee_date
    ON attendance_logs(employee_id, timestamp);
";

const CREATE_DEPOSITS: &str = "
CREATE TABLE IF NOT EXISTS deposits (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    employee_id     INTEGER NOT NULL REFERENCES employees(id),
    amount          TEXT NOT NULL,
    state           TEXT NOT NULL DEFAULT 'Pending'
                    CHECK(state IN ('Pending', 'Active', 'Released', 'Forfeited')),
    correlation_id  TEXT NOT NULL DEFAULT '',
    created_at      TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at      TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_deposits_employee
    ON deposits(employee_id);
";

const CREATE_DEPOSIT_STATE_LOGS: &str = "
CREATE TABLE IF NOT EXISTS deposit_state_logs (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    deposit_id      INTEGER NOT NULL REFERENCES deposits(id),
    from_state      TEXT NOT NULL,
    to_state        TEXT NOT NULL,
    changed_by      TEXT NOT NULL DEFAULT 'system',
    reason          TEXT NOT NULL DEFAULT '',
    correlation_id  TEXT NOT NULL DEFAULT '',
    created_at      TEXT NOT NULL DEFAULT (datetime('now'))
);
";

const CREATE_PAYROLL_PERIODS: &str = "
CREATE TABLE IF NOT EXISTS payroll_periods (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    period_start    TEXT NOT NULL,
    period_end      TEXT NOT NULL,
    status          TEXT NOT NULL DEFAULT 'Open'
                    CHECK(status IN ('Open', 'Closed', 'Paid')),
    created_at      TEXT NOT NULL DEFAULT (datetime('now'))
);
";

const CREATE_PAYROLL_ENTRIES: &str = "
CREATE TABLE IF NOT EXISTS payroll_entries (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    payroll_period_id INTEGER NOT NULL REFERENCES payroll_periods(id),
    employee_id     INTEGER NOT NULL REFERENCES employees(id),
    base_salary     TEXT NOT NULL,
    overtime_pay    TEXT NOT NULL DEFAULT '0.00',
    deduction       TEXT NOT NULL DEFAULT '0.00',
    deposit_deduction TEXT NOT NULL DEFAULT '0.00',
    net_pay         TEXT NOT NULL,
    correlation_id  TEXT NOT NULL,
    created_at      TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(payroll_period_id, employee_id)
);

CREATE INDEX IF NOT EXISTS idx_payroll_period_employee
    ON payroll_entries(payroll_period_id, employee_id);
";

const CREATE_PAYROLL_ADJUSTMENTS: &str = "
CREATE TABLE IF NOT EXISTS payroll_adjustments (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    payroll_entry_id INTEGER NOT NULL REFERENCES payroll_entries(id),
    amount          TEXT NOT NULL,
    reason          TEXT NOT NULL,
    approved_by     INTEGER REFERENCES employees(id),
    correlation_id  TEXT NOT NULL,
    created_at      TEXT NOT NULL DEFAULT (datetime('now'))
);
";

const CREATE_BACKUP_LOGS: &str = "
CREATE TABLE IF NOT EXISTS backup_logs (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    backup_path     TEXT NOT NULL,
    file_size_bytes INTEGER,
    checksum        TEXT,
    status          TEXT NOT NULL CHECK(status IN ('Success', 'Failed')),
    error_message   TEXT,
    created_at      TEXT NOT NULL DEFAULT (datetime('now'))
);
";

// === New tables from migration 002 ===

const CREATE_WORK_SESSIONS: &str = "
CREATE TABLE IF NOT EXISTS work_sessions (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    employee_id     INTEGER NOT NULL REFERENCES employees(id),
    check_in_id     INTEGER NOT NULL REFERENCES attendance_logs(id),
    check_out_id    INTEGER REFERENCES attendance_logs(id),
    check_in_time   TEXT NOT NULL,
    check_out_time  TEXT,
    duration_minutes INTEGER,
    correlation_id  TEXT NOT NULL,
    created_at      TEXT NOT NULL DEFAULT (datetime('now'))
);
";

const CREATE_DEPOSIT_DEDUCTION_LOGS: &str = "
CREATE TABLE IF NOT EXISTS deposit_deduction_logs (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    deposit_id      INTEGER NOT NULL REFERENCES deposits(id),
    payroll_entry_id INTEGER NOT NULL REFERENCES payroll_entries(id),
    amount          TEXT NOT NULL,
    remaining_amount TEXT NOT NULL,
    correlation_id  TEXT NOT NULL,
    created_at      TEXT NOT NULL DEFAULT (datetime('now'))
);
";

const CREATE_EXPORT_LOGS: &str = "
CREATE TABLE IF NOT EXISTS export_logs (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    employee_id     INTEGER NOT NULL REFERENCES employees(id),
    export_type     TEXT NOT NULL CHECK(export_type IN ('payroll', 'deposit', 'attendance')),
    file_name       TEXT NOT NULL,
    record_count    INTEGER,
    correlation_id  TEXT NOT NULL,
    created_at      TEXT NOT NULL DEFAULT (datetime('now'))
);
";

const CREATE_SYSTEM_CONFIG: &str = "
CREATE TABLE IF NOT EXISTS system_config (
    key             TEXT PRIMARY KEY,
    value           TEXT NOT NULL,
    description     TEXT,
    updated_by      INTEGER REFERENCES employees(id),
    updated_at      TEXT NOT NULL DEFAULT (datetime('now'))
);
";

/// Run all database migrations in the correct order.
/// Each statement is executed inside a transaction to ensure atomicity.
pub fn run_migrations(conn: &Connection) -> Result<()> {
    let tx = conn
        .unchecked_transaction()
        .context("Failed to start migration transaction")?;

    let statements = [
        CREATE_EMPLOYEES,
        CREATE_ATTENDANCE_LOGS,
        CREATE_DEPOSITS,
        CREATE_DEPOSIT_STATE_LOGS,
        CREATE_PAYROLL_PERIODS,
        CREATE_PAYROLL_ENTRIES,
        CREATE_PAYROLL_ADJUSTMENTS,
        CREATE_BACKUP_LOGS,
        CREATE_WORK_SESSIONS,
        CREATE_DEPOSIT_DEDUCTION_LOGS,
        CREATE_EXPORT_LOGS,
        CREATE_SYSTEM_CONFIG,
    ];

    for (i, sql) in statements.iter().enumerate() {
        tx.execute_batch(sql)
            .with_context(|| format!("Migration step {} failed", i + 1))?;
    }

    // Normalize legacy timestamps: convert "YYYY-MM-DD HH:MM:SS" → "YYYY-MM-DDTHH:MM:SSZ"
    tx.execute_batch(
        "UPDATE attendance_logs \
         SET timestamp = replace(timestamp, ' ', 'T') || 'Z' \
         WHERE instr(timestamp, 'T') = 0;"
    ).context("Failed to normalize attendance timestamps")?;

    tx.commit().context("Failed to commit migration transaction")?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    #[test]
    fn test_migration_runs_successfully() {
        let conn = Connection::open_in_memory().unwrap();
        run_migrations(&conn).unwrap();

        // Verify tables exist by querying sqlite_master (exclude internal sqlite_ tables)
        let tables: Vec<String> = conn
            .prepare(
                "SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%' ORDER BY name",
            )
            .unwrap()
            .query_map([], |row| row.get(0))
            .unwrap()
            .filter_map(|r| r.ok())
            .collect();

        let expected = vec![
            "attendance_logs",
            "backup_logs",
            "deposit_deduction_logs",
            "deposit_state_logs",
            "deposits",
            "employees",
            "export_logs",
            "payroll_adjustments",
            "payroll_entries",
            "payroll_periods",
            "system_config",
            "work_sessions",
        ];

        assert_eq!(tables, expected, "All expected tables must be created");
    }

    #[test]
    fn test_migration_is_idempotent() {
        let conn = Connection::open_in_memory().unwrap();
        run_migrations(&conn).unwrap();
        // Running again should not fail
        run_migrations(&conn).unwrap();
    }

    #[test]
    fn test_foreign_key_constraints() {
        let conn = Connection::open_in_memory().unwrap();
        // Enable foreign keys for this test
        conn.execute_batch("PRAGMA foreign_keys = ON;").unwrap();
        run_migrations(&conn).unwrap();

        // Attempt to insert an attendance_log with a non-existent employee_id
        let result = conn.execute(
            "INSERT INTO attendance_logs (employee_id, event_type, correlation_id) VALUES (?1, ?2, ?3)",
            rusqlite::params![999, "check_in", "test-correlation"],
        );

        assert!(result.is_err(), "Foreign key violation should be rejected");
    }
}
