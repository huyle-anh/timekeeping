use rusqlite::{params, Connection, Result as SqlResult};
use rust_decimal::Decimal;
use std::str::FromStr;
use tracing::instrument;

use crate::deposit::{Deposit, DepositState, DepositStateLog};

/// Parse a DepositState from a DB string, returning a rusqlite error on failure.
fn parse_state(s: &str) -> SqlResult<DepositState> {
    DepositState::from_str(s).map_err(|e| {
        rusqlite::Error::FromSqlConversionFailure(
            3,
            rusqlite::types::Type::Text,
            Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, e)),
        )
    })
}

/// Parse a Decimal from a DB string, returning a rusqlite error on failure.
fn parse_decimal(s: &str, col: usize) -> SqlResult<Decimal> {
    Decimal::from_str(s).map_err(|e| {
        rusqlite::Error::FromSqlConversionFailure(
            col,
            rusqlite::types::Type::Text,
            Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Invalid decimal '{}': {}", s, e),
            )),
        )
    })
}

/// Create a new deposit for an employee.
#[instrument(skip(conn))]
pub fn create(
    conn: &Connection,
    employee_id: i64,
    amount: &Decimal,
    correlation_id: &str,
) -> SqlResult<i64> {
    conn.execute(
        "INSERT INTO deposits (employee_id, amount, state, correlation_id, created_at, updated_at)
         VALUES (?1, ?2, 'Pending', ?3, datetime('now'), datetime('now'))",
        params![employee_id, amount.to_string(), correlation_id],
    )?;
    Ok(conn.last_insert_rowid())
}

/// Get a deposit by its ID.
#[instrument(skip(conn))]
pub fn get_by_id(conn: &Connection, id: i64) -> SqlResult<Option<Deposit>> {
    let mut stmt = conn.prepare(
        "SELECT id, employee_id, amount, state, correlation_id, created_at, updated_at
         FROM deposits WHERE id = ?1",
    )?;

    let mut rows = stmt.query_map(params![id], |row| {
        let state_str: String = row.get(3)?;
        let state = parse_state(&state_str)?;
        let amount_str: String = row.get(2)?;
        let amount = parse_decimal(&amount_str, 2)?;

        Ok(Deposit {
            id: row.get(0)?,
            employee_id: row.get(1)?,
            amount,
            state,
            correlation_id: row.get(4)?,
            created_at: row.get(5)?,
            updated_at: row.get(6)?,
        })
    })?;

    match rows.next() {
        Some(Ok(deposit)) => Ok(Some(deposit)),
        Some(Err(e)) => Err(e),
        None => Ok(None),
    }
}

/// List all deposits, optionally filtered by employee.
#[instrument(skip(conn))]
pub fn list(conn: &Connection, employee_id: Option<i64>) -> SqlResult<Vec<Deposit>> {
    let (sql, param_values): (&str, Vec<Box<dyn rusqlite::types::ToSql>>) = match employee_id {
        Some(eid) => (
            "SELECT id, employee_id, amount, state, correlation_id, created_at, updated_at
             FROM deposits WHERE employee_id = ?1 ORDER BY created_at DESC",
            vec![Box::new(eid)],
        ),
        None => (
            "SELECT id, employee_id, amount, state, correlation_id, created_at, updated_at
             FROM deposits ORDER BY created_at DESC",
            vec![],
        ),
    };

    let mut stmt = conn.prepare(sql)?;

    let params_refs: Vec<&dyn rusqlite::types::ToSql> = param_values.iter().map(|p| p.as_ref()).collect();

    let rows = stmt.query_map(params_refs.as_slice(), |row| {
        let state_str: String = row.get(3)?;
        let state = parse_state(&state_str)?;
        let amount_str: String = row.get(2)?;
        let amount = parse_decimal(&amount_str, 2)?;

        Ok(Deposit {
            id: row.get(0)?,
            employee_id: row.get(1)?,
            amount,
            state,
            correlation_id: row.get(4)?,
            created_at: row.get(5)?,
            updated_at: row.get(6)?,
        })
    })?;

    let mut deposits = Vec::new();
    for row in rows {
        deposits.push(row?);
    }
    Ok(deposits)
}

/// Update the state of a deposit and log the transition.
#[instrument(skip(conn))]
pub fn update_state(
    conn: &Connection,
    deposit_id: i64,
    new_state: &DepositState,
    changed_by: &str,
    reason: &str,
    correlation_id: &str,
) -> SqlResult<bool> {
    // Get current state
    let current = get_by_id(conn, deposit_id)?;
    let current_state = match current {
        Some(ref d) => d.state,
        None => return Ok(false),
    };

    // Validate transition using the state machine
    if !current_state.can_transition_to(new_state) {
        return Ok(false);
    }

    // Use a savepoint so that the UPDATE and the audit log INSERT are atomic.
    conn.execute_batch("SAVEPOINT sp_update_deposit_state")?;

    let result: SqlResult<()> = (|| {
        let state_str = new_state.to_string();
        conn.execute(
            "UPDATE deposits SET state = ?1, updated_at = datetime('now') WHERE id = ?2",
            params![state_str, deposit_id],
        )?;

        let from_state_str = current_state.to_string();
        conn.execute(
            "INSERT INTO deposit_state_logs (deposit_id, from_state, to_state, changed_by, reason, correlation_id, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, datetime('now'))",
            params![deposit_id, from_state_str, state_str, changed_by, reason, correlation_id],
        )?;
        Ok(())
    })();

    match result {
        Ok(()) => {
            conn.execute_batch("RELEASE SAVEPOINT sp_update_deposit_state")?;
            Ok(true)
        }
        Err(e) => {
            conn.execute_batch("ROLLBACK TO SAVEPOINT sp_update_deposit_state").ok();
            Err(e)
        }
    }
}

/// Get state transition logs for a deposit.
#[instrument(skip(conn))]
pub fn get_state_logs(conn: &Connection, deposit_id: i64) -> SqlResult<Vec<DepositStateLog>> {
    let mut stmt = conn.prepare(
        "SELECT id, deposit_id, from_state, to_state, changed_by, reason, correlation_id, created_at
         FROM deposit_state_logs WHERE deposit_id = ?1 ORDER BY created_at ASC",
    )?;

    let rows = stmt.query_map(params![deposit_id], |row| {
        let from_str: String = row.get(2)?;
        let to_str: String = row.get(3)?;
        let from_state = parse_state(&from_str)?;
        let to_state = parse_state(&to_str)?;

        Ok(DepositStateLog {
            id: row.get(0)?,
            deposit_id: row.get(1)?,
            from_state,
            to_state,
            changed_by: row.get(4)?,
            reason: row.get(5)?,
            correlation_id: row.get(6)?,
            created_at: row.get(7)?,
        })
    })?;

    let mut logs = Vec::new();
    for row in rows {
        logs.push(row?);
    }
    Ok(logs)
}
