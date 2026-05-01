use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::instrument;
use uuid::Uuid;

use crate::db::employee_repo;
use crate::db::attendance_repo;
use crate::db::AttendanceLog;
use crate::errors::AppError;
use crate::AppState;
use rusqlite::params;

/// Check attendance rate limit for an employee. Returns Err if the limit is exceeded.
fn check_rate_limit(state: &Arc<AppState>, employee_id: i64) -> Result<(), AppError> {
    let mut limiter = state
        .rate_limiter
        .lock()
        .map_err(|_| AppError::Internal("Rate limiter unavailable".to_string()))?;

    let key = employee_id.to_string();
    let now = Instant::now();
    let window = Duration::from_secs(60);
    let limit = state.config.rate_limit_per_minute as usize;

    let timestamps = limiter.entry(key).or_insert_with(Vec::new);
    // Evict timestamps outside the rolling window
    timestamps.retain(|&t| now.duration_since(t) < window);

    if timestamps.len() >= limit {
        return Err(AppError::Validation(format!(
            "Rate limit exceeded: max {} attendance requests per minute for employee {}",
            limit, employee_id
        )));
    }
    timestamps.push(now);
    Ok(())
}

/// Health check endpoint.
#[instrument(skip(state))]
pub async fn health_check(State(state): State<Arc<AppState>>) -> Result<impl IntoResponse, AppError> {
    // Simple health check: try to get a connection from the pool
    let _conn = state.pool.get().map_err(|e| {
        tracing::error!("Health check failed: {}", e);
        AppError::Internal("Database connection failed".to_string())
    })?;
    Ok(Json(serde_json::json!({"status": "ok"})))
}

// ── Employee CRUD ──────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct CreateEmployeeRequest {
    pub name: String,
    pub role: String,
    pub device_id: Option<String>,
    pub pay_type: String,             // "Hourly" or "Salary"
    pub hourly_rate: Option<String>,  // required when pay_type == "Hourly"
    pub monthly_salary: Option<String>, // required when pay_type == "Salary"
}

#[derive(Debug, Serialize)]
pub struct EmployeeResponse {
    pub id: i64,
    pub name: String,
    pub role: String,
    pub device_id: Option<String>,
    pub pay_type: String,
    pub hourly_rate: Option<String>,
    pub monthly_salary: Option<String>,
    pub hours_worked_this_month: Option<f64>,
    pub total_salary_this_month: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// POST /employees (requires admin)
#[instrument(skip(state))]
pub async fn create_employee(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateEmployeeRequest>,
) -> Result<impl IntoResponse, AppError> {
    let correlation_id = Uuid::new_v4().to_string();
    tracing::info!(correlation_id = %correlation_id, "Creating employee");

    // Validate role
    let valid_roles = ["Admin", "Manager", "Staff"];
    if !valid_roles.contains(&payload.role.as_str()) {
        return Err(AppError::Validation(format!(
            "Invalid role '{}'. Must be one of: Admin, Manager, Staff",
            payload.role
        )));
    }

    // Validate pay_type
    let valid_pay_types = ["Hourly", "Salary"];
    if !valid_pay_types.contains(&payload.pay_type.as_str()) {
        return Err(AppError::Validation(format!(
            "Invalid pay_type '{}'. Must be 'Hourly' or 'Salary'",
            payload.pay_type
        )));
    }

    let hourly_rate: Option<Decimal> = payload
        .hourly_rate
        .as_deref()
        .filter(|s| !s.is_empty())
        .map(|s| s.parse().map_err(|e| AppError::Validation(format!("Invalid hourly_rate: {}", e))))
        .transpose()?;

    let monthly_salary: Option<Decimal> = payload
        .monthly_salary
        .as_deref()
        .filter(|s| !s.is_empty())
        .map(|s| s.parse().map_err(|e| AppError::Validation(format!("Invalid monthly_salary: {}", e))))
        .transpose()?;

    let conn = state
        .pool
        .get()
        .map_err(|e| AppError::Database(e))?;

    // Normalize device_id: treat empty string as absent to avoid UNIQUE constraint issues
    let device_id = payload.device_id.as_deref().filter(|s| !s.is_empty());

    let id = employee_repo::create(
        &conn,
        &payload.name,
        &payload.role,
        device_id,
        &payload.pay_type,
        hourly_rate.as_ref(),
        monthly_salary.as_ref(),
    )
    .map_err(|e| AppError::Internal(format!("Failed to create employee: {}", e)))?;

    let employee = employee_repo::get_by_id(&conn, id)
        .map_err(|e| AppError::Internal(format!("Failed to fetch created employee: {}", e)))?
        .ok_or_else(|| AppError::Internal("Created employee not found".to_string()))?;

    let current_month = chrono::Utc::now().format("%Y-%m").to_string();
    let (hours, salary) = employee_repo::get_hours_and_salary(&conn, employee.id, &current_month)
        .unwrap_or((0.0, None));
    let is_hourly = employee.pay_type == "Hourly";

    tracing::info!(correlation_id = %correlation_id, employee_id = %id, "Employee created");

    Ok((StatusCode::CREATED, Json(EmployeeResponse {
        id: employee.id,
        name: employee.name,
        role: employee.role,
        device_id: employee.device_id,
        pay_type: employee.pay_type,
        hourly_rate: employee.hourly_rate.map(|r| r.to_string()),
        monthly_salary: employee.monthly_salary.map(|s| s.to_string()),
        hours_worked_this_month: if is_hourly { Some(hours) } else { None },
        total_salary_this_month: salary.map(|s| s.to_string()),
        created_at: employee.created_at,
        updated_at: employee.updated_at,
    })))
}

/// GET /employees
#[instrument(skip(state))]
pub async fn list_employees(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
    let conn = state
        .pool
        .get()
        .map_err(|e| AppError::Database(e))?;

    let employees = employee_repo::list(&conn)
        .map_err(|e| AppError::Internal(format!("Failed to list employees: {}", e)))?;

    let current_month = chrono::Utc::now().format("%Y-%m").to_string();

    let response: Vec<EmployeeResponse> = employees
        .into_iter()
        .map(|emp| {
            let (hours, salary) = employee_repo::get_hours_and_salary(&conn, emp.id, &current_month)
                .unwrap_or((0.0, None));
            let is_hourly = emp.pay_type == "Hourly";
            EmployeeResponse {
                id: emp.id,
                name: emp.name,
                role: emp.role,
                device_id: emp.device_id,
                pay_type: emp.pay_type,
                hourly_rate: emp.hourly_rate.map(|r| r.to_string()),
                monthly_salary: emp.monthly_salary.map(|s| s.to_string()),
                hours_worked_this_month: if is_hourly { Some(hours) } else { None },
                total_salary_this_month: salary.map(|s| s.to_string()),
                created_at: emp.created_at,
                updated_at: emp.updated_at,
            }
        })
        .collect();
    Ok(Json(response))
}

/// GET /employees/:id
#[instrument(skip(state))]
pub async fn get_employee(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, AppError> {
    let conn = state
        .pool
        .get()
        .map_err(|e| AppError::Database(e))?;

    let employee = employee_repo::get_by_id(&conn, id)
        .map_err(|e| AppError::Internal(format!("Failed to get employee: {}", e)))?
        .ok_or_else(|| AppError::NotFound(format!("Employee with id {} not found", id)))?;

    let current_month = chrono::Utc::now().format("%Y-%m").to_string();
    let (hours, salary) = employee_repo::get_hours_and_salary(&conn, employee.id, &current_month)
        .unwrap_or((0.0, None));
    let is_hourly = employee.pay_type == "Hourly";

    Ok(Json(EmployeeResponse {
        id: employee.id,
        name: employee.name,
        role: employee.role,
        device_id: employee.device_id,
        pay_type: employee.pay_type,
        hourly_rate: employee.hourly_rate.map(|r| r.to_string()),
        monthly_salary: employee.monthly_salary.map(|s| s.to_string()),
        hours_worked_this_month: if is_hourly { Some(hours) } else { None },
        total_salary_this_month: salary.map(|s| s.to_string()),
        created_at: employee.created_at,
        updated_at: employee.updated_at,
    }))
}

#[derive(Debug, Deserialize)]
pub struct UpdateEmployeeRequest {
    pub name: String,
    pub role: String,
    pub device_id: Option<String>,
    pub pay_type: String,
    pub hourly_rate: Option<String>,
    pub monthly_salary: Option<String>,
}

/// DELETE /employees/:id (requires admin)
#[instrument(skip(state))]
pub async fn delete_employee(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, AppError> {
    let correlation_id = Uuid::new_v4().to_string();
    tracing::info!(correlation_id = %correlation_id, employee_id = %id, "Deleting employee");

    let conn = state
        .pool
        .get()
        .map_err(|e| AppError::Database(e))?;

    // Verify employee exists
    let _employee = employee_repo::get_by_id(&conn, id)
        .map_err(|e| AppError::Internal(format!("Failed to verify employee: {}", e)))?
        .ok_or_else(|| AppError::NotFound(format!("Employee with id {} not found", id)))?;

    // Delete employee
    conn.execute("DELETE FROM employees WHERE id = ?1", params![id])
        .map_err(|e| AppError::Internal(format!("Failed to delete employee: {}", e)))?;

    tracing::info!(correlation_id = %correlation_id, employee_id = %id, "Employee deleted");

    Ok(Json(serde_json::json!({"message": "Employee deleted successfully"})))
}

/// PUT /employees/:id (requires admin)
#[instrument(skip(state))]
pub async fn update_employee(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
    Json(payload): Json<UpdateEmployeeRequest>,
) -> Result<impl IntoResponse, AppError> {
    let correlation_id = Uuid::new_v4().to_string();
    tracing::info!(correlation_id = %correlation_id, employee_id = %id, "Updating employee");

    // Validate role
    let valid_roles = ["Admin", "Manager", "Staff"];
    if !valid_roles.contains(&payload.role.as_str()) {
        return Err(AppError::Validation(format!(
            "Invalid role '{}'. Must be one of: Admin, Manager, Staff",
            payload.role
        )));
    }

    // Validate pay_type
    let valid_pay_types = ["Hourly", "Salary"];
    if !valid_pay_types.contains(&payload.pay_type.as_str()) {
        return Err(AppError::Validation(format!(
            "Invalid pay_type '{}'. Must be 'Hourly' or 'Salary'",
            payload.pay_type
        )));
    }

    let hourly_rate: Option<Decimal> = payload
        .hourly_rate
        .as_deref()
        .filter(|s| !s.is_empty())
        .map(|s| s.parse().map_err(|e| AppError::Validation(format!("Invalid hourly_rate: {}", e))))
        .transpose()?;

    let monthly_salary: Option<Decimal> = payload
        .monthly_salary
        .as_deref()
        .filter(|s| !s.is_empty())
        .map(|s| s.parse().map_err(|e| AppError::Validation(format!("Invalid monthly_salary: {}", e))))
        .transpose()?;

    let conn = state
        .pool
        .get()
        .map_err(|e| AppError::Database(e))?;

    let device_id = payload.device_id.as_deref().filter(|s| !s.is_empty());

    let updated = employee_repo::update(
        &conn,
        id,
        &payload.name,
        &payload.role,
        device_id,
        &payload.pay_type,
        hourly_rate.as_ref(),
        monthly_salary.as_ref(),
    )
    .map_err(|e| AppError::Internal(format!("Failed to update employee: {}", e)))?;

    if !updated {
        return Err(AppError::NotFound(format!("Employee with id {} not found", id)));
    }

    let employee = employee_repo::get_by_id(&conn, id)
        .map_err(|e| AppError::Internal(format!("Failed to fetch updated employee: {}", e)))?
        .ok_or_else(|| AppError::Internal("Updated employee not found".to_string()))?;

    let current_month = chrono::Utc::now().format("%Y-%m").to_string();
    let (hours, salary) = employee_repo::get_hours_and_salary(&conn, employee.id, &current_month)
        .unwrap_or((0.0, None));
    let is_hourly = employee.pay_type == "Hourly";

    tracing::info!(correlation_id = %correlation_id, employee_id = %id, "Employee updated");

    Ok(Json(EmployeeResponse {
        id: employee.id,
        name: employee.name,
        role: employee.role,
        device_id: employee.device_id,
        pay_type: employee.pay_type,
        hourly_rate: employee.hourly_rate.map(|r| r.to_string()),
        monthly_salary: employee.monthly_salary.map(|s| s.to_string()),
        hours_worked_this_month: if is_hourly { Some(hours) } else { None },
        total_salary_this_month: salary.map(|s| s.to_string()),
        created_at: employee.created_at,
        updated_at: employee.updated_at,
    }))
}

// ── Attendance (Check-in/Check-out) ────────────────────────────

#[derive(Debug, Deserialize)]
pub struct CheckInRequest {
    pub employee_id: i64,
    pub device_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CheckOutRequest {
    pub employee_id: i64,
    pub device_id: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct AttendanceResponse {
    pub id: i64,
    pub employee_id: i64,
    pub event_type: String,
    pub timestamp: String,
    pub correlation_id: String,
}

impl From<AttendanceLog> for AttendanceResponse {
    fn from(a: AttendanceLog) -> Self {
        AttendanceResponse {
            id: a.id,
            employee_id: a.employee_id,
            event_type: a.event_type,
            timestamp: a.timestamp,
            correlation_id: a.correlation_id,
        }
    }
}

/// POST /attendance/check-in
#[instrument(skip(state))]
pub async fn check_in(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CheckInRequest>,
) -> Result<impl IntoResponse, AppError> {
    let correlation_id = Uuid::new_v4().to_string();
    tracing::info!(correlation_id = %correlation_id, employee_id = %payload.employee_id, "Check-in");

    check_rate_limit(&state, payload.employee_id)?;

    let conn = state
        .pool
        .get()
        .map_err(|e| AppError::Database(e))?;

    // Verify employee exists
    let _employee = employee_repo::get_by_id(&conn, payload.employee_id)
        .map_err(|e| AppError::Internal(format!("Failed to verify employee: {}", e)))?
        .ok_or_else(|| AppError::NotFound(format!("Employee {} not found", payload.employee_id)))?;

    // Validate: employee must not already be checked in
    let last_event = attendance_repo::get_last_event_type(&conn, payload.employee_id)
        .map_err(|e| AppError::Internal(format!("Failed to read last event: {}", e)))?;
    if last_event.as_deref() == Some("check_in") {
        return Err(AppError::Validation(format!(
            "Employee {} is already checked in. Please check out first.",
            payload.employee_id
        )));
    }

    // Normalize device_id: treat empty string as absent
    let device_id = payload.device_id.as_deref().filter(|s| !s.is_empty());

    let log = attendance_repo::create_check_in(
        &conn,
        payload.employee_id,
        device_id,
        &correlation_id,
    )
    .map_err(|e| AppError::Internal(format!("Failed to create check-in: {}", e)))?;

    tracing::info!(correlation_id = %correlation_id, attendance_id = %log.id, "Check-in recorded");

    Ok((StatusCode::CREATED, Json(AttendanceResponse::from(log))))
}

/// POST /attendance/check-out
#[instrument(skip(state))]
pub async fn check_out(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CheckOutRequest>,
) -> Result<impl IntoResponse, AppError> {
    let correlation_id = Uuid::new_v4().to_string();
    tracing::info!(correlation_id = %correlation_id, employee_id = %payload.employee_id, "Check-out");

    check_rate_limit(&state, payload.employee_id)?;

    let conn = state
        .pool
        .get()
        .map_err(|e| AppError::Database(e))?;

    // Verify employee exists
    let _employee = employee_repo::get_by_id(&conn, payload.employee_id)
        .map_err(|e| AppError::Internal(format!("Failed to verify employee: {}", e)))?
        .ok_or_else(|| AppError::NotFound(format!("Employee {} not found", payload.employee_id)))?;

    // Validate: employee must have an open check-in before checking out
    let last_event = attendance_repo::get_last_event_type(&conn, payload.employee_id)
        .map_err(|e| AppError::Internal(format!("Failed to read last event: {}", e)))?;
    if last_event.as_deref() != Some("check_in") {
        return Err(AppError::Validation(format!(
            "Employee {} has no open check-in. Please check in first.",
            payload.employee_id
        )));
    }

    // Normalize device_id: treat empty string as absent
    let device_id = payload.device_id.as_deref().filter(|s| !s.is_empty());

    let log = attendance_repo::create_check_out(
        &conn,
        payload.employee_id,
        device_id,
        &correlation_id,
    )
    .map_err(|e| AppError::Internal(format!("Failed to create check-out: {}", e)))?;

    tracing::info!(correlation_id = %correlation_id, attendance_id = %log.id, "Check-out recorded");

    Ok((StatusCode::CREATED, Json(AttendanceResponse::from(log))))
}

#[derive(Debug, Deserialize)]
pub struct ListAttendanceQuery {
    pub employee_id: Option<i64>,
    /// Filter by exact date YYYY-MM-DD
    pub date: Option<String>,
    /// Filter by month YYYY-MM (ignored if `date` is set)
    pub month: Option<String>,
}

/// GET /attendance
#[instrument(skip(state))]
pub async fn list_attendance(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ListAttendanceQuery>,
) -> Result<impl IntoResponse, AppError> {
    let conn = state
        .pool
        .get()
        .map_err(|e| AppError::Database(e))?;

    let logs = attendance_repo::list(
        &conn,
        query.employee_id,
        query.date.as_deref(),
        query.month.as_deref(),
    )
    .map_err(|e| AppError::Internal(format!("Failed to list attendance: {}", e)))?;

    let response: Vec<AttendanceResponse> = logs.into_iter().map(AttendanceResponse::from).collect();
    Ok(Json(response))
}

#[derive(Debug, Deserialize)]
pub struct GetEmployeeAttendanceQuery {
    /// Filter by date in YYYY-MM-DD format
    pub date: Option<String>,
}

/// GET /employees/:id/attendance
#[instrument(skip(state))]
pub async fn get_employee_attendance(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
    Query(query): Query<GetEmployeeAttendanceQuery>,
) -> Result<impl IntoResponse, AppError> {
    let conn = state
        .pool
        .get()
        .map_err(|e| AppError::Database(e))?;

    // Verify employee exists
    let _employee = employee_repo::get_by_id(&conn, id)
        .map_err(|e| AppError::Internal(format!("Failed to verify employee: {}", e)))?
        .ok_or_else(|| AppError::NotFound(format!("Employee {} not found", id)))?;

    let logs = attendance_repo::list(&conn, Some(id), query.date.as_deref(), None)
        .map_err(|e| AppError::Internal(format!("Failed to list attendance for employee {}: {}", id, e)))?;

    let response: Vec<AttendanceResponse> = logs.into_iter().map(AttendanceResponse::from).collect();
    Ok(Json(response))
}
