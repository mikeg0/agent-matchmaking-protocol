use crate::error::{AmpError, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum NegotiationState {
    Discovery,
    Interest,
    Mutual,
    Disclosing,
    Disclosed,
    Meeting,
    Active,
    Withdrawn,
    Expired,
    Rejected,
    Blocked,
    SafetyHold,
}

pub struct NegotiationMachine;

impl NegotiationMachine {
    pub const PROGRESSION: [NegotiationState; 7] = [
        NegotiationState::Discovery,
        NegotiationState::Interest,
        NegotiationState::Mutual,
        NegotiationState::Disclosing,
        NegotiationState::Disclosed,
        NegotiationState::Meeting,
        NegotiationState::Active,
    ];

    pub fn is_terminal(state: NegotiationState) -> bool {
        matches!(
            state,
            NegotiationState::Withdrawn
                | NegotiationState::Expired
                | NegotiationState::Rejected
                | NegotiationState::Blocked
                | NegotiationState::SafetyHold
        )
    }

    pub fn next_state(current: NegotiationState) -> Option<NegotiationState> {
        let idx = Self::PROGRESSION.iter().position(|s| *s == current)?;
        Self::PROGRESSION.get(idx + 1).copied()
    }

    pub fn validate_transition(
        current: NegotiationState,
        to: NegotiationState,
        pending_human_approval: bool,
        actor_is_participant: bool,
    ) -> Result<()> {
        if Self::is_terminal(current) {
            return Err(AmpError::InvalidStateTransition(format!(
                "cannot transition from terminal state {current:?}"
            )));
        }

        if to == NegotiationState::Withdrawn {
            if actor_is_participant {
                return Ok(());
            }
            return Err(AmpError::InvalidStateTransition(
                "only participants can withdraw".to_string(),
            ));
        }

        if matches!(
            to,
            NegotiationState::Expired
                | NegotiationState::Rejected
                | NegotiationState::Blocked
                | NegotiationState::SafetyHold
        ) {
            return Err(AmpError::InvalidStateTransition(format!(
                "transition to terminal state {to:?} requires dedicated safety/moderation flow"
            )));
        }

        let expected = Self::next_state(current);
        if expected != Some(to) {
            return Err(AmpError::InvalidStateTransition(format!(
                "invalid transition {current:?} -> {to:?}; expected {expected:?}"
            )));
        }

        if (current == NegotiationState::Mutual && to == NegotiationState::Disclosing)
            || (current == NegotiationState::Disclosed && to == NegotiationState::Meeting)
        {
            if !pending_human_approval {
                return Err(AmpError::InvalidStateTransition(format!(
                    "transition {current:?} -> {to:?} requires human approval"
                )));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{NegotiationMachine, NegotiationState};

    #[test]
    fn allows_progression_transition() {
        let result = NegotiationMachine::validate_transition(
            NegotiationState::Discovery,
            NegotiationState::Interest,
            true,
            true,
        );
        assert!(result.is_ok());
    }

    #[test]
    fn blocks_skipped_transition() {
        let result = NegotiationMachine::validate_transition(
            NegotiationState::Discovery,
            NegotiationState::Mutual,
            true,
            true,
        );
        assert!(result.is_err());
    }

    #[test]
    fn allows_withdraw_for_participant() {
        let result = NegotiationMachine::validate_transition(
            NegotiationState::Interest,
            NegotiationState::Withdrawn,
            false,
            true,
        );
        assert!(result.is_ok());
    }

    #[test]
    fn blocks_human_gate_without_approval() {
        let result = NegotiationMachine::validate_transition(
            NegotiationState::Mutual,
            NegotiationState::Disclosing,
            false,
            true,
        );
        assert!(result.is_err());
    }
}
