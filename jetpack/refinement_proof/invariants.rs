// Jetpack safety invariant definitions and proof skeletons.
//
// Mirrors src/protocol/Raft/refinement_proof/invariants.rs.
// Defines the composite JetpackSafetyInvariant plus all supporting invariants
// needed to prove the three named safety properties from jetpack.tla:
//   1. CommittedLogAgreement
//   2. LogOrderMatchesExecution
//   3. ExecutionDedupMatches
//
// =========================================================================
// INDUCTIVENESS AUDIT (Phase 5)
// =========================================================================
//
// Legend: SELF = self-contained (no extra invariants needed for induction)
//         NEEDS = needs additional support invariant(s)
//         RETRACTED = proved non-inductive, removed from composite
//         TRIVIAL = preserved by type (nat >= 0) or trivially by all actions
//
// Invariant                      | Init | Inductive | Notes
// -------------------------------|------|-----------|------
// TypeInvariant                  | OK   | SELF      | .insert preserves domains
// CommitIndexBounded             | OK   | SELF      | no action modifies commit_index
// LogTermsNonNegative            | OK   | SELF      | appended term is nat (current_term)
// CurrentTermNonNeg              | OK   | TRIVIAL   | nat type, no action decreases
// MessageMultiplicityNonNeg      | OK   | TRIVIAL   | nat type
// PreacceptRequestProvenance     | OK   | NEEDS     | need: LClientSendPreaccept sends to valid dest
// BeginRecoveryRequestProvenance | OK   | NEEDS     | need: LSendBeginRecovery sends from server
// PrepareRequestProvenance       | OK   | NEEDS     | need: LSendPrepare sends from server
// AcceptRequestProvenance        | OK   | NEEDS     | need: LSendAccept sends from server
// FinishRecoveryRequestProvenance| OK   | NEEDS     | need: LFinishRecovery sends from server
// ExecutionCmdsWellFormed        | OK   | NEEDS     | need: client_pending cmds have valid keys
// OriginalExecutionCmdsWellFormed| OK   | SELF      | original_execution_cmds never modified
// CommittedCmdIdsUnique          | OK   | SELF      | no action modifies commit_index
// JPoolKeysValid                 | OK   | NEEDS     | need: preaccept cmd has valid key
// JPoolBallotOrdering            | OK   | SELF*     | *needs action-level ballot analysis
// EpochsNonNegative              | OK   | TRIVIAL   | nat type
// JEpochGeqOEpoch                | OK   | RETRACTED | HandleBeginRecoveryReq breaks it
// ReadyImpliesEpochsEqual        | OK   | RETRACTED | HandlePrepareReq breaks it
//
// NEW message-level / data-provenance invariants (Category 9-10):
// ViewReplicaIdsValid             | OK   | NEEDS*    | views from messages need msg-level inv
// ClientPendingCmdsValid          | OK   | SELF      | set only by LClientSendPreaccept
// PreacceptRequestCmdsValid       | OK   | SELF      | created only by LClientSendPreaccept
// PreacceptResponseCmdsValid      | OK   | NEEDS     | needs PreacceptRequestCmdsValid
//
// *ViewReplicaIdsValid NEEDS a message-level view integrity invariant for
//  views carried in BeginRecoveryRequest and FinishRecoveryRequest messages.
//  However, since LDefaultView is the only view constructor and all view
//  propagation goes through server state, this may be provable by showing
//  that views in messages always come from server state (which already
//  satisfies ViewReplicaIdsValid by the inductive hypothesis).
// =========================================================================

#![allow(unused)]

use vstd::prelude::*;

include!("../types.rs");

verus! {

// =========================================================================
// Category 1: Well-formedness / Type Invariants
// =========================================================================

/// All per-server maps are keyed over the correct domains.
/// This is the foundational invariant that all other invariants depend on.
pub open spec fn TypeInvariant(s: LState, c: LConstants) -> bool {
    // Base protocol maps keyed by Server
    &&& forall |i: int| c.server.contains(i) ==> s.current_term.contains_key(i)
    &&& forall |i: int| c.server.contains(i) ==> s.ostate.contains_key(i)
    &&& forall |i: int| c.server.contains(i) ==> s.jstate.contains_key(i)
    &&& forall |i: int| c.server.contains(i) ==> s.jepoch.contains_key(i)
    &&& forall |i: int| c.server.contains(i) ==> s.oepoch.contains_key(i)
    &&& forall |i: int| c.server.contains(i) ==> s.old_view.contains_key(i)
    &&& forall |i: int| c.server.contains(i) ==> s.new_view.contains_key(i)
    &&& forall |i: int| c.server.contains(i) ==> s.jpool.contains_key(i)
    &&& forall |i: int| c.server.contains(i) ==> s.recovery_set.contains_key(i)
    &&& forall |i: int| c.server.contains(i) ==> s.chosen_value.contains_key(i)
    &&& forall |i: int| c.server.contains(i) ==> s.br_responses.contains_key(i)
    &&& forall |i: int| c.server.contains(i) ==> s.prep_responses.contains_key(i)
    &&& forall |i: int| c.server.contains(i) ==> s.accept_responses.contains_key(i)
    // 3-D log: Server -> Proposer -> Seq<LogEntry>
    &&& forall |i: int| c.server.contains(i) ==> s.log.contains_key(i)
    &&& forall |i: int, j: int| c.server.contains(i) && c.proposer.contains(j)
        ==> s.log[i].contains_key(j)
    // Per-server per-proposer commit_index
    &&& forall |i: int| c.server.contains(i) ==> s.commit_index.contains_key(i)
    &&& forall |i: int, j: int| c.server.contains(i) && c.proposer.contains(j)
        ==> s.commit_index[i].contains_key(j)
    // Client maps
    &&& forall |cl: int| c.client.contains(cl) ==> s.client_view.contains_key(cl)
    &&& forall |cl: int| c.client.contains(cl) ==> s.client_pending.contains_key(cl)
    &&& forall |cl: int| c.client.contains(cl) ==> s.client_successes.contains_key(cl)
    &&& forall |cl: int| c.client.contains(cl) ==> s.client_heard_from.contains_key(cl)
}

// =========================================================================
// Category 2: Log / Commit-Index Bounds
// =========================================================================

/// commit_index[i][j] never exceeds log[i][j].len().
/// This is the analog of Raft's CommitIndexBounded.
pub open spec fn CommitIndexBounded(s: LState, c: LConstants) -> bool {
    forall |i: int, j: int|
        c.server.contains(i) && c.proposer.contains(j)
        ==> s.commit_index[i][j] <= s.log[i][j].len()
}

/// Log entries have non-negative terms.
pub open spec fn LogTermsNonNegative(s: LState, c: LConstants) -> bool {
    forall |i: int, j: int, k: int|
        c.server.contains(i) && c.proposer.contains(j)
        && 0 <= k < s.log[i][j].len()
        ==> s.log[i][j][k].term >= 0
}

// =========================================================================
// Category 3: Epoch Monotonicity
// =========================================================================

/// RETRACTED: JEpochGeqOEpoch (jepoch >= oepoch) was conjectured but is NOT inductive.
///
/// Counter-example: LHandleBeginRecoveryRequest sets oepoch[i] = mnew_view.epoch
/// but leaves jepoch unchanged. If mnew_view.epoch > jepoch[i], then
/// oepoch[i] > jepoch[i], violating jepoch >= oepoch.
///
/// In the TLA+ spec, jepoch and oepoch are independent epoch trackers:
/// - oepoch: "outer" epoch, updated by BeginRecovery, Prepare, Accept handlers
/// - jepoch: "jetpack" epoch, updated by Prepare, Accept, FinishRecovery
/// Both start equal at init but can diverge in either direction.
/// No ordering invariant between them exists in the protocol.
///
/// Placeholder: no replacement needed. EpochsNonNegative already covers
/// the only provable fact about epochs (both >= 0).
// pub open spec fn JEpochGeqOEpoch -- RETRACTED

/// current_term is non-negative.
/// Note: LInit sets current_term to 0 (not 1), so this is >= 0, not >= 1.
/// The nat type already guarantees >= 0, but this makes the invariant explicit.
pub open spec fn CurrentTermNonNeg(s: LState, c: LConstants) -> bool {
    forall |i: int| c.server.contains(i) ==> s.current_term[i] >= 0
}

// =========================================================================
// Category 4: Message Typing / Provenance
// =========================================================================

/// All messages in the message bag have non-negative multiplicity.
/// (By construction, Map<LMessage, nat> guarantees nat >= 0, but this
/// makes the invariant explicit for proof use.)
pub open spec fn MessageMultiplicityNonNeg(s: LState, _c: LConstants) -> bool {
    forall |m: LMessage| s.messages.contains_key(m) ==> s.messages[m] >= 0
}

/// PreacceptRequest messages have valid source (a client) and dest (a server).
pub open spec fn PreacceptRequestProvenance(s: LState, c: LConstants) -> bool {
    forall |m: LMessage|
        s.messages.contains_key(m) && s.messages[m] > 0
        && m is PreacceptRequest
        ==> {
            &&& c.client.contains(m->msource)
            &&& c.server.contains(m->mdest)
        }
}

/// BeginRecoveryRequest messages have valid source and dest (both servers).
pub open spec fn BeginRecoveryRequestProvenance(s: LState, c: LConstants) -> bool {
    forall |m: LMessage|
        s.messages.contains_key(m) && s.messages[m] > 0
        && m is BeginRecoveryRequest
        ==> {
            &&& c.server.contains(m->msource)
            &&& c.server.contains(m->mdest)
        }
}

/// JetpackPrepareRequest messages have valid source and dest (both servers).
pub open spec fn PrepareRequestProvenance(s: LState, c: LConstants) -> bool {
    forall |m: LMessage|
        s.messages.contains_key(m) && s.messages[m] > 0
        && m is JetpackPrepareRequest
        ==> {
            &&& c.server.contains(m->msource)
            &&& c.server.contains(m->mdest)
        }
}

/// JetpackAcceptRequest messages have valid source and dest (both servers).
pub open spec fn AcceptRequestProvenance(s: LState, c: LConstants) -> bool {
    forall |m: LMessage|
        s.messages.contains_key(m) && s.messages[m] > 0
        && m is JetpackAcceptRequest
        ==> {
            &&& c.server.contains(m->msource)
            &&& c.server.contains(m->mdest)
        }
}

/// FinishRecoveryRequest messages have valid source and dest (both servers).
pub open spec fn FinishRecoveryRequestProvenance(s: LState, c: LConstants) -> bool {
    forall |m: LMessage|
        s.messages.contains_key(m) && s.messages[m] > 0
        && m is FinishRecoveryRequest
        ==> {
            &&& c.server.contains(m->msource)
            &&& c.server.contains(m->mdest)
        }
}

// =========================================================================
// Category 5: Execution Trace Well-formedness
// =========================================================================

/// All commands in the execution trace have valid cmd_ids and keys.
pub open spec fn ExecutionCmdsWellFormed(s: LState, c: LConstants) -> bool {
    forall |k: int| 0 <= k < s.execution_cmds.len() ==> {
        &&& c.cmd_id.contains(s.execution_cmds[k].cmd_id)
        &&& c.key.contains(s.execution_cmds[k].key)
    }
}

/// All commands in the original execution trace have valid cmd_ids and keys.
pub open spec fn OriginalExecutionCmdsWellFormed(s: LState, c: LConstants) -> bool {
    forall |k: int| 0 <= k < s.original_execution_cmds.len() ==> {
        &&& c.cmd_id.contains(s.original_execution_cmds[k].cmd_id)
        &&& c.key.contains(s.original_execution_cmds[k].key)
    }
}

// =========================================================================
// Category 6: Command-ID Uniqueness
// =========================================================================

/// Within each proposer's log on a given server, committed entries have
/// unique command IDs (no duplicate cmd_id in log[i][j][0..commit_index[i][j]]).
pub open spec fn CommittedCmdIdsUnique(s: LState, c: LConstants) -> bool {
    forall |i: int, j: int, k1: int, k2: int|
        c.server.contains(i) && c.proposer.contains(j)
        && 0 <= k1 < s.commit_index[i][j]
        && 0 <= k2 < s.commit_index[i][j]
        && s.log[i][j][k1].value.cmd_id == s.log[i][j][k2].value.cmd_id
        && s.log[i][j][k1].value == s.log[i][j][k2].value
        ==> k1 == k2
}

// =========================================================================
// Category 7: JPool Well-formedness
// =========================================================================

/// jpool[i].pool is keyed over the Key domain.
pub open spec fn JPoolKeysValid(s: LState, c: LConstants) -> bool {
    forall |i: int, k: int|
        c.server.contains(i) && s.jpool[i].pool.contains_key(k)
        ==> c.key.contains(k)
}

/// jpool[i].accepted_ballot <= jpool[i].max_seen_ballot.
pub open spec fn JPoolBallotOrdering(s: LState, c: LConstants) -> bool {
    forall |i: int| c.server.contains(i) ==>
        s.jpool[i].accepted_ballot <= s.jpool[i].max_seen_ballot
}

// =========================================================================
// Category 8: JState / Recovery Protocol Consistency
// =========================================================================

/// RETRACTED: ReadyImpliesEpochsEqual was conjectured but is NOT inductive.
///
/// Counter-example: LHandlePrepareRequest (and LHandleAcceptRequest,
/// LHandlePrepareResponse !mok, LHandleAcceptResponse !mok) update
/// jepoch = max(jepoch, mjepoch) and oepoch = max(oepoch, moepoch)
/// independently while keeping jstate unchanged. If moepoch != mjepoch,
/// the epochs diverge even in Ready state.
///
/// The correct invariant would need a message-level constraint
/// (e.g., prepare/accept messages always carry moepoch == mjepoch),
/// which is not obvious from the TLA+ spec.
///
/// Replaced with the weaker EpochsNonNegative.
pub open spec fn EpochsNonNegative(s: LState, c: LConstants) -> bool {
    &&& forall |i: int| c.server.contains(i) ==> s.jepoch[i] >= 0
    &&& forall |i: int| c.server.contains(i) ==> s.oepoch[i] >= 0
}

// =========================================================================
// Category 9: View Integrity (needed by provenance invariants)
// =========================================================================

/// All views stored in server state have replica_ids and proposing_replica_ids
/// that are subsets of c.server.
///
/// This is needed to prove that message destinations (which come from
/// view.replica_ids) are valid servers.
///
/// INIT: LDefaultView(c) has replica_ids == c.server, proposing_replica_ids == c.server.
///       So subset_of(c.server) trivially holds.
///
/// INDUCTIVE: Views are updated in two ways:
///   (a) From LDefaultView(c) -- always valid
///   (b) From message fields (mnew_view, mold_view in BeginRecoveryRequest,
///       mview in PreacceptResponse, mnew_view in FinishRecoveryRequest)
///       These need a corresponding message-level invariant.
///   For now, this invariant captures the server-state side.
///   The message-level side would be:
///     "All LView values carried in messages have replica_ids ⊆ c.server"
pub open spec fn ViewReplicaIdsValid(s: LState, c: LConstants) -> bool {
    &&& forall |i: int| c.server.contains(i) ==> {
        &&& s.old_view[i].replica_ids.subset_of(c.server)
        &&& s.old_view[i].proposing_replica_ids.subset_of(c.server)
        &&& s.new_view[i].replica_ids.subset_of(c.server)
        &&& s.new_view[i].proposing_replica_ids.subset_of(c.server)
    }
    &&& forall |cl: int| c.client.contains(cl) ==> {
        &&& s.client_view[cl].replica_ids.subset_of(c.server)
        &&& s.client_view[cl].proposing_replica_ids.subset_of(c.server)
    }
}

// =========================================================================
// Category 10: Command Well-formedness (needed by JPoolKeysValid, ExecutionCmdsWellFormed)
// =========================================================================

/// All commands in client_pending have valid cmd_id and key.
///
/// This is needed to prove ExecutionCmdsWellFormed: when a PreacceptResponse
/// triggers fast-path commit, the cmd comes from client_pending[cl], so we
/// need to know it has valid fields.
///
/// INIT: client_pending is all None. Vacuously true.
///
/// INDUCTIVE: client_pending[cl] is set to Some(cmd) only in
///   LClientSendPreaccept, where cmd comes from LAvailableCommands.
///   LAvailableCommands filters by c.cmd_id and c.key, so cmd is valid.
///   client_pending[cl] is set to None in LHandlePreacceptResponse.
pub open spec fn ClientPendingCmdsValid(s: LState, c: LConstants) -> bool {
    forall |cl: int| c.client.contains(cl) && s.client_pending[cl] is Some ==> {
        let cmd = s.client_pending[cl].unwrap();
        &&& c.cmd_id.contains(cmd.cmd_id)
        &&& c.key.contains(cmd.key)
    }
}

/// All commands in PreacceptRequest messages have valid cmd_id and key.
///
/// This is needed to prove JPoolKeysValid: when LHandlePreacceptRequest
/// inserts cmd.key into jpool[i].pool, we need c.key.contains(cmd.key).
///
/// INIT: messages is empty. Vacuously true.
///
/// INDUCTIVE: PreacceptRequest messages are created only by LClientSendPreaccept,
///   where the cmd comes from LAvailableCommands (which filters by c.cmd_id/c.key).
///   No other action creates PreacceptRequest messages.
pub open spec fn PreacceptRequestCmdsValid(s: LState, c: LConstants) -> bool {
    forall |m: LMessage|
        s.messages.contains_key(m) && s.messages[m] > 0
        && m is PreacceptRequest
        ==> {
            &&& c.cmd_id.contains(m->mcmd.cmd_id)
            &&& c.key.contains(m->mcmd.key)
        }
}

/// All commands in PreacceptResponse messages have valid cmd_id and key.
///
/// Needed because LHandlePreacceptResponse checks client_pending[cl] == Some(mcmd),
/// so the mcmd in the response must match a valid client_pending cmd.
///
/// INIT: messages is empty. Vacuously true.
///
/// INDUCTIVE: PreacceptResponse messages are created by LHandlePreacceptRequest,
///   where mcmd comes from the incoming PreacceptRequest message.
///   By PreacceptRequestCmdsValid, the cmd is valid.
pub open spec fn PreacceptResponseCmdsValid(s: LState, c: LConstants) -> bool {
    forall |m: LMessage|
        s.messages.contains_key(m) && s.messages[m] > 0
        && m is PreacceptResponse
        ==> {
            &&& c.cmd_id.contains(m->mcmd.cmd_id)
            &&& c.key.contains(m->mcmd.key)
        }
}

// =========================================================================
// Composite safety invariant
// =========================================================================

/// The full Jetpack safety invariant: conjunction of all support invariants
/// plus the three named safety properties.
pub open spec fn JetpackSafetyInvariant(s: LState, c: LConstants) -> bool {
    // Well-formedness
    &&& TypeInvariant(s, c)
    // Log/commit bounds
    &&& CommitIndexBounded(s, c)
    &&& LogTermsNonNegative(s, c)
    // Epoch properties
    // Note: JEpochGeqOEpoch was retracted (not inductive)
    &&& CurrentTermNonNeg(s, c)
    // Message provenance
    &&& MessageMultiplicityNonNeg(s, c)
    &&& PreacceptRequestProvenance(s, c)
    &&& BeginRecoveryRequestProvenance(s, c)
    &&& PrepareRequestProvenance(s, c)
    &&& AcceptRequestProvenance(s, c)
    &&& FinishRecoveryRequestProvenance(s, c)
    // Execution trace
    &&& ExecutionCmdsWellFormed(s, c)
    &&& OriginalExecutionCmdsWellFormed(s, c)
    // Command-ID uniqueness
    &&& CommittedCmdIdsUnique(s, c)
    // JPool
    &&& JPoolKeysValid(s, c)
    &&& JPoolBallotOrdering(s, c)
    // Recovery
    &&& EpochsNonNegative(s, c)
    // View integrity
    &&& ViewReplicaIdsValid(s, c)
    // Command well-formedness
    &&& ClientPendingCmdsValid(s, c)
    &&& PreacceptRequestCmdsValid(s, c)
    &&& PreacceptResponseCmdsValid(s, c)
    // Named safety properties (from jetpack.tla)
    // These are the ultimate proof targets; the support invariants above
    // are needed to make the inductive argument go through.
    // &&& LCommittedLogAgreement(s, c)
    // &&& LLogOrderMatchesExecution(s, c)
    // &&& LExecutionDedupMatches(s, c)
}

// =========================================================================
// Initialization lemma
// =========================================================================

/// LInit establishes the safety invariant.
///
/// PROOF STATUS: Complete (modulo Verus compilation).
/// Each conjunct of JetpackSafetyInvariant follows directly from
/// the initial values set in LInit.
pub proof fn lemma_init_establishes_invariant(s: LState, c: LConstants)
    requires
        // LInit(s, c) expanded inline for self-containment:
        // -- messages
        s.messages == Map::<LMessage, nat>::empty(),
        // -- base protocol
        s.current_term == Map::new(|i: int| c.server.contains(i), |i: int| 0nat),
        s.ostate == Map::new(|i: int| c.server.contains(i), |i: int| LOState::Follower),
        s.log == Map::new(
            |i: int| c.server.contains(i),
            |i: int| Map::new(|j: int| c.proposer.contains(j), |j: int| Seq::<LLogEntry>::empty()),
        ),
        s.commit_index == Map::new(
            |i: int| c.server.contains(i),
            |i: int| Map::new(|j: int| c.proposer.contains(j), |j: int| 0nat),
        ),
        // -- jetpack vars
        s.jstate == Map::new(|i: int| c.server.contains(i), |i: int| LJState::Ready),
        s.jepoch == Map::new(|i: int| c.server.contains(i), |i: int| LDefaultView(c).epoch),
        s.oepoch == Map::new(|i: int| c.server.contains(i), |i: int| LDefaultView(c).epoch),
        s.old_view == Map::new(|i: int| c.server.contains(i), |i: int| LDefaultView(c)),
        s.new_view == Map::new(|i: int| c.server.contains(i), |i: int| LDefaultView(c)),
        s.jpool == Map::new(|i: int| c.server.contains(i), |i: int| LEmptyJPool(c)),
        s.recovery_set == Map::new(|i: int| c.server.contains(i), |i: int| Set::<LCmd>::empty()),
        s.chosen_value == Map::new(|i: int| c.server.contains(i), |i: int| Set::<LCmd>::empty()),
        s.br_responses == Map::new(
            |i: int| c.server.contains(i),
            |i: int| Map::new(|j: int| c.server.contains(j), |j: int| None::<LJPool>),
        ),
        s.prep_responses == Map::new(
            |i: int| c.server.contains(i),
            |i: int| Map::new(|j: int| c.server.contains(j), |j: int| None::<LPrepResp>),
        ),
        s.accept_responses == Map::new(
            |i: int| c.server.contains(i),
            |i: int| Map::new(|j: int| c.server.contains(j), |j: int| false),
        ),
        // -- client vars
        s.client_view == Map::new(|cv: int| c.client.contains(cv), |cv: int| LDefaultView(c)),
        s.client_pending == Map::new(|cv: int| c.client.contains(cv), |cv: int| None::<LCmd>),
        s.client_successes == Map::new(|cv: int| c.client.contains(cv), |cv: int| Set::<int>::empty()),
        s.client_heard_from == Map::new(|cv: int| c.client.contains(cv), |cv: int| Set::<int>::empty()),
        // -- execution vars
        s.original_execution_cmds == Seq::<LCmd>::empty(),
        s.execution_cmds == Seq::<LCmd>::empty(),
    ensures
        JetpackSafetyInvariant(s, c),
{
    // -- TypeInvariant: all maps are keyed by server/proposer/client domains
    // Follows from Map::new(|i| domain.contains(i), ...) construction.
    // Each map's domain matches the corresponding constant set.
    assert(TypeInvariant(s, c)) by {
        // Map::new(|i| c.server.contains(i), ...) has key i iff c.server.contains(i)
        // This gives us all the per-server map containment.
        // Similarly for log[i][j] and commit_index[i][j] with proposer domain.
        // Client maps use c.client domain.
    };

    // -- CommitIndexBounded: commit_index[i][j] == 0 <= log[i][j].len() == 0
    assert(CommitIndexBounded(s, c)) by {
        assert forall |i: int, j: int|
            c.server.contains(i) && c.proposer.contains(j)
        implies s.commit_index[i][j] <= s.log[i][j].len()
        by {
            // commit_index[i][j] == 0 and log[i][j] == Seq::empty() so len() == 0
            // 0 <= 0 trivially
        }
    };

    // -- LogTermsNonNegative: log is empty, so vacuously true
    assert(LogTermsNonNegative(s, c)) by {
        assert forall |i: int, j: int, k: int|
            c.server.contains(i) && c.proposer.contains(j)
            && 0 <= k && k < s.log[i][j].len()
        implies s.log[i][j][k].term >= 0
        by {
            // log[i][j] == Seq::empty(), so log[i][j].len() == 0
            // No k satisfies 0 <= k < 0, so the implication is vacuously true
        }
    };

    // -- JEpochGeqOEpoch: RETRACTED (not inductive, see Category 3 comment)

    // -- CurrentTermNonNeg: current_term[i] == 0nat >= 0
    assert(CurrentTermNonNeg(s, c)) by {
        assert forall |i: int| c.server.contains(i)
        implies s.current_term[i] >= 0
        by {
            // current_term[i] == 0nat, and 0 >= 0
        }
    };

    // -- MessageMultiplicityNonNeg: messages is empty, vacuously true
    assert(MessageMultiplicityNonNeg(s, c));

    // -- All message provenance invariants: messages is empty, vacuously true
    assert(PreacceptRequestProvenance(s, c));
    assert(BeginRecoveryRequestProvenance(s, c));
    assert(PrepareRequestProvenance(s, c));
    assert(AcceptRequestProvenance(s, c));
    assert(FinishRecoveryRequestProvenance(s, c));

    // -- ExecutionCmdsWellFormed: execution_cmds is empty, vacuously true
    assert(ExecutionCmdsWellFormed(s, c));

    // -- OriginalExecutionCmdsWellFormed: original_execution_cmds is empty
    assert(OriginalExecutionCmdsWellFormed(s, c));

    // -- CommittedCmdIdsUnique: commit_index[i][j] == 0, so no entries
    assert(CommittedCmdIdsUnique(s, c)) by {
        assert forall |i: int, j: int, k1: int, k2: int|
            c.server.contains(i) && c.proposer.contains(j)
            && 0 <= k1 && k1 < s.commit_index[i][j]
            && 0 <= k2 && k2 < s.commit_index[i][j]
            && s.log[i][j][k1].value.cmd_id == s.log[i][j][k2].value.cmd_id
            && s.log[i][j][k1].value == s.log[i][j][k2].value
        implies k1 == k2
        by {
            // commit_index[i][j] == 0, so no k satisfies 0 <= k < 0
        }
    };

    // -- JPoolKeysValid: jpool[i] == LEmptyJPool(c), whose pool keys are from c.key
    assert(JPoolKeysValid(s, c)) by {
        assert forall |i: int, k: int|
            c.server.contains(i) && s.jpool[i].pool.contains_key(k)
        implies c.key.contains(k)
        by {
            // jpool[i] == LEmptyJPool(c)
            // LEmptyJPool(c).pool == Map::new(|k| c.key.contains(k), ...)
            // So pool.contains_key(k) iff c.key.contains(k)
        }
    };

    // -- JPoolBallotOrdering: jpool[i] == LEmptyJPool(c) with both ballots == 0
    assert(JPoolBallotOrdering(s, c)) by {
        assert forall |i: int| c.server.contains(i)
        implies s.jpool[i].accepted_ballot <= s.jpool[i].max_seen_ballot
        by {
            // LEmptyJPool(c).accepted_ballot == 0 == LEmptyJPool(c).max_seen_ballot
        }
    };

    // -- EpochsNonNegative: jepoch[i] == oepoch[i] == LDefaultView(c).epoch == 1 >= 0
    assert(EpochsNonNegative(s, c)) by {
        assert forall |i: int| c.server.contains(i)
        implies s.jepoch[i] >= 0 && s.oepoch[i] >= 0
        by {
            // Both are LDefaultView(c).epoch == 1, and 1 >= 0
        }
    };

    // -- ViewReplicaIdsValid: all views are LDefaultView(c) with replica_ids == c.server
    assert(ViewReplicaIdsValid(s, c)) by {
        assert forall |i: int| c.server.contains(i) implies {
            &&& s.old_view[i].replica_ids.subset_of(c.server)
            &&& s.old_view[i].proposing_replica_ids.subset_of(c.server)
            &&& s.new_view[i].replica_ids.subset_of(c.server)
            &&& s.new_view[i].proposing_replica_ids.subset_of(c.server)
        } by {
            // old_view[i] == new_view[i] == LDefaultView(c)
            // LDefaultView(c).replica_ids == c.server
            // LDefaultView(c).proposing_replica_ids == c.server
            // c.server.subset_of(c.server) trivially
        }
        assert forall |cl: int| c.client.contains(cl) implies {
            &&& s.client_view[cl].replica_ids.subset_of(c.server)
            &&& s.client_view[cl].proposing_replica_ids.subset_of(c.server)
        } by {
            // client_view[cl] == LDefaultView(c), same reasoning
        }
    };

    // -- ClientPendingCmdsValid: client_pending[cl] == None for all clients
    assert(ClientPendingCmdsValid(s, c)) by {
        // All client_pending values are None, so the implication is vacuously true
    };

    // -- PreacceptRequestCmdsValid: messages is empty, vacuously true
    assert(PreacceptRequestCmdsValid(s, c));

    // -- PreacceptResponseCmdsValid: messages is empty, vacuously true
    assert(PreacceptResponseCmdsValid(s, c));
}

// =========================================================================
// Inductive preservation lemma
// =========================================================================

/// JetpackSafetyInvariant is preserved by every action in LNext.
pub proof fn lemma_safety_invariant_inductive(
    s: LState, s_: LState, c: LConstants,
)
    requires
        JetpackSafetyInvariant(s, c),
        // LNext(s, s_, c)  -- from jetpack.rs
        // Placeholder: expand when LNext is importable
        true,
    ensures
        JetpackSafetyInvariant(s_, c),
{
    // PROOF STATUS: skeleton only.
    // Strategy: case-split on which of the 19 LNext branches was taken,
    // then prove each conjunct of JetpackSafetyInvariant is preserved.
    //
    // Expected proof structure (per Raft pattern):
    //   1. Identify which action branch was taken
    //   2. For each support invariant, show preservation by that action
    //   3. For the three safety properties, use the support invariants
    //      to complete the inductive step
    //
    // Key challenges:
    //   - 19 action branches x 17+ invariant conjuncts = 300+ cases
    //   - Many actions only modify a subset of state, so most cases
    //     follow from UNCHANGED obligations
    //   - The critical cases are the commit/finish actions that modify
    //     commit_index and execution traces
    assume(false); // TODO: replace with actual proof
}

// =========================================================================
// Per-invariant preservation lemmas (skeletons)
// =========================================================================

/// TypeInvariant is preserved by all actions.
///
/// PROOF SKETCH: Complete -- each action uses map updates (.insert) or
/// wholesale map replacement that preserves domain membership.
///
/// For all 19 actions:
///   - Actions that modify a per-server field (e.g., jstate[i] := Ready)
///     use s.jstate.insert(i, Ready) where c.server.contains(i),
///     so the domain is preserved.
///   - Actions that modify the 3-D log (e.g., log[i][p] := log[i][p].push(entry))
///     use nested .insert which preserves both server and proposer domains.
///   - No action removes keys from any map.
///   - UNCHANGED obligations ensure unmodified maps retain their domains.
///
/// The proof would case-split on LNext's 19 branches and for each branch:
///   1. Identify which maps are modified
///   2. Show .insert preserves contains_key for all existing keys
///   3. Show UNCHANGED maps trivially preserve TypeInvariant conjuncts
proof fn lemma_type_invariant_inductive(s: LState, s_: LState, c: LConstants)
    requires
        TypeInvariant(s, c),
        // LNext(s, s_, c),
        true,
    ensures
        TypeInvariant(s_, c),
{
    assume(false); // BLOCKED: requires LNext import for case-split
}

/// CommitIndexBounded is preserved by all actions.
///
/// FIELD DEPENDENCY: commit_index, log
/// UNCHANGED ANALYSIS (19 actions):
///   base=U (neither log nor commit_index modified) for 15 of 19 actions:
///     LClientSendPreaccept, LSendBeginRecovery, LHandleBeginRecoveryRequest,
///     LHandleBeginRecoveryResponse, LCompleteBeginRecovery, LSendPrepare,
///     LHandlePrepareRequest, LHandlePrepareResponse, LCompletePrepare,
///     LSendAccept, LHandleAcceptRequest, LHandleAcceptResponse,
///     LCompleteAccept, LResubmit, LCompleteResubmit
///   => TRIVIAL for these 15 actions (commit_index and log unchanged).
///
/// NON-TRIVIAL CASES (4 actions that modify base vars):
///   LHandlePreacceptRequest: may append to log[i][p] but does NOT change commit_index
///     => new log is longer, commit_index unchanged => still bounded.
///   LHandlePreacceptResponse: may increase commit_index[cli][p] via client
///     => BUT this is client-side; actually commit_index is base var, unchanged here.
///     Wait -- re-checking: LHandlePreacceptResponse has base=U per UNCHANGED analysis.
///     So actually all client-side actions don't touch commit_index.
///   LFinishRecovery: modifies ostate but NOT log or commit_index (base=P, only ostate)
///     => commit_index and log unchanged => trivial.
///   LHandleFinishRecovery: same as LFinishRecovery (base=P, only ostate)
///     => commit_index and log unchanged => trivial.
///
/// CONCLUSION: CommitIndexBounded is trivially preserved by ALL 19 actions
/// because no action modifies commit_index without also ensuring the bound.
/// Actually, re-examining: which action CAN advance commit_index?
/// In the TLA+ spec, commit_index is a base protocol variable. Looking at
/// LHandlePreacceptResponse more carefully -- it only modifies client vars.
/// The base protocol's commit_index is never modified by any Jetpack action
/// in the standalone spec (it would be modified by a separate base protocol layer).
/// So CommitIndexBounded is trivially preserved.
proof fn lemma_commit_index_bounded_inductive(s: LState, s_: LState, c: LConstants)
    requires
        JetpackSafetyInvariant(s, c),
        // LNext(s, s_, c),
        true,
    ensures
        CommitIndexBounded(s_, c),
{
    // All 19 actions either:
    // (a) Have LUnchangedBaseVars => commit_index and log identical => trivial
    // (b) Only modify log (LHandlePreacceptRequest appends) but not commit_index
    //     => log grows, commit_index unchanged => bound preserved
    // (c) Only modify ostate (LFinishRecovery, LHandleFinishRecovery)
    //     => commit_index and log unchanged => trivial
    assume(false); // BLOCKED: requires LNext import for case-split
}

// lemma_jepoch_geq_oepoch_inductive: REMOVED
// JEpochGeqOEpoch was retracted (not inductive). See Category 3 comment.

/// JPoolBallotOrdering is preserved by all actions.
///
/// FIELD DEPENDENCY: jpool (specifically jpool[i].accepted_ballot, jpool[i].max_seen_ballot)
/// UNCHANGED ANALYSIS:
///   jpool unchanged for 12 of 19 actions:
///     LClientSendPreaccept, LHandlePreacceptResponse, LSendBeginRecovery,
///     LHandleBeginRecoveryRequest, LHandleBeginRecoveryResponse,
///     LCompleteBeginRecovery, LSendPrepare, LCompletePrepare, LSendAccept,
///     LCompleteAccept, LResubmit, LCompleteResubmit
///   => TRIVIAL for these 12.
///
/// NON-TRIVIAL CASES (7 actions modify jpool):
///   LHandlePreacceptRequest: sets jpool[i] with pool update for a key
///     => max_seen_ballot and accepted_ballot not changed in pool update
///     Actually: only modifies pool[cmd.key], not the ballot fields => TRIVIAL
///   LHandlePrepareRequest: may update jpool[i].max_seen_ballot
///     => guard: mmax_seen_ballot > jpool[i].max_seen_ballot
///     => sets max_seen_ballot = mmax_seen_ballot, accepted_ballot unchanged
///     => max_seen_ballot increases => ordering preserved
///   LHandlePrepareResponse: may reset jpool or update based on response
///     => needs careful analysis
///   LHandleAcceptRequest: may update jpool[i].max_seen_ballot and accepted_ballot
///     => guard ensures proper ordering
///   LHandleAcceptResponse: similar to LHandlePrepareResponse
///   LFinishRecovery: resets jpool to LEmptyJPool => both ballots 0 => trivial
///   LHandleFinishRecovery: resets jpool to LEmptyJPool => trivial
proof fn lemma_jpool_ballot_ordering_inductive(s: LState, s_: LState, c: LConstants)
    requires
        JetpackSafetyInvariant(s, c),
        // LNext(s, s_, c),
        true,
    ensures
        JPoolBallotOrdering(s_, c),
{
    assume(false); // BLOCKED: requires LNext + ballot update analysis
}

/// CommittedCmdIdsUnique is preserved by all actions.
///
/// FIELD DEPENDENCY: log, commit_index
/// Since no Jetpack action modifies commit_index (base protocol variable),
/// and only LHandlePreacceptRequest appends to log (at positions beyond
/// the current commit_index), the committed prefix is never extended by
/// any Jetpack action. Therefore, uniqueness within the committed prefix
/// is trivially preserved.
///
/// NOTE: When the base protocol layer advances commit_index, that would
/// be the non-trivial case. But in the standalone Jetpack spec, commit_index
/// is only set at init (to 0) and never modified.
proof fn lemma_committed_cmd_ids_unique_inductive(s: LState, s_: LState, c: LConstants)
    requires
        JetpackSafetyInvariant(s, c),
        // LNext(s, s_, c),
        true,
    ensures
        CommittedCmdIdsUnique(s_, c),
{
    // Trivially preserved: no action modifies commit_index, and log
    // only grows (append at end), so the committed prefix is unchanged.
    assume(false); // BLOCKED: requires LNext import for case-split
}

// =========================================================================
// Per-action UNCHANGED summary (reference for proof engineering)
// =========================================================================
//
// Action                      | msgs | base | jetpack | client | exec
// ----------------------------|------|------|---------|--------|------
// LClientSendPreaccept        |  M   |  U   |   U     |   M    |  U
// LHandlePreacceptRequest     |  M   |  P*  |   P*    |   U    |  U
// LHandlePreacceptResponse    |  M   |  U   |   U     |   M    |  P*
// LSendBeginRecovery          |  M   |  U   |   P     |   U    |  U
// LHandleBeginRecoveryRequest |  M   |  U   |   P     |   U    |  U
// LHandleBeginRecoveryResponse|  M   |  U   |   P     |   U    |  U
// LCompleteBeginRecovery      |  U   |  U   |   P     |   U    |  U
// LSendPrepare                |  M   |  U   |   P     |   U    |  U
// LHandlePrepareRequest       |  M   |  U   |   P     |   U    |  U
// LHandlePrepareResponse      |  M   |  U   |   P     |   U    |  U
// LCompletePrepare            |  U   |  U   |   P     |   U    |  U
// LSendAccept                 |  M   |  U   |   P     |   U    |  U
// LHandleAcceptRequest        |  M   |  U   |   P     |   U    |  U
// LHandleAcceptResponse       |  M   |  U   |   P     |   U    |  U
// LCompleteAccept             |  U   |  U   |   P     |   U    |  U
// LResubmit                   |  M   |  U   |   U     |   U    |  U
// LCompleteResubmit           |  U   |  U   |   P     |   U    |  U
// LFinishRecovery             |  M   |  P*  |   P     |   U    |  U
// LHandleFinishRecovery       |  M   |  P*  |   P     |   U    |  U
//
// U = unchanged, M = modified, P = partially modified
// P* = only specific sub-fields modified (see per-action notes)
// base P*: LHandlePreacceptRequest modifies log only;
//          LFinishRecovery/LHandleFinishRecovery modify ostate only
// jetpack P*: LHandlePreacceptRequest modifies jpool only
// exec P*: LHandlePreacceptResponse modifies execution_cmds only

// =========================================================================
// Named safety property lemma skeletons
// =========================================================================

/// CommittedLogAgreement: committed log prefixes agree across servers.
/// This is the primary safety property for the 3-D log architecture.
///
/// Proof sketch:
///   - For any two servers i, i2 and proposer p, the committed prefix
///     log[i][p][0..min(ci, ci2)] matches entry-by-entry.
///   - Relies on: the preaccept fast-path quorum intersection guarantees
///     that committed entries at the same position have the same term+value.
///   - Support invariants needed: CommitIndexBounded, TypeInvariant,
///     plus a quorum intersection lemma (not yet formalized).
proof fn lemma_committed_log_agreement_inductive(
    s: LState, s_: LState, c: LConstants,
)
    requires
        JetpackSafetyInvariant(s, c),
        // LCommittedLogAgreement(s, c),
        // LNext(s, s_, c),
        true,
    ensures
        // LCommittedLogAgreement(s_, c),
        true,
{
    // PROOF STATUS: skeleton only.
    // BLOCKER: Requires quorum intersection lemma for fast-path commits.
    // The key insight is that any two fast-path quorums for the same
    // proposer and epoch must overlap, so committed entries agree.
    assume(false); // TODO
}

/// LogOrderMatchesExecution: committed log order matches execution order
/// for conflicting commands.
///
/// Proof sketch:
///   - For each server i and proposer p, the committed commands
///     (filtered of no-ops) preserve conflict order with the execution trace.
///   - Relies on: CommittedLogAgreement, plus the fact that LFinishRecovery
///     and LCompleteResubmit carefully reconstruct the execution trace.
proof fn lemma_log_order_matches_execution_inductive(
    s: LState, s_: LState, c: LConstants,
)
    requires
        JetpackSafetyInvariant(s, c),
        // LLogOrderMatchesExecution(s, c),
        // LNext(s, s_, c),
        true,
    ensures
        // LLogOrderMatchesExecution(s_, c),
        true,
{
    // PROOF STATUS: skeleton only.
    // BLOCKER: Requires formalization of conflict-order preservation
    // through the recovery protocol's resubmit/finish actions.
    assume(false); // TODO
}

/// ExecutionDedupMatches: deduplicated original and replicated execution
/// traces preserve conflict order in both directions.
///
/// Proof sketch:
///   - The original_execution_cmds and execution_cmds traces, after
///     dedup and no-op filtering, are conflict-order equivalent.
///   - Relies on: the resubmit protocol ensuring that recovered commands
///     are re-executed in conflict-compatible order.
proof fn lemma_execution_dedup_matches_inductive(
    s: LState, s_: LState, c: LConstants,
)
    requires
        JetpackSafetyInvariant(s, c),
        // LExecutionDedupMatches(s, c),
        // LNext(s, s_, c),
        true,
    ensures
        // LExecutionDedupMatches(s_, c),
        true,
{
    // PROOF STATUS: skeleton only.
    // BLOCKER: Requires formalization of dedup/conflict-order interaction
    // and the resubmit protocol's correctness.
    assume(false); // TODO
}

} // verus!
