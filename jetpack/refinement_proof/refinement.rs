// Jetpack refinement theorem skeleton.
//
// Mirrors src/protocol/Raft/refinement_proof/refinement.rs.
// The refinement mapping shows that the Jetpack distributed protocol
// refines a sequential command execution model.

#![allow(unused)]

use vstd::prelude::*;

include!("../types.rs");

verus! {

// Include protocol spec for access to LInit, LNext, and action predicates.
include!("../jetpack_body.rs");

// =========================================================================
// Abstract sequential state (refinement target)
// =========================================================================

/// The abstract state that Jetpack refines to: a sequential command log
/// where all committed commands across all proposers are consistently ordered.
pub struct JetpackAbstractState {
    /// The committed commands, in execution order, after dedup.
    pub committed_cmds: Seq<LCmd>,
    /// The set of server IDs in the cluster.
    pub server_ids: Set<int>,
    /// The set of proposer IDs.
    pub proposer_ids: Set<int>,
}

// =========================================================================
// Refinement mapping
// =========================================================================

/// Map Jetpack protocol state to abstract sequential state.
///
/// The refinement extracts the union of all committed commands across
/// all servers and proposers, deduplicated. The key safety claim is that
/// this extraction is consistent: all servers agree on the committed prefix
/// for each proposer (CommittedLogAgreement), and the execution trace
/// matches the committed log order (LogOrderMatchesExecution).
pub open spec fn AbstractifyJetpackState(s: LState, c: LConstants) -> JetpackAbstractState {
    JetpackAbstractState {
        // Use server 0's view as the canonical committed log
        // (CommittedLogAgreement ensures all servers agree)
        committed_cmds: s.execution_cmds,
        server_ids: c.server,
        proposer_ids: c.proposer,
    }
}

// =========================================================================
// Refinement theorem (skeleton)
// =========================================================================

/// The main refinement theorem: for any valid Jetpack behavior,
/// the abstracted behavior refines a sequential command execution.
///
/// Concretely:
///   1. CommittedLogAgreement ensures all servers see the same committed
///      prefix for each proposer -- the 3-D log is consistent.
///   2. LogOrderMatchesExecution ensures the committed command order
///      matches the execution trace for conflicting commands.
///   3. ExecutionDedupMatches ensures the deduplicated original and
///      replicated execution traces are conflict-order equivalent.
///
/// Together, these properties guarantee that external observers see
/// a consistent sequential execution despite the multi-proposer,
/// multi-server architecture.
///
/// PROOF STATUS: skeleton only.
/// BLOCKERS:
///   - lemma_init_establishes_invariant has assume(false)
///   - lemma_safety_invariant_inductive has assume(false)
///   - The three named property lemmas have assume(false)
///   - Quorum intersection lemma not yet formalized
///   - Conflict-order preservation through recovery not yet formalized
pub proof fn lemma_refinement_correct(
    s: LState, s_: LState, c: LConstants,
)
    requires
        // JetpackSafetyInvariant(s, c),
        LNext(s, s_, c),
    ensures
        // JetpackSafetyInvariant(s_, c),
        // AbstractifyJetpackState(s_, c) refines AbstractifyJetpackState(s, c)
        // (the abstract state evolves consistently)
        true,
{
    // PROOF STATUS: skeleton only.
    // Would call lemma_safety_invariant_inductive(s, s_, c) and then
    // show that the abstraction map preserves the refinement relation.
    assume(false); // TODO
}

} // verus!
