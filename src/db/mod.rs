use anyhow::{Context, Result};
use rusqlite::{params, Connection};
use rust_decimal::Decimal;
use std::str::FromStr;
use tracing::instrument;

// Các struct hiện có (giả sử)
pub struct Employee {
    pub id: i64,
    pub name: String,
    pub role: String,
    pub device_id: Option<String>,
    pub hourly_rate: Decimal,
    pub created_at: String,
    pub updated_at: String,
}

/// Repository functions for Employee operations.
pub mod employee_repo {
    use super::*;

    /// Create a new employee and return the generated id.
    #[instrument(skip(conn))]
    pub fn create(
        conn: &Connection,
        name: &str,
        role: &str,
        device_id: Option<&str>,
        hourly_rate: &Decimal,
    ) -> Result<i64> {
        let rate_str = hourly_rate.to_string();
        conn.execute(
            "INSERT INTO employees (name, role, device_id, hourly_rate) VALUES (?1, ?2, ?3, ?4)",
            params![name, role, device_id, rate_str],
        )
        .context("Failed to insert employee")?;
        Ok(conn.last_insert_rowid())
    }

    /// Retrieve an employee by id.
    #[instrument(skip(conn))]
    pub fn get_by_id(conn: &Connection, id: i64) -> Result<Option<Employee>> {
        let mut stmt = conn
            .prepare("SELECT id, name, role, device_id, hourly_rate, created_at, updated_at FROM employees WHERE id = ?1")
            .context("Failed to prepare get employee statement")?;

        let mut rows = stmt
            .query_map(params![id], |row| {
                let rate_str: String = row.get(4)?;
                let rate = Decimal::from_str(&rate_str)
                    .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
                Ok(Employee {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    role: row.get(2)?,
                    device_id: row.get(3)?,
                    hourly_rate: rate,
                    created_at: row.get(5)?,
                    updated_at: row.get(6)?,
                })
            })
            .context("Failed to query employee")?;

        match rows.next() {
            Some(Ok(emp)) => Ok(Some(emp)),
            Some(Err(e)) => Err(e.into()),
            None => Ok(None),
        }
    }

    /// List all employees.
    #[instrument(skip(conn))]
    pub fn list(conn: &Connection) -> Result<Vec<Employee>> {
        let mut stmt = conn
            .prepare("SELECT id, name, role, device_id, hourly_rate, created_at, updated_at FROM employees ORDER BY id")
            .context("Failed to prepare list employees statement")?;

        let rows = stmt
            .query_map([], |row| {
                let rate_str: String = row.get(4)?;
                let rate = Decimal::from_str(&rate_str)
                    .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
                Ok(Employee {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    role: row.get(2)?,
                    device_id: row.get(3)?,
                    hourly_rate: rate,
                    created_at: row.get(5)?,
                    updated_at: row.get(6)?,
                })
            })
            .context("Failed to query employees")?;

        let mut employees = Vec::new();
        for row in rows {
            employees.push(row.context("Failed to read employee row")?);
        }
        Ok(employees)
    }

    /// Update an existing employee. Returns true if a row was updated.
    #[instrument(skip(conn))]
    pub fn update(
        conn: &Connection,
        id: i64,
        name: &str,
        role: &str,
        device_id: Option<&str>,
        hourly_rate: &Decimal,
    ) -> Result<bool> {
        let rate_str = hourly_rate.to_string();
        let affected = conn
            .execute(
                "UPDATE employees SET name = ?1, role = ?2, device_id = ?3, hourly_rate = ?4, updated_at = datetime('now') WHERE id = ?5",
                params![name, role, device_id, rate_str, id],
            )
            .context("Failed to update employee")?;
        Ok(affected > 0)
    }
}

pub struct AttendanceLog {
    pub id: i64,
    pub employee_id: i64,
    pub event_type: String, // "check_in" or "check_out"
    pub timestamp: String,
    pub correlation_id: String,
    pub created_at: String,
}

/// Repository functions for Attendance operations.
pub mod attendance_repo {
    use super::*;
    use anyhow::{Context, Result};
    use rusqlite::params;
    use tracing::instrument;

    /// Create a check-in log entry.
    #[instrument(skip(conn))]
    pub fn create_check_in(
        conn: &Connection,
        employee_id: i64,
        device_id: Option<&str>,
        correlation_id: &str,
    ) -> Result<AttendanceLog> {
        conn.execute(
            "INSERT INTO attendance_logs (employee_id, event_type, device_id, correlation_id, timestamp) \
             VALUES (?1, 'check_in', ?2, ?3, strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))",
            params![employee_id, device_id, correlation_id],
        )
        .context("Failed to insert check-in")?;

        let id = conn.last_insert_rowid();
        get_by_id(conn, id)
    }

    /// Create a check-out log entry.
    #[instrument(skip(conn))]
    pub fn create_check_out(
        conn: &Connection,
        employee_id: i64,
        device_id: Option<&str>,
        correlation_id: &str,
    ) -> Result<AttendanceLog> {
        conn.execute(
            "INSERT INTO attendance_logs (employee_id, event_type, device_id, correlation_id, timestamp) \
             VALUES (?1, 'check_out', ?2, ?3, strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))",
            params![employee_id, device_id, correlation_id],
        )
        .context("Failed to insert check-out")?;

        let id = conn.last_insert_rowid();
        get_by_id(conn, id)
    }

    /// Get a single attendance log by id.
    fn get_by_id(conn: &Connection, id: i64) -> Result<AttendanceLog> {
        let mut stmt = conn
            .prepare("SELECT id, employee_id, event_type, timestamp, correlation_id, created_at FROM attendance_logs WHERE id = ?1")
            .context("Failed to prepare get attendance statement")?;

        let log = stmt
            .query_row(params![id], |row| {
                Ok(AttendanceLog {
                    id: row.get(0)?,
                    employee_id: row.get(1)?,
                    event_type: row.get(2)?,
                    timestamp: row.get(3)?,
                    correlation_id: row.get(4)?,
                    created_at: row.get(5)?,
                })
            })
            .context("Failed to query attendance log")?;

        Ok(log)
    }

    /// Get the most recent event_type for an employee, or None if no records exist.
    #[instrument(skip(conn))]
    pub fn get_last_event_type(conn: &Connection, employee_id: i64) -> Result<Option<String>> {
        let mut stmt = conn
            .prepare(
                "SELECT event_type FROM attendance_logs \
                 WHERE employee_id = ?1 ORDER BY timestamp DESC, id DESC LIMIT 1",
            )
            .context("Failed to prepare last event query")?;

        let mut rows = stmt
            .query_map(params![employee_id], |row| row.get(0))
            .context("Failed to query last event type")?;

        match rows.next() {
            Some(Ok(event_type)) => Ok(Some(event_type)),
            Some(Err(e)) => Err(e.into()),
            None => Ok(None),
        }
    }

    /// List attendance logs, optionally filtered by employee_id, date (YYYY-MM-DD), or month (YYYY-MM).
    /// If both `date` and `month` are provided, `date` takes precedence.
    #[instrument(skip(conn))]
    pub fn list(
        conn: &Connection,
        employee_id: Option<i64>,
        date: Option<&str>,
        month: Option<&str>,
    ) -> Result<Vec<AttendanceLog>> {
        let mut conds: Vec<String> = Vec::new();
        let mut param_values: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

        if let Some(eid) = employee_id {
            conds.push(format!("employee_id = ?{}", conds.len() + 1));
            param_values.push(Box::new(eid));
        }
        if let Some(d) = date {
            conds.push(format!("DATE(timestamp) = ?{}", conds.len() + 1));
            param_values.push(Box::new(d.to_string()));
        } else if let Some(m) = month {
            conds.push(format!("strftime('%Y-%m', timestamp) = ?{}", conds.len() + 1));
            param_values.push(Box::new(m.to_string()));
        }

        let where_clause = if conds.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", conds.join(" AND "))
        };

        let sql = format!(
            "SELECT id, employee_id, event_type, timestamp, correlation_id, created_at \
             FROM attendance_logs {} ORDER BY timestamp ASC",
            where_clause
        );

        let mut stmt = conn.prepare(&sql).context("Failed to prepare list attendance statement")?;
        let params_refs: Vec<&dyn rusqlite::types::ToSql> =
            param_values.iter().map(|b| b.as_ref()).collect();

        let rows = stmt
            .query_map(params_refs.as_slice(), |row| {
                Ok(AttendanceLog {
                    id: row.get(0)?,
                    employee_id: row.get(1)?,
                    event_type: row.get(2)?,
                    timestamp: row.get(3)?,
                    correlation_id: row.get(4)?,
                    created_at: row.get(5)?,
                })
            })
            .context("Failed to query attendance logs")?;

        let mut logs = Vec::new();
        for row in rows {
            logs.push(row.context("Failed to read attendance row")?);
        }
        Ok(logs)
    }
}

pub struct PayrollPeriod {
    pub id: i64,
    pub start_date: String,
    pub end_date: String,
    pub is_closed: bool,
    pub created_at: String,
}

pub struct PayrollEntry {
    pub id: i64,
    pub payroll_period_id: i64,
    pub employee_id: i64,
    pub base_salary: String,
    pub overtime_pay: String,
    pub deduction: String,
    pub deposit_deduction: String,
    pub net_pay: String,
    pub correlation_id: String,
    pub created_at: String,
}

// === Bảng mới ===

pub struct WorkSession {
    pub id: i64,
    pub employee_id: i64,
    pub check_in_id: i64,
    pub check_out_id: Option<i64>,
    pub check_in_time: String,
    pub check_out_time: Option<String>,
    pub duration_minutes: Option<i64>,
    pub correlation_id: String,
    pub created_at: String,
}

pub struct DepositDeductionLog {
    pub id: i64,
    pub deposit_id: i64,
    pub payroll_entry_id: i64,
    pub amount: String,
    pub remaining_amount: String,
    pub correlation_id: String,
    pub created_at: String,
}

pub struct ExportLog {
    pub id: i64,
    pub employee_id: i64,
    pub export_type: String,
    pub file_name: String,
    pub record_count: Option<i64>,
    pub correlation_id: String,
    pub created_at: String,
}

pub struct SystemConfig {
    pub key: String,
    pub value: String,
    pub description: Option<String>,
    pub updated_by: Option<i64>,
    pub updated_at: String,
}

pub mod deposit_repo;
pub mod schema;
