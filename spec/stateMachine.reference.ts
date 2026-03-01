/**
 * AMP/1.0 Negotiation State Machine
 *
 * Progression:
 *   DISCOVERY → INTEREST → MUTUAL → DISCLOSING → DISCLOSED → MEETING → ACTIVE
 *
 * Terminal states:
 *   WITHDRAWN | EXPIRED | REJECTED | BLOCKED | SAFETY_HOLD
 *
 * Rules:
 * - Progressive states must be traversed in order; no skipping
 * - Either party can WITHDRAW from any non-terminal state
 * - Some transitions require human approval
 * - ACTIVE requires both parties to confirm
 */

export type NegotiationState =
  | 'DISCOVERY'
  | 'INTEREST'
  | 'MUTUAL'
  | 'DISCLOSING'
  | 'DISCLOSED'
  | 'MEETING'
  | 'ACTIVE'
  | 'WITHDRAWN'
  | 'EXPIRED'
  | 'REJECTED'
  | 'BLOCKED'
  | 'SAFETY_HOLD';

export interface StateTransition {
  from: NegotiationState;
  to: NegotiationState;
  actor: string; // opaque_id of the agent performing the action
  reason?: string;
  timestamp: string;
}

export interface Negotiation {
  id: string;
  initiator_opaque_id: string;
  target_opaque_id: string;
  state: NegotiationState;
  state_history: StateTransition[];
  disclosure_level: string;
  pending_human_approval: boolean;
  human_approval_required_for: string | null;
  human_approval_requested_at: string | null;
  human_approval_expires_at: string | null;
  meeting_proposed_by: string | null;
  meeting_proposed_at: string | null;
  activated_by: string[];
  created_at: string;
  updated_at: string;
}

// States that require human approval before the transition executes
export const HUMAN_GATED_TRANSITIONS: Partial<Record<NegotiationState, NegotiationState>> = {
  MUTUAL: 'DISCLOSING', // Human approves sharing more info
  DISCLOSED: 'MEETING', // Human approves meeting
};

export const TERMINAL_STATES: NegotiationState[] = [
  'WITHDRAWN',
  'EXPIRED',
  'REJECTED',
  'BLOCKED',
  'SAFETY_HOLD',
];

// Valid non-terminal progression transitions (in order)
const STATE_ORDER: NegotiationState[] = [
  'DISCOVERY',
  'INTEREST',
  'MUTUAL',
  'DISCLOSING',
  'DISCLOSED',
  'MEETING',
  'ACTIVE',
];

export function isTerminalState(state: NegotiationState): boolean {
  return TERMINAL_STATES.includes(state);
}

/**
 * Returns the next logical state (excluding terminal states).
 */
export function nextState(current: NegotiationState): NegotiationState | null {
  const idx = STATE_ORDER.indexOf(current);
  if (idx === -1 || idx === STATE_ORDER.length - 1) return null;
  return STATE_ORDER[idx + 1];
}

/**
 * Validates a proposed state transition.
 * Returns { valid: true } or { valid: false, reason: string }
 */
export function validateTransition(
  current: NegotiationState,
  to: NegotiationState,
  negotiation: Negotiation,
  actorId: string,
): { valid: true } | { valid: false; reason: string } {
  if (isTerminalState(current)) {
    return { valid: false, reason: `Cannot transition from terminal state ${current}` };
  }

  if (to === 'WITHDRAWN') {
    // Any participant can withdraw at any time
    const isParticipant = actorId === negotiation.initiator_opaque_id || actorId === negotiation.target_opaque_id;
    if (!isParticipant) {
      return { valid: false, reason: 'Only participants can withdraw' };
    }
    return { valid: true };
  }

  // Other terminal states are controlled by dedicated moderation / safety flows.
  if (to === 'EXPIRED' || to === 'REJECTED' || to === 'BLOCKED' || to === 'SAFETY_HOLD') {
    return { valid: false, reason: `Transition to terminal state ${to} requires dedicated safety/moderation flow` };
  }

  const expected = nextState(current);
  if (to !== expected) {
    return {
      valid: false,
      reason: `Invalid transition: ${current} → ${to}. Expected next state: ${expected ?? 'none'}`,
    };
  }

  // Check human gate — transition is blocked until human approves
  if (HUMAN_GATED_TRANSITIONS[current] === to && !negotiation.pending_human_approval) {
    return {
      valid: false,
      reason: `Transition ${current} → ${to} requires human approval. Call /human-approve first.`,
    };
  }

  return { valid: true };
}

/**
 * Builds a state transition record.
 */
export function buildTransition(
  from: NegotiationState,
  to: NegotiationState,
  actor: string,
  reason?: string,
): StateTransition {
  return { from, to, actor, reason, timestamp: new Date().toISOString() };
}
