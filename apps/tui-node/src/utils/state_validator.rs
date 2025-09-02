// State Validator - Ensures only valid state combinations
// This can be added to existing code without breaking changes

use super::state::{AppState, MeshStatus, DkgState};
use frost_core::Ciphersuite;
use std::fmt;

/// Represents the composite state as a single value
#[derive(Debug, Clone, PartialEq)]
pub struct CompositeState {
    pub has_session: bool,
    pub mesh_status: MeshStatus,
    pub dkg_state: DkgState,
}

/// Result of state validation
#[derive(Debug)]
pub enum StateValidity {
    Valid,
    Invalid(String),
    Warning(String),
}

impl fmt::Display for StateValidity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StateValidity::Valid => write!(f, "Valid"),
            StateValidity::Invalid(msg) => write!(f, "Invalid: {}", msg),
            StateValidity::Warning(msg) => write!(f, "Warning: {}", msg),
        }
    }
}

/// Extract composite state from AppState
pub fn get_composite_state<C: Ciphersuite>(app_state: &AppState<C>) -> CompositeState {
    CompositeState {
        has_session: app_state.session.is_some(),
        mesh_status: app_state.mesh_status.clone(),
        dkg_state: app_state.dkg_state.clone(),
    }
}

/// Validate if a state combination is valid according to design
pub fn validate_state_combination(state: &CompositeState) -> StateValidity {
    use MeshStatus::*;
    use DkgState::*;
    
    match (&state.has_session, &state.mesh_status, &state.dkg_state) {
        // Valid combinations (23 total)
        
        // Initial states
        (false, Incomplete, Idle) => StateValidity::Valid,
        
        // Session active states
        (true, Incomplete, Idle) => StateValidity::Valid,
        (true, PartiallyReady { .. }, Idle) => StateValidity::Valid,
        (true, Ready, Idle) => StateValidity::Valid,
        
        // DKG states (only with session AND full mesh)
        (true, Ready, Round1InProgress) => StateValidity::Valid,
        (true, Ready, Round1Complete) => StateValidity::Valid,
        (true, Ready, Round2InProgress) => StateValidity::Valid,
        (true, Ready, Round2Complete) => StateValidity::Valid,
        (true, Ready, Finalizing) => StateValidity::Valid,
        (true, Ready, Complete) => StateValidity::Valid,
        
        // Failed states
        (true, Ready, Failed(_)) => StateValidity::Valid,
        (true, PartiallyReady { .. }, Failed(_)) => StateValidity::Valid,
        (true, Incomplete, Failed(_)) => StateValidity::Valid,
        
        // Completed states (session might be gone)
        (false, Incomplete, Complete) => StateValidity::Valid,
        (false, Incomplete, Failed(_)) => StateValidity::Valid,
        (true, _, Complete) => StateValidity::Valid,
        
        // Invalid combinations
        
        // Can't have mesh without session
        (false, Ready, _) => StateValidity::Invalid(
            "Mesh cannot be Ready without an active session".to_string()
        ),
        (false, PartiallyReady { .. }, _) => StateValidity::Invalid(
            "Mesh cannot be PartiallyReady without an active session".to_string()
        ),
        
        // Can't do DKG without full mesh
        (_, Incomplete, Round1InProgress) => StateValidity::Invalid(
            "Cannot start DKG Round 1 without full mesh".to_string()
        ),
        (_, PartiallyReady { .. }, Round1InProgress) => StateValidity::Invalid(
            "Cannot start DKG Round 1 with only partial mesh".to_string()
        ),
        (_, Incomplete, Round2InProgress) => StateValidity::Invalid(
            "Cannot be in DKG Round 2 without full mesh".to_string()
        ),
        (_, PartiallyReady { .. }, Round2InProgress) => StateValidity::Invalid(
            "Cannot be in DKG Round 2 with only partial mesh".to_string()
        ),
        
        // Can't do DKG without session
        (false, _, Round1InProgress) => StateValidity::Invalid(
            "Cannot do DKG without an active session".to_string()
        ),
        (false, _, Round2InProgress) => StateValidity::Invalid(
            "Cannot do DKG without an active session".to_string()
        ),
        (false, _, Finalizing) => StateValidity::Invalid(
            "Cannot finalize DKG without an active session".to_string()
        ),
        
        // Catch-all for other invalid combinations
        _ => StateValidity::Warning(
            format!("Unusual state combination: session={}, mesh={:?}, dkg={:?}",
                state.has_session, state.mesh_status, state.dkg_state)
        ),
    }
}

/// Validate a state transition
pub fn validate_transition(
    from: &CompositeState,
    to: &CompositeState,
) -> Result<(), String> {
    use MeshStatus::*;
    use DkgState::*;
    
    // First check if target state is valid
    match validate_state_combination(to) {
        StateValidity::Invalid(msg) => return Err(format!("Invalid target state: {}", msg)),
        _ => {}
    }
    
    // Check if transition is valid
    match (&from.mesh_status, &from.dkg_state, &to.mesh_status, &to.dkg_state) {
        // DKG can only start from Idle when mesh is Ready
        (Ready, Idle, Ready, Round1InProgress) => Ok(()),
        
        // DKG round progression
        (Ready, Round1InProgress, Ready, Round1Complete) => Ok(()),
        (Ready, Round1Complete, Ready, Round2InProgress) => Ok(()),
        (Ready, Round2InProgress, Ready, Round2Complete) => Ok(()),
        (Ready, Round2Complete, Ready, Finalizing) => Ok(()),
        (Ready, Finalizing, Ready, Complete) => Ok(()),
        (Ready, Finalizing, Ready, Failed(_)) => Ok(()),
        
        // Mesh state changes
        (Incomplete, dkg, PartiallyReady { .. }, new_dkg) if dkg == new_dkg => Ok(()),
        (PartiallyReady { .. }, dkg, Ready, new_dkg) if dkg == new_dkg => Ok(()),
        (Ready, dkg, PartiallyReady { .. }, Failed(_)) if dkg != &Idle => Ok(()), // DKG fails on disconnect
        (PartiallyReady { .. }, dkg, Incomplete, new_dkg) if dkg == new_dkg => Ok(()),
        
        // Session state changes
        (_, Idle, _, Idle) if !from.has_session && to.has_session => Ok(()), // Session created
        (_, dkg, _, new_dkg) if from.has_session && !to.has_session && dkg == new_dkg => Ok(()), // Session ended
        
        // No change is always valid
        _ if from == to => Ok(()),
        
        // Everything else is invalid
        _ => Err(format!(
            "Invalid transition: ({}, {:?}, {:?}) -> ({}, {:?}, {:?})",
            from.has_session, from.mesh_status, from.dkg_state,
            to.has_session, to.mesh_status, to.dkg_state
        )),
    }
}

/// Check AppState and log warnings for invalid combinations
pub async fn check_state_validity<C: Ciphersuite>(app_state: &mut AppState<C>) {
    let composite = get_composite_state(app_state);
    
    match validate_state_combination(&composite) {
        StateValidity::Valid => {
            // State is valid, nothing to do
        },
        StateValidity::Invalid(msg) => {
            tracing::error!("Invalid state detected: {}", msg);
            
            // Could attempt recovery here
            // For now, just log for monitoring
        },
        StateValidity::Warning(msg) => {
            tracing::warn!("Unusual state: {}", msg);
        },
    }
}

/// Wrapper to ensure state updates maintain validity
pub async fn update_state_safely<C, F>(
    app_state: &mut AppState<C>,
    update_fn: F,
) -> Result<(), String>
where
    C: Ciphersuite,
    F: FnOnce(&mut AppState<C>),
{
    // Get current state
    let before = get_composite_state(app_state);
    
    // Apply update
    update_fn(app_state);
    
    // Get new state
    let after = get_composite_state(app_state);
    
    // Validate transition
    validate_transition(&before, &after)?;
    
    // Check final state validity
    check_state_validity(app_state).await;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_valid_states() {
        let valid_states = vec![
            (false, MeshStatus::Incomplete, DkgState::Idle),
            (true, MeshStatus::Incomplete, DkgState::Idle),
            (true, MeshStatus::Ready, DkgState::Idle),
            (true, MeshStatus::Ready, DkgState::Round1InProgress),
            (true, MeshStatus::Ready, DkgState::Complete),
        ];
        
        for (has_session, mesh, dkg) in valid_states {
            let state = CompositeState {
                has_session,
                mesh_status: mesh,
                dkg_state: dkg,
            };
            
            match validate_state_combination(&state) {
                StateValidity::Valid => {}, // Good
                other => panic!("State should be valid but got: {:?}", other),
            }
        }
    }
    
    #[test]
    fn test_invalid_states() {
        let invalid_states = vec![
            // Can't have mesh without session
            (false, MeshStatus::Ready, DkgState::Idle),
            // Can't do DKG without full mesh
            (true, MeshStatus::Incomplete, DkgState::Round1InProgress),
            (true, MeshStatus::PartiallyReady { ready_devices: Default::default(), total_devices: 3 }, DkgState::Round2InProgress),
            // Can't do DKG without session
            (false, MeshStatus::Ready, DkgState::Round1InProgress),
        ];
        
        for (has_session, mesh, dkg) in invalid_states {
            let state = CompositeState {
                has_session,
                mesh_status: mesh,
                dkg_state: dkg,
            };
            
            match validate_state_combination(&state) {
                StateValidity::Invalid(_) => {}, // Good, should be invalid
                other => panic!("State should be invalid but got: {:?}", other),
            }
        }
    }
    
    #[test]
    fn test_valid_transitions() {
        // Test DKG progression
        let from = CompositeState {
            has_session: true,
            mesh_status: MeshStatus::Ready,
            dkg_state: DkgState::Idle,
        };
        
        let to = CompositeState {
            has_session: true,
            mesh_status: MeshStatus::Ready,
            dkg_state: DkgState::Round1InProgress,
        };
        
        assert!(validate_transition(&from, &to).is_ok());
    }
    
    #[test]
    fn test_invalid_transitions() {
        // Can't go from Idle to Round2 directly
        let from = CompositeState {
            has_session: true,
            mesh_status: MeshStatus::Ready,
            dkg_state: DkgState::Idle,
        };
        
        let to = CompositeState {
            has_session: true,
            mesh_status: MeshStatus::Ready,
            dkg_state: DkgState::Round2InProgress,
        };
        
        assert!(validate_transition(&from, &to).is_err());
    }
}