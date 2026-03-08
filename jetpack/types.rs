// Jetpack protocol types -- hand-translated from jetpack/jetpack.tla
//
// This file defines all shared types for the Jetpack plugin consensus protocol.
// See jetpack/translation_audit.md for the complete TLA+ -> Verus mapping.

use vstd::prelude::*;

verus! {

// ---- Jetpack protocol states ----

#[derive(PartialEq, Eq, Structural)]
pub enum LJState {
    Ready,
    Recovery,
    AfterBeginRecovery,
    AfterPrepare,
    AfterAccept,
    AfterResubmit,
}

// ---- Base protocol server states ----

#[derive(PartialEq, Eq, Structural)]
pub enum LOState {
    Follower,
    Candidate,
    Leader,
    ToBeLeader,
}

// ---- Command type ----
// TLA+: Commands == { [cmd_id |-> id, key |-> k] : id \in CmdId, k \in Key }

#[derive(PartialEq, Eq, Structural)]
pub struct LCmd {
    pub cmd_id: int,
    pub key: int,
}

// ---- Log entry ----
// TLA+: implicit from log usage: [term |-> Nat, value |-> Cmd]

#[derive(PartialEq, Eq, Structural)]
pub struct LLogEntry {
    pub term: nat,
    pub value: LCmd,
}

// ---- View ----
// TLA+: View == [epoch: Nat, proposing_replica_ids: SUBSET Server, replica_ids: SUBSET Server]

#[derive(PartialEq, Eq, Structural)]
pub struct LView {
    pub epoch: nat,
    pub proposing_replica_ids: Set<int>,
    pub replica_ids: Set<int>,
}

// ---- JPool ----
// TLA+: JPool == [max_seen_ballot: Nat, accepted_ballot: Nat,
//                 accepted_value: SUBSET Commands, pool: [Key -> Commands union {NilCmd}]]
// pool maps key -> Option<LCmd> where None = NilCmd

#[derive(PartialEq, Eq, Structural)]
pub struct LJPool {
    pub max_seen_ballot: nat,
    pub accepted_ballot: nat,
    pub accepted_value: Set<LCmd>,
    pub pool: Map<int, Option<LCmd>>,
}

// ---- PrepResp ----
// TLA+: PrepResp == [accepted_ballot: Nat, accepted_value: SUBSET Commands]

#[derive(PartialEq, Eq, Structural)]
pub struct LPrepResp {
    pub accepted_ballot: nat,
    pub accepted_value: Set<LCmd>,
}

// ---- Message types ----
// TLA+: 9 message types distinguished by mtype field.
// Verus: enum variants.

#[derive(PartialEq, Eq, Structural)]
pub enum LMessage {
    PreacceptRequest {
        msource: int,
        mdest: int,
        mepoch: nat,
        mview: LView,
        mcmd: LCmd,
    },
    PreacceptResponse {
        msuccess: bool,
        mjepoch: nat,
        mview: LView,
        mcmd: LCmd,
        msource: int,
        mdest: int,
    },
    BeginRecoveryRequest {
        msource: int,
        mdest: int,
        mold_view: LView,
        mnew_view: LView,
    },
    BeginRecoveryResponse {
        mjpool: LJPool,
        msource: int,
        mdest: int,
    },
    JetpackPrepareRequest {
        moepoch: nat,
        mjepoch: nat,
        mmax_seen_ballot: nat,
        msource: int,
        mdest: int,
    },
    JetpackPrepareResponse {
        mok: bool,
        maccepted_ballot: nat,
        maccepted_value: Set<LCmd>,
        moepoch: nat,
        mjepoch: nat,
        mmax_seen_ballot: nat,
        msource: int,
        mdest: int,
    },
    JetpackAcceptRequest {
        moepoch: nat,
        mjepoch: nat,
        mmax_seen_ballot: nat,
        mvalue: Set<LCmd>,
        msource: int,
        mdest: int,
    },
    JetpackAcceptResponse {
        mok: bool,
        mmax_seen_ballot: nat,
        msource: int,
        mdest: int,
    },
    FinishRecoveryRequest {
        moepoch: nat,
        mnew_view: LView,
        msource: int,
        mdest: int,
    },
}

// ---- Constants ----
// TLA+: CONSTANTS Server, Client, CmdId, Key, NoOpCmd, Proposer, ProposerOf(_)

pub struct LConstants {
    pub server: Set<int>,
    pub client: Set<int>,
    pub cmd_id: Set<int>,
    pub key: Set<int>,
    pub no_op_cmd: LCmd,
    pub proposer: Set<int>,
    pub proposer_of: Map<int, int>,  // Server -> Proposer
}

// ---- State ----
// All variables from jetpack.tla combined into a single state struct.
// See translation_audit.md section3 for the complete variable inventory.

pub struct LState {
    // Message bag: Map<LMessage, nat> where value = multiplicity
    pub messages: Map<LMessage, nat>,

    // Base protocol variables (Jetpack reads/writes)
    pub current_term: Map<int, nat>,          // [Server -> Nat]
    pub ostate: Map<int, LOState>,            // [Server -> LOState]
    pub log: Map<int, Map<int, Seq<LLogEntry>>>,  // [Server -> [Proposer -> Seq(LogEntry)]]
    pub commit_index: Map<int, Map<int, nat>>,     // [Server -> [Proposer -> Nat]]

    // Jetpack per-server variables
    pub jstate: Map<int, LJState>,
    pub jepoch: Map<int, nat>,
    pub oepoch: Map<int, nat>,
    pub old_view: Map<int, LView>,
    pub new_view: Map<int, LView>,
    pub jpool: Map<int, LJPool>,
    pub recovery_set: Map<int, Set<LCmd>>,
    pub chosen_value: Map<int, Set<LCmd>>,
    pub br_responses: Map<int, Map<int, Option<LJPool>>>,
    pub prep_responses: Map<int, Map<int, Option<LPrepResp>>>,
    pub accept_responses: Map<int, Map<int, bool>>,

    // Client variables
    pub client_view: Map<int, LView>,
    pub client_pending: Map<int, Option<LCmd>>,
    pub client_successes: Map<int, Set<int>>,
    pub client_heard_from: Map<int, Set<int>>,

    // Execution tracking
    pub original_execution_cmds: Seq<LCmd>,
    pub execution_cmds: Seq<LCmd>,
}

} // verus!
