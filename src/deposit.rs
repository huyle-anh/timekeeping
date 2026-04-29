use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use std::fmt;
use std::str::FromStr;

/// Represents the possible states of a deposit in the state machine.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DepositState {
    /// Deposit is pending (awaiting payment from employee)
    Pending,
    /// Deposit is active (employee has paid, deposit is being held)
    Active,
    /// Deposit has been released back to the employee
    Released,
    /// Deposit has been forfeited (deducted due to policy violation)
    Forfeited,
}

impl DepositState {
    /// Returns true if transitioning from `self` to `target` is allowed.
    pub fn can_transition_to(&self, target: &DepositState) -> bool {
        matches!(
            (self, target),
            (DepositState::Pending, DepositState::Active)
                | (DepositState::Active, DepositState::Released)
                | (DepositState::Active, DepositState::Forfeited)
        )
    }

    /// Returns all valid target states from the current state.
    pub fn valid_transitions(&self) -> &'static [DepositState] {
        match self {
            DepositState::Pending => &[DepositState::Active],
            DepositState::Active => &[DepositState::Released, DepositState::Forfeited],
            DepositState::Released => &[],
            DepositState::Forfeited => &[],
        }
    }
}

impl fmt::Display for DepositState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DepositState::Pending => write!(f, "Pending"),
            DepositState::Active => write!(f, "Active"),
            DepositState::Released => write!(f, "Released"),
            DepositState::Forfeited => write!(f, "Forfeited"),
        }
    }
}

impl FromStr for DepositState {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Pending" => Ok(DepositState::Pending),
            "Active" => Ok(DepositState::Active),
            "Released" => Ok(DepositState::Released),
            "Forfeited" => Ok(DepositState::Forfeited),
            other => Err(format!("Unknown deposit state: {}", other)),
        }
    }
}

/// A deposit record for an employee.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Deposit {
    pub id: i64,
    pub employee_id: i64,
    pub amount: Decimal,
    pub state: DepositState,
    pub correlation_id: String,
    pub created_at: String,
    pub updated_at: String,
}

/// A log entry recording a state transition for a deposit.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DepositStateLog {
    pub id: i64,
    pub deposit_id: i64,
    pub from_state: DepositState,
    pub to_state: DepositState,
    pub changed_by: String, // user identifier or "system"
    pub reason: String,
    pub correlation_id: String,
    pub created_at: String,
}

/// Errors that can occur during deposit operations.
#[derive(Debug, Error)]
pub enum DepositError {
    #[error("Invalid state transition: cannot go from {from:?} to {to:?}")]
    InvalidTransition {
        from: DepositState,
        to: DepositState,
    },
    #[error("Deposit not found: {0}")]
    NotFound(i64),
    #[error("Database error: {0}")]
    Database(String),
    #[error("Validation error: {0}")]
    Validation(String),
}

/// Attempt to transition a deposit from its current state to a new state.
/// Returns the new state if the transition is valid, otherwise returns an error.
pub fn transition_state(
    current_state: &DepositState,
    target_state: &DepositState,
) -> Result<DepositState, DepositError> {
    if current_state.can_transition_to(target_state) {
        Ok(*target_state)
    } else {
        Err(DepositError::InvalidTransition {
            from: *current_state,
            to: *target_state,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_transitions() {
        // Pending -> Active is valid
        assert!(DepositState::Pending.can_transition_to(&DepositState::Active));
        // Pending -> Released is invalid
        assert!(!DepositState::Pending.can_transition_to(&DepositState::Released));
        // Pending -> Forfeited is invalid
        assert!(!DepositState::Pending.can_transition_to(&DepositState::Forfeited));

        // Active -> Released is valid
        assert!(DepositState::Active.can_transition_to(&DepositState::Released));
        // Active -> Forfeited is valid
        assert!(DepositState::Active.can_transition_to(&DepositState::Forfeited));
        // Active -> Pending is invalid
        assert!(!DepositState::Active.can_transition_to(&DepositState::Pending));

        // Released -> anything is invalid
        assert!(!DepositState::Released.can_transition_to(&DepositState::Pending));
        assert!(!DepositState::Released.can_transition_to(&DepositState::Active));
        assert!(!DepositState::Released.can_transition_to(&DepositState::Forfeited));

        // Forfeited -> anything is invalid
        assert!(!DepositState::Forfeited.can_transition_to(&DepositState::Pending));
        assert!(!DepositState::Forfeited.can_transition_to(&DepositState::Active));
        assert!(!DepositState::Forfeited.can_transition_to(&DepositState::Released));
    }

    #[test]
    fn test_transition_state_function() {
        assert!(transition_state(&DepositState::Pending, &DepositState::Active).is_ok());
        assert!(transition_state(&DepositState::Active, &DepositState::Released).is_ok());
        assert!(transition_state(&DepositState::Active, &DepositState::Forfeited).is_ok());

        assert!(transition_state(&DepositState::Pending, &DepositState::Released).is_err());
        assert!(transition_state(&DepositState::Released, &DepositState::Active).is_err());
    }

    #[test]
    fn test_valid_transitions_list() {
        assert_eq!(DepositState::Pending.valid_transitions(), &[DepositState::Active]);
        assert_eq!(
            DepositState::Active.valid_transitions(),
            &[DepositState::Released, DepositState::Forfeited]
        );
        assert!(DepositState::Released.valid_transitions().is_empty());
        assert!(DepositState::Forfeited.valid_transitions().is_empty());
    }
}
