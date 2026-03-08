// Jetpack plugin consensus protocol — hand-translated from jetpack/jetpack.tla
//
// This is a tla-rs / Verus-style spec preserving the genuine 3-D log model,
// message-bag semantics, per-proposer commitIndex, and all Jetpack actions
// and safety properties from the original TLA+ spec.
//
// See jetpack/translation_audit.md for the translation mapping.

#![allow(unused)]

use vstd::prelude::*;

// In a real Verus build these would be module imports:
// use crate::jetpack::types::*;
// For now, types are defined in types.rs alongside this file.
// Include types inline for self-containment during development.
include!("types.rs");

verus! {

// =========================================================================
// Sentinels and defaults
// =========================================================================

// TLA+: DefaultView == [epoch |-> 1, proposing_replica_ids |-> Server, replica_ids |-> Server]
pub open spec fn LDefaultView(c: LConstants) -> LView {
    LView {
        epoch: 1,
        proposing_replica_ids: c.server,
        replica_ids: c.server,
    }
}

// TLA+: EmptyJPool == [max_seen_ballot |-> 0, accepted_ballot |-> 0,
//                      accepted_value |-> {}, pool |-> [k \in Key |-> NilCmd]]
pub open spec fn LEmptyJPool(c: LConstants) -> LJPool {
    LJPool {
        max_seen_ballot: 0,
        accepted_ballot: 0,
        accepted_value: Set::empty(),
        pool: Map::new(|k: int| c.key.contains(k), |k: int| None),
    }
}

// =========================================================================
// Quorum helpers
// =========================================================================

// TLA+: Quorum == {q \in SUBSET(Server) : Cardinality(q) * 2 > Cardinality(Server)}
pub open spec fn LQuorum(c: LConstants) -> Set<Set<int>> {
    Set::new(|q: Set<int>|
        q.subset_of(c.server)
        && q.finite()
        && c.server.finite()
        && q.len() * 2 > c.server.len()
    )
}

// TLA+: JQuorum(v) == {q \in SUBSET(v.replica_ids) : Cardinality(q) * 2 > Cardinality(v.replica_ids)}
pub open spec fn LJQuorum(v: LView) -> Set<Set<int>> {
    Set::new(|q: Set<int>|
        q.subset_of(v.replica_ids)
        && q.finite()
        && v.replica_ids.finite()
        && q.len() * 2 > v.replica_ids.len()
    )
}

// TLA+: FastpathQuorum(v) == {q \in JQuorum(v) :
//     /\ v.proposing_replica_ids \subseteq q
//     /\ \A q2 \in JQuorum(v) :
//          v.proposing_replica_ids \subseteq q2 => (q \cap q2) \in JQuorum(v)}
pub open spec fn LFastpathQuorum(v: LView) -> Set<Set<int>> {
    Set::new(|q: Set<int>|
        LJQuorum(v).contains(q)
        && v.proposing_replica_ids.subset_of(q)
        && forall |q2: Set<int>|
            LJQuorum(v).contains(q2) && v.proposing_replica_ids.subset_of(q2)
            ==> LJQuorum(v).contains(q.intersect(q2))
    )
}

// =========================================================================
// Utility helpers
// =========================================================================

// TLA+: Min(s) == CHOOSE x \in s : \A y \in s : x <= y
pub open spec fn LMin(s: Set<nat>) -> nat
    recommends s.len() > 0
{
    choose |x: nat| s.contains(x) && forall |y: nat| s.contains(y) ==> x <= y
}

// TLA+: Max(s) == CHOOSE x \in s : \A y \in s : x >= y
pub open spec fn LMax(s: Set<nat>) -> nat
    recommends s.len() > 0
{
    choose |x: nat| s.contains(x) && forall |y: nat| s.contains(y) ==> x >= y
}

// TLA+: SeqToSet(s) == {s[i] : i \in 1..Len(s)}
pub open spec fn LSeqToSet(s: Seq<LCmd>) -> Set<LCmd> {
    Set::new(|cmd: LCmd| exists |i: int| 0 <= i < s.len() && s[i] == cmd)
}

// TLA+: CmdConflicts(a, b) == a.key = b.key /\ a # b
pub open spec fn LCmdConflicts(a: LCmd, b: LCmd) -> bool {
    a.key == b.key && a != b
}

// =========================================================================
// Message bag helpers
// =========================================================================

// TLA+: WithMessage(m, msgs)
pub open spec fn LWithMessage(m: LMessage, msgs: Map<LMessage, nat>) -> Map<LMessage, nat> {
    if msgs.contains_key(m) {
        msgs.insert(m, msgs[m] + 1)
    } else {
        msgs.insert(m, 1)
    }
}

// TLA+: WithoutMessage(m, msgs)
pub open spec fn LWithoutMessage(m: LMessage, msgs: Map<LMessage, nat>) -> Map<LMessage, nat> {
    if msgs.contains_key(m) {
        if msgs[m] <= 1 {
            msgs.remove(m)
        } else {
            msgs.insert(m, (msgs[m] - 1) as nat)
        }
    } else {
        msgs
    }
}

// TLA+: RECURSIVE AddMessages(_, _)
// AddMessages(ms, msgs) == IF ms = {} THEN msgs
//     ELSE LET m == CHOOSE x \in ms : TRUE
//          IN AddMessages(ms \ {m}, WithMessage(m, msgs))
//
// Verus: We use a recursive spec fn over a sequence representation of the set,
// since Verus sets don't have a natural structural recursion.
// For spec purposes, we define AddMessages as a fold over an arbitrary ordering.
pub open spec fn LAddMessages(ms: Set<LMessage>, msgs: Map<LMessage, nat>) -> Map<LMessage, nat>
    decreases ms.len()
{
    if ms.finite() && ms.len() > 0 {
        let m = ms.choose();
        LAddMessages(ms.remove(m), LWithMessage(m, msgs))
    } else {
        msgs
    }
}

// =========================================================================
// Recursive sequence helpers
// =========================================================================

// TLA+: RECURSIVE RemoveCmd(_, _)
pub open spec fn LRemoveCmd(seq: Seq<LCmd>, cmd: LCmd) -> Seq<LCmd>
    decreases seq.len()
{
    if seq.len() == 0 {
        Seq::empty()
    } else {
        let head = seq[0];
        let tail = seq.subrange(1, seq.len() as int);
        if head == cmd {
            LRemoveCmd(tail, cmd)
        } else {
            Seq::empty().push(head) + LRemoveCmd(tail, cmd)
        }
    }
}

// TLA+: RECURSIVE Dedup(_)
pub open spec fn LDedup(seq: Seq<LCmd>) -> Seq<LCmd>
    decreases seq.len()
{
    if seq.len() == 0 {
        Seq::empty()
    } else {
        let head = seq[0];
        let tail = seq.subrange(1, seq.len() as int);
        Seq::empty().push(head) + LDedup(LRemoveCmd(tail, head))
    }
}

// TLA+: FilterNoOps(seq) == SelectSeq(seq, LAMBDA x : x # NoOpCmd)
pub open spec fn LFilterNoOps(seq: Seq<LCmd>, no_op: LCmd) -> Seq<LCmd>
    decreases seq.len()
{
    if seq.len() == 0 {
        Seq::empty()
    } else {
        let head = seq[0];
        let tail = seq.subrange(1, seq.len() as int);
        if head != no_op {
            Seq::empty().push(head) + LFilterNoOps(tail, no_op)
        } else {
            LFilterNoOps(tail, no_op)
        }
    }
}

// TLA+: RECURSIVE IndexOf(_, _)
// Returns 0 if not found, 1-based index otherwise.
pub open spec fn LIndexOf(s: Seq<LCmd>, e: LCmd) -> int
    decreases s.len()
{
    if s.len() == 0 {
        0int
    } else if s[0] == e {
        1int
    } else {
        let rest = LIndexOf(s.subrange(1, s.len() as int), e);
        if rest == 0 { 0int } else { rest + 1 }
    }
}

// TLA+: ConflictOrderPreserved(s1, s2)
pub open spec fn LConflictOrderPreserved(s1: Seq<LCmd>, s2: Seq<LCmd>) -> bool {
    forall |k1: int, k2: int|
        0 <= k1 < k2 < s1.len()
        && LCmdConflicts(s1[k1], s1[k2])
        && LIndexOf(s2, s1[k1]) > 0
        && LIndexOf(s2, s1[k2]) > 0
        ==> LIndexOf(s2, s1[k1]) < LIndexOf(s2, s1[k2])
}

// =========================================================================
// Log / commit helpers
// =========================================================================

// TLA+: JPoolCommands(p) == {p.pool[k] : k \in Key} \ {NilCmd}
pub open spec fn LJPoolCommands(p: LJPool, c: LConstants) -> Set<LCmd> {
    Set::new(|cmd: LCmd|
        exists |k: int| c.key.contains(k) && p.pool.contains_key(k) && p.pool[k] == Some(cmd)
    )
}

// TLA+: HasConflict(pool, cmd)
pub open spec fn LHasConflict(pool: Map<int, Option<LCmd>>, cmd: LCmd) -> bool {
    pool.contains_key(cmd.key)
    && pool[cmd.key] != None
    && pool[cmd.key] != Some(cmd)
}

// TLA+: RecoveryCommands(i, qs)
pub open spec fn LRecoveryCommands(s: LState, c: LConstants, i: int, qs: Set<int>) -> Set<LCmd> {
    Set::new(|cmd: LCmd|
        exists |cmd_id: int, key: int|
            c.cmd_id.contains(cmd_id)
            && c.key.contains(key)
            && cmd == LCmd { cmd_id, key }
            && ({
                let supporters = Set::new(|sv: int|
                    qs.contains(sv)
                    && s.br_responses[i].contains_key(sv)
                    && s.br_responses[i][sv].is_some()
                    && LJPoolCommands(s.br_responses[i][sv].unwrap(), c).contains(cmd)
                );
                supporters.finite() && qs.finite()
                && supporters.len() * 2 > qs.len()
            })
    )
}

// TLA+: CommittedCmds(i, j)
pub open spec fn LCommittedCmds(s: LState, i: int, j: int) -> Seq<LCmd> {
    let ci = s.commit_index[i][j];
    if ci == 0 {
        Seq::empty()
    } else {
        Seq::new(ci, |k: int| s.log[i][j][k].value)
    }
}

// TLA+: AllCommittedCmds(i)
pub open spec fn LAllCommittedCmds(s: LState, c: LConstants, i: int) -> Set<LCmd> {
    Set::new(|cmd: LCmd|
        exists |j: int|
            c.proposer.contains(j)
            && LSeqToSet(LCommittedCmds(s, i, j)).contains(cmd)
    )
}

// TLA+: LogCmdIds
pub open spec fn LLogCmdIds(s: LState, c: LConstants) -> Set<int> {
    Set::new(|id: int|
        exists |i: int, j: int, k: int|
            c.server.contains(i)
            && c.proposer.contains(j)
            && s.log.contains_key(i)
            && s.log[i].contains_key(j)
            && 0 <= k < s.log[i][j].len()
            && s.log[i][j][k].value.cmd_id == id
    )
}

// TLA+: ExecCmdIds
pub open spec fn LExecCmdIds(s: LState) -> Set<int> {
    Set::new(|id: int|
        exists |i: int| 0 <= i < s.execution_cmds.len() && s.execution_cmds[i].cmd_id == id
    )
}

// TLA+: OriginalExecCmdIds
pub open spec fn LOriginalExecCmdIds(s: LState) -> Set<int> {
    Set::new(|id: int|
        exists |i: int| 0 <= i < s.original_execution_cmds.len()
            && s.original_execution_cmds[i].cmd_id == id
    )
}

// TLA+: UsedCmdIds
pub open spec fn LUsedCmdIds(s: LState, c: LConstants) -> Set<int> {
    LLogCmdIds(s, c).union(LExecCmdIds(s)).union(LOriginalExecCmdIds(s))
}

// TLA+: AvailableCommands
pub open spec fn LAvailableCommands(s: LState, c: LConstants) -> Set<LCmd> {
    Set::new(|cmd: LCmd|
        c.cmd_id.contains(cmd.cmd_id)
        && c.key.contains(cmd.key)
        && !LUsedCmdIds(s, c).contains(cmd.cmd_id)
    )
}

// TLA+: ChosenExecutedInView(i)
pub open spec fn LChosenExecutedInView(s: LState, c: LConstants, i: int) -> bool {
    forall |cmd: LCmd|
        s.chosen_value[i].contains(cmd)
        ==> forall |sv: int|
            s.new_view[i].replica_ids.contains(sv)
            ==> LAllCommittedCmds(s, c, sv).contains(cmd)
}

// =========================================================================
// Initialization
// =========================================================================

// TLA+: InitJetpackVars
pub open spec fn LInitJetpackVars(s: LState, c: LConstants) -> bool {
    let dv = LDefaultView(c);
    let ejp = LEmptyJPool(c);
    &&& s.jstate == Map::new(|i: int| c.server.contains(i), |i: int| LJState::Ready)
    &&& s.jepoch == Map::new(|i: int| c.server.contains(i), |i: int| dv.epoch)
    &&& s.oepoch == Map::new(|i: int| c.server.contains(i), |i: int| dv.epoch)
    &&& s.old_view == Map::new(|i: int| c.server.contains(i), |i: int| dv)
    &&& s.new_view == Map::new(|i: int| c.server.contains(i), |i: int| dv)
    &&& s.jpool == Map::new(|i: int| c.server.contains(i), |i: int| ejp)
    &&& s.recovery_set == Map::new(|i: int| c.server.contains(i), |i: int| Set::empty())
    &&& s.chosen_value == Map::new(|i: int| c.server.contains(i), |i: int| Set::empty())
    &&& s.br_responses == Map::new(
            |i: int| c.server.contains(i),
            |i: int| Map::new(|j: int| c.server.contains(j), |j: int| None::<LJPool>)
        )
    &&& s.prep_responses == Map::new(
            |i: int| c.server.contains(i),
            |i: int| Map::new(|j: int| c.server.contains(j), |j: int| None::<LPrepResp>)
        )
    &&& s.accept_responses == Map::new(
            |i: int| c.server.contains(i),
            |i: int| Map::new(|j: int| c.server.contains(j), |j: int| false)
        )
}

// TLA+: InitClientVars
pub open spec fn LInitClientVars(s: LState, c: LConstants) -> bool {
    let dv = LDefaultView(c);
    &&& s.client_view == Map::new(|cv: int| c.client.contains(cv), |cv: int| dv)
    &&& s.client_pending == Map::new(|cv: int| c.client.contains(cv), |cv: int| None::<LCmd>)
    &&& s.client_successes == Map::new(|cv: int| c.client.contains(cv), |cv: int| Set::<int>::empty())
    &&& s.client_heard_from == Map::new(|cv: int| c.client.contains(cv), |cv: int| Set::<int>::empty())
}

// TLA+: InitExecutionVars
pub open spec fn LInitExecutionVars(s: LState) -> bool {
    &&& s.original_execution_cmds == Seq::empty()
    &&& s.execution_cmds == Seq::empty()
}

// TLA+: Combined init (messages + base protocol init is wrapper-provided)
pub open spec fn LInit(s: LState, c: LConstants) -> bool {
    &&& s.messages == Map::empty()
    &&& LInitJetpackVars(s, c)
    &&& LInitClientVars(s, c)
    &&& LInitExecutionVars(s)
    // Base protocol init (currentTerm, ostate, log, commitIndex) is wrapper-specific.
    // For standalone spec, initialize to defaults:
    &&& s.current_term == Map::new(|i: int| c.server.contains(i), |i: int| 0nat)
    &&& s.ostate == Map::new(|i: int| c.server.contains(i), |i: int| LOState::Follower)
    &&& s.log == Map::new(
            |i: int| c.server.contains(i),
            |i: int| Map::new(|j: int| c.proposer.contains(j), |j: int| Seq::<LLogEntry>::empty())
        )
    &&& s.commit_index == Map::new(
            |i: int| c.server.contains(i),
            |i: int| Map::new(|j: int| c.proposer.contains(j), |j: int| 0nat)
        )
}

// =========================================================================
// Unchanged helpers (for readability in actions)
// =========================================================================

pub open spec fn LUnchangedBaseVars(s: LState, s_: LState) -> bool {
    &&& s_.current_term == s.current_term
    &&& s_.ostate == s.ostate
    &&& s_.log == s.log
    &&& s_.commit_index == s.commit_index
}

pub open spec fn LUnchangedJetpackVars(s: LState, s_: LState) -> bool {
    &&& s_.jstate == s.jstate
    &&& s_.jepoch == s.jepoch
    &&& s_.oepoch == s.oepoch
    &&& s_.old_view == s.old_view
    &&& s_.new_view == s.new_view
    &&& s_.jpool == s.jpool
    &&& s_.recovery_set == s.recovery_set
    &&& s_.chosen_value == s.chosen_value
    &&& s_.br_responses == s.br_responses
    &&& s_.prep_responses == s.prep_responses
    &&& s_.accept_responses == s.accept_responses
}

pub open spec fn LUnchangedClientVars(s: LState, s_: LState) -> bool {
    &&& s_.client_view == s.client_view
    &&& s_.client_pending == s.client_pending
    &&& s_.client_successes == s.client_successes
    &&& s_.client_heard_from == s.client_heard_from
}

pub open spec fn LUnchangedExecutionVars(s: LState, s_: LState) -> bool {
    &&& s_.original_execution_cmds == s.original_execution_cmds
    &&& s_.execution_cmds == s.execution_cmds
}

// =========================================================================
// Actions
// =========================================================================

// ---- ClientSendPreaccept(c) ----
// TLA+: Client sends a Preaccept to all replicas in its view.
pub open spec fn LClientSendPreaccept(s: LState, s_: LState, c: LConstants, cl: int) -> bool {
    &&& c.client.contains(cl)
    &&& s.client_pending[cl] == None
    &&& LAvailableCommands(s, c).len() > 0
    &&& exists |cmd: LCmd| {
        &&& LAvailableCommands(s, c).contains(cmd)
        &&& {
            let view = s.client_view[cl];
            let msg_set = Set::new(|m: LMessage|
                exists |sv: int| view.replica_ids.contains(sv) && m == LMessage::PreacceptRequest {
                    msource: cl,
                    mdest: sv,
                    mepoch: view.epoch,
                    mview: view,
                    mcmd: cmd,
                }
            );
            &&& s_.messages == LAddMessages(msg_set, s.messages)
            &&& s_.client_pending == s.client_pending.insert(cl, Some(cmd))
            &&& s_.client_successes == s.client_successes.insert(cl, Set::empty())
            &&& s_.client_heard_from == s.client_heard_from.insert(cl, Set::empty())
            &&& s_.client_view == s.client_view
            &&& LUnchangedBaseVars(s, s_)
            &&& LUnchangedJetpackVars(s, s_)
            &&& LUnchangedExecutionVars(s, s_)
        }
    }
}

// ---- HandlePreacceptRequest(i, m) ----
pub open spec fn LHandlePreacceptRequest(s: LState, s_: LState, c: LConstants, i: int, m: LMessage) -> bool {
    &&& c.server.contains(i)
    &&& s.messages.contains_key(m)
    &&& m matches LMessage::PreacceptRequest { msource, mdest, mepoch, mview, mcmd }
    &&& mdest == i
    &&& {
        let cmd = mcmd;
        let epoch_ok = mepoch == s.jepoch[i];
        let ready_ok = s.jstate[i] == LJState::Ready;
        let no_conflict = !LHasConflict(s.jpool[i].pool, cmd);
        let accept = epoch_ok && ready_ok && no_conflict;
        let reply = LMessage::PreacceptResponse {
            msuccess: accept,
            mjepoch: s.jepoch[i],
            mview: s.new_view[i],
            mcmd: cmd,
            msource: i,
            mdest: msource,
        };
        let j = c.proposer_of[i];
        let new_log_entry = LLogEntry { term: s.current_term[i], value: cmd };
        let new_log_seq = s.log[i][j].push(new_log_entry);

        &&& s_.jpool == if accept {
                s.jpool.insert(i, LJPool {
                    pool: s.jpool[i].pool.insert(cmd.key, Some(cmd)),
                    ..s.jpool[i]
                })
            } else { s.jpool }
        &&& s_.log == if epoch_ok && ready_ok && s.ostate[i] == LOState::Leader {
                s.log.insert(i, s.log[i].insert(j, new_log_seq))
            } else { s.log }
        &&& s_.messages == LWithoutMessage(m, LWithMessage(reply, s.messages))
        &&& s_.current_term == s.current_term
        &&& s_.ostate == s.ostate
        &&& s_.commit_index == s.commit_index
        &&& s_.jstate == s.jstate
        &&& s_.jepoch == s.jepoch
        &&& s_.oepoch == s.oepoch
        &&& s_.old_view == s.old_view
        &&& s_.new_view == s.new_view
        &&& s_.recovery_set == s.recovery_set
        &&& s_.chosen_value == s.chosen_value
        &&& s_.br_responses == s.br_responses
        &&& s_.prep_responses == s.prep_responses
        &&& s_.accept_responses == s.accept_responses
        &&& LUnchangedClientVars(s, s_)
        &&& LUnchangedExecutionVars(s, s_)
    }
}

// ---- HandlePreacceptResponse(c, m) ----
pub open spec fn LHandlePreacceptResponse(s: LState, s_: LState, c: LConstants, cl: int, m: LMessage) -> bool {
    &&& c.client.contains(cl)
    &&& s.messages.contains_key(m)
    &&& m matches LMessage::PreacceptResponse { msuccess, mjepoch, mview, mcmd, msource, mdest }
    &&& mdest == cl
    &&& s.client_pending[cl] == Some(mcmd)
    &&& {
        let view = s.client_view[cl];
        let new_heard = s.client_heard_from[cl].insert(msource);
        let new_successes = if msuccess {
            s.client_successes[cl].insert(msource)
        } else {
            s.client_successes[cl]
        };
        let fast_ok = LFastpathQuorum(view).contains(new_successes);
        let remaining = view.replica_ids.difference(new_heard);
        let can_still_succeed = exists |q: Set<int>|
            LFastpathQuorum(view).contains(q)
            && q.subset_of(new_successes.union(remaining));
        let abandon = !fast_ok && !can_still_succeed;

        &&& s_.client_successes == if fast_ok || abandon {
                s.client_successes.insert(cl, Set::empty())
            } else {
                s.client_successes.insert(cl, new_successes)
            }
        &&& s_.client_heard_from == if fast_ok || abandon {
                s.client_heard_from.insert(cl, Set::empty())
            } else {
                s.client_heard_from.insert(cl, new_heard)
            }
        &&& s_.client_pending == if fast_ok || abandon {
                s.client_pending.insert(cl, None)
            } else {
                s.client_pending
            }
        &&& s_.client_view == if !msuccess && mview.epoch > s.client_view[cl].epoch {
                s.client_view.insert(cl, mview)
            } else {
                s.client_view
            }
        &&& s_.execution_cmds == if fast_ok {
                s.execution_cmds.push(mcmd)
            } else {
                s.execution_cmds
            }
        &&& s_.original_execution_cmds == s.original_execution_cmds
        &&& s_.messages == LWithoutMessage(m, s.messages)
        &&& LUnchangedBaseVars(s, s_)
        &&& LUnchangedJetpackVars(s, s_)
    }
}

// ---- SendBeginRecovery(i) ----
pub open spec fn LSendBeginRecovery(s: LState, s_: LState, c: LConstants, i: int) -> bool {
    &&& c.server.contains(i)
    &&& s.ostate[i] == LOState::ToBeLeader
    &&& s.jstate[i] == LJState::Ready
    &&& {
        let view = s.new_view[i];
        let msg_set = Set::new(|m: LMessage|
            exists |sv: int| view.replica_ids.contains(sv) && m == LMessage::BeginRecoveryRequest {
                msource: i,
                mdest: sv,
                mold_view: s.old_view[i],
                mnew_view: s.new_view[i],
            }
        );
        &&& s_.messages == LAddMessages(msg_set, s.messages)
        &&& s_.jstate == s.jstate.insert(i, LJState::Recovery)
        &&& s_.br_responses == s.br_responses.insert(i,
                Map::new(|j: int| c.server.contains(j), |j: int| None::<LJPool>))
        &&& LUnchangedBaseVars(s, s_)
        &&& s_.jepoch == s.jepoch
        &&& s_.oepoch == s.oepoch
        &&& s_.old_view == s.old_view
        &&& s_.new_view == s.new_view
        &&& s_.jpool == s.jpool
        &&& s_.recovery_set == s.recovery_set
        &&& s_.chosen_value == s.chosen_value
        &&& s_.prep_responses == s.prep_responses
        &&& s_.accept_responses == s.accept_responses
        &&& LUnchangedClientVars(s, s_)
        &&& LUnchangedExecutionVars(s, s_)
    }
}

// ---- HandleBeginRecoveryRequest(i, m) ----
pub open spec fn LHandleBeginRecoveryRequest(s: LState, s_: LState, c: LConstants, i: int, m: LMessage) -> bool {
    &&& c.server.contains(i)
    &&& s.messages.contains_key(m)
    &&& m matches LMessage::BeginRecoveryRequest { msource, mdest, mold_view, mnew_view }
    &&& mdest == i
    &&& {
        let reply = LMessage::BeginRecoveryResponse {
            mjpool: s.jpool[i],
            msource: i,
            mdest: msource,
        };
        &&& s_.old_view == s.old_view.insert(i, mold_view)
        &&& s_.new_view == s.new_view.insert(i, mnew_view)
        &&& s_.oepoch == s.oepoch.insert(i, mnew_view.epoch)
        &&& s_.jstate == s.jstate.insert(i, LJState::Recovery)
        &&& s_.messages == LWithoutMessage(m, LWithMessage(reply, s.messages))
        &&& LUnchangedBaseVars(s, s_)
        &&& s_.jepoch == s.jepoch
        &&& s_.jpool == s.jpool
        &&& s_.recovery_set == s.recovery_set
        &&& s_.chosen_value == s.chosen_value
        &&& s_.br_responses == s.br_responses
        &&& s_.prep_responses == s.prep_responses
        &&& s_.accept_responses == s.accept_responses
        &&& LUnchangedClientVars(s, s_)
        &&& LUnchangedExecutionVars(s, s_)
    }
}

// ---- HandleBeginRecoveryResponse(i, m) ----
pub open spec fn LHandleBeginRecoveryResponse(s: LState, s_: LState, c: LConstants, i: int, m: LMessage) -> bool {
    &&& c.server.contains(i)
    &&& s.messages.contains_key(m)
    &&& m matches LMessage::BeginRecoveryResponse { mjpool, msource, mdest }
    &&& mdest == i
    &&& s.jstate[i] == LJState::Recovery
    &&& s_.br_responses == s.br_responses.insert(i, s.br_responses[i].insert(msource, Some(mjpool)))
    &&& s_.messages == LWithoutMessage(m, s.messages)
    &&& LUnchangedBaseVars(s, s_)
    &&& s_.jstate == s.jstate
    &&& s_.jepoch == s.jepoch
    &&& s_.oepoch == s.oepoch
    &&& s_.old_view == s.old_view
    &&& s_.new_view == s.new_view
    &&& s_.jpool == s.jpool
    &&& s_.recovery_set == s.recovery_set
    &&& s_.chosen_value == s.chosen_value
    &&& s_.prep_responses == s.prep_responses
    &&& s_.accept_responses == s.accept_responses
    &&& LUnchangedClientVars(s, s_)
    &&& LUnchangedExecutionVars(s, s_)
}

// ---- CompleteBeginRecovery(i) ----
pub open spec fn LCompleteBeginRecovery(s: LState, s_: LState, c: LConstants, i: int) -> bool {
    &&& c.server.contains(i)
    &&& s.jstate[i] == LJState::Recovery
    &&& exists |qs: Set<int>| {
        &&& LJQuorum(s.new_view[i]).contains(qs)
        &&& forall |sv: int| qs.contains(sv) ==>
            s.br_responses[i].contains_key(sv) && s.br_responses[i][sv].is_some()
        &&& {
            let rec = LRecoveryCommands(s, c, i, qs);
            &&& s_.recovery_set == s.recovery_set.insert(i, rec)
            &&& s_.chosen_value == s.chosen_value.insert(i, rec)
            &&& s_.jstate == s.jstate.insert(i, LJState::AfterBeginRecovery)
        }
    }
    &&& s_.messages == s.messages
    &&& LUnchangedBaseVars(s, s_)
    &&& s_.jepoch == s.jepoch
    &&& s_.oepoch == s.oepoch
    &&& s_.old_view == s.old_view
    &&& s_.new_view == s.new_view
    &&& s_.jpool == s.jpool
    &&& s_.br_responses == s.br_responses
    &&& s_.prep_responses == s.prep_responses
    &&& s_.accept_responses == s.accept_responses
    &&& LUnchangedClientVars(s, s_)
    &&& LUnchangedExecutionVars(s, s_)
}

// ---- SendPrepare(i) ----
pub open spec fn LSendPrepare(s: LState, s_: LState, c: LConstants, i: int) -> bool {
    &&& c.server.contains(i)
    &&& s.jstate[i] == LJState::AfterBeginRecovery
    &&& {
        let view = s.new_view[i];
        let msg_set = Set::new(|m: LMessage|
            exists |sv: int| view.replica_ids.contains(sv) && m == LMessage::JetpackPrepareRequest {
                moepoch: s.oepoch[i],
                mjepoch: s.jepoch[i],
                mmax_seen_ballot: s.jpool[i].max_seen_ballot,
                msource: i,
                mdest: sv,
            }
        );
        &&& s_.messages == LAddMessages(msg_set, s.messages)
        &&& s_.prep_responses == s.prep_responses.insert(i,
                Map::new(|j: int| c.server.contains(j), |j: int| None::<LPrepResp>))
        &&& LUnchangedBaseVars(s, s_)
        &&& s_.jstate == s.jstate
        &&& s_.jepoch == s.jepoch
        &&& s_.oepoch == s.oepoch
        &&& s_.old_view == s.old_view
        &&& s_.new_view == s.new_view
        &&& s_.jpool == s.jpool
        &&& s_.recovery_set == s.recovery_set
        &&& s_.chosen_value == s.chosen_value
        &&& s_.br_responses == s.br_responses
        &&& s_.accept_responses == s.accept_responses
        &&& LUnchangedClientVars(s, s_)
        &&& LUnchangedExecutionVars(s, s_)
    }
}

// ---- HandlePrepareRequest(i, m) ----
pub open spec fn LHandlePrepareRequest(s: LState, s_: LState, c: LConstants, i: int, m: LMessage) -> bool {
    &&& c.server.contains(i)
    &&& s.messages.contains_key(m)
    &&& m matches LMessage::JetpackPrepareRequest { moepoch, mjepoch, mmax_seen_ballot, msource, mdest }
    &&& mdest == i
    &&& {
        let ok = moepoch >= s.oepoch[i]
            && mjepoch >= s.jepoch[i]
            && mmax_seen_ballot >= s.jpool[i].max_seen_ballot;
        let reply = if ok {
            LMessage::JetpackPrepareResponse {
                mok: true,
                maccepted_ballot: s.jpool[i].accepted_ballot,
                maccepted_value: s.jpool[i].accepted_value,
                moepoch: s.oepoch[i],
                mjepoch: s.jepoch[i],
                mmax_seen_ballot: mmax_seen_ballot,
                msource: i,
                mdest: msource,
            }
        } else {
            LMessage::JetpackPrepareResponse {
                mok: false,
                maccepted_ballot: s.jpool[i].accepted_ballot,
                maccepted_value: s.jpool[i].accepted_value,
                moepoch: s.oepoch[i],
                mjepoch: s.jepoch[i],
                mmax_seen_ballot: s.jpool[i].max_seen_ballot,
                msource: i,
                mdest: msource,
            }
        };
        let max_oepoch = if s.oepoch[i] >= moepoch { s.oepoch[i] } else { moepoch };
        let max_jepoch = if s.jepoch[i] >= mjepoch { s.jepoch[i] } else { mjepoch };

        &&& s_.oepoch == if ok { s.oepoch.insert(i, max_oepoch) } else { s.oepoch }
        &&& s_.jepoch == if ok { s.jepoch.insert(i, max_jepoch) } else { s.jepoch }
        &&& s_.jpool == if ok {
                s.jpool.insert(i, LJPool { max_seen_ballot: mmax_seen_ballot, ..s.jpool[i] })
            } else { s.jpool }
        &&& s_.messages == LWithoutMessage(m, LWithMessage(reply, s.messages))
        &&& LUnchangedBaseVars(s, s_)
        &&& s_.jstate == s.jstate
        &&& s_.old_view == s.old_view
        &&& s_.new_view == s.new_view
        &&& s_.recovery_set == s.recovery_set
        &&& s_.chosen_value == s.chosen_value
        &&& s_.br_responses == s.br_responses
        &&& s_.prep_responses == s.prep_responses
        &&& s_.accept_responses == s.accept_responses
        &&& LUnchangedClientVars(s, s_)
        &&& LUnchangedExecutionVars(s, s_)
    }
}

// ---- HandlePrepareResponse(i, m) ----
pub open spec fn LHandlePrepareResponse(s: LState, s_: LState, c: LConstants, i: int, m: LMessage) -> bool {
    &&& c.server.contains(i)
    &&& s.messages.contains_key(m)
    &&& m matches LMessage::JetpackPrepareResponse { mok, maccepted_ballot, maccepted_value, moepoch, mjepoch, mmax_seen_ballot, msource, mdest }
    &&& mdest == i
    &&& if mok {
        &&& s_.prep_responses == s.prep_responses.insert(i,
                s.prep_responses[i].insert(msource, Some(LPrepResp {
                    accepted_ballot: maccepted_ballot,
                    accepted_value: maccepted_value,
                })))
        &&& s_.oepoch == s.oepoch
        &&& s_.jepoch == s.jepoch
        &&& s_.jpool == s.jpool
    } else {
        let max_oepoch = if s.oepoch[i] >= moepoch { s.oepoch[i] } else { moepoch };
        let max_jepoch = if s.jepoch[i] >= mjepoch { s.jepoch[i] } else { mjepoch };
        let max_ballot = if s.jpool[i].max_seen_ballot >= mmax_seen_ballot {
            s.jpool[i].max_seen_ballot
        } else {
            mmax_seen_ballot
        };
        &&& s_.prep_responses == s.prep_responses
        &&& s_.oepoch == s.oepoch.insert(i, max_oepoch)
        &&& s_.jepoch == s.jepoch.insert(i, max_jepoch)
        &&& s_.jpool == s.jpool.insert(i, LJPool { max_seen_ballot: max_ballot, ..s.jpool[i] })
    }
    &&& s_.messages == LWithoutMessage(m, s.messages)
    &&& LUnchangedBaseVars(s, s_)
    &&& s_.jstate == s.jstate
    &&& s_.old_view == s.old_view
    &&& s_.new_view == s.new_view
    &&& s_.recovery_set == s.recovery_set
    &&& s_.chosen_value == s.chosen_value
    &&& s_.br_responses == s.br_responses
    &&& s_.accept_responses == s.accept_responses
    &&& LUnchangedClientVars(s, s_)
    &&& LUnchangedExecutionVars(s, s_)
}

// ---- CompletePrepare(i) ----
pub open spec fn LCompletePrepare(s: LState, s_: LState, c: LConstants, i: int) -> bool {
    &&& c.server.contains(i)
    &&& s.jstate[i] == LJState::AfterBeginRecovery
    &&& exists |qs: Set<int>| {
        &&& LJQuorum(s.new_view[i]).contains(qs)
        &&& forall |sv: int| qs.contains(sv) ==>
            s.prep_responses[i].contains_key(sv) && s.prep_responses[i][sv].is_some()
        &&& {
            let resp_vals = Set::new(|r: LPrepResp|
                exists |sv: int| qs.contains(sv) && s.prep_responses[i][sv] == Some(r)
            );
            let ballots = Set::new(|b: nat|
                exists |r: LPrepResp| resp_vals.contains(r) && r.accepted_ballot == b
            );
            let maxb = if ballots.len() == 0 { 0nat } else { LMax(ballots) };
            let top_vals = Set::new(|r: LPrepResp|
                resp_vals.contains(r) && r.accepted_ballot == maxb
            );
            let best_vals = Set::new(|v: Set<LCmd>|
                exists |r: LPrepResp| top_vals.contains(r) && r.accepted_value == v
            );
            let pick = if maxb == 0 || best_vals.len() == 0 {
                s.chosen_value[i]
            } else {
                best_vals.choose()
            };
            &&& s_.chosen_value == s.chosen_value.insert(i, pick)
            &&& s_.jstate == s.jstate.insert(i, LJState::AfterPrepare)
        }
    }
    &&& s_.messages == s.messages
    &&& LUnchangedBaseVars(s, s_)
    &&& s_.jepoch == s.jepoch
    &&& s_.oepoch == s.oepoch
    &&& s_.old_view == s.old_view
    &&& s_.new_view == s.new_view
    &&& s_.jpool == s.jpool
    &&& s_.recovery_set == s.recovery_set
    &&& s_.br_responses == s.br_responses
    &&& s_.prep_responses == s.prep_responses
    &&& s_.accept_responses == s.accept_responses
    &&& LUnchangedClientVars(s, s_)
    &&& LUnchangedExecutionVars(s, s_)
}

// ---- SendAccept(i) ----
pub open spec fn LSendAccept(s: LState, s_: LState, c: LConstants, i: int) -> bool {
    &&& c.server.contains(i)
    &&& s.jstate[i] == LJState::AfterPrepare
    &&& {
        let view = s.new_view[i];
        let msg_set = Set::new(|m: LMessage|
            exists |sv: int| view.replica_ids.contains(sv) && m == LMessage::JetpackAcceptRequest {
                moepoch: s.oepoch[i],
                mjepoch: s.jepoch[i],
                mmax_seen_ballot: s.jpool[i].max_seen_ballot,
                mvalue: s.chosen_value[i],
                msource: i,
                mdest: sv,
            }
        );
        &&& s_.messages == LAddMessages(msg_set, s.messages)
        &&& s_.accept_responses == s.accept_responses.insert(i,
                Map::new(|j: int| c.server.contains(j), |j: int| false))
        &&& LUnchangedBaseVars(s, s_)
        &&& s_.jstate == s.jstate
        &&& s_.jepoch == s.jepoch
        &&& s_.oepoch == s.oepoch
        &&& s_.old_view == s.old_view
        &&& s_.new_view == s.new_view
        &&& s_.jpool == s.jpool
        &&& s_.recovery_set == s.recovery_set
        &&& s_.chosen_value == s.chosen_value
        &&& s_.br_responses == s.br_responses
        &&& s_.prep_responses == s.prep_responses
        &&& LUnchangedClientVars(s, s_)
        &&& LUnchangedExecutionVars(s, s_)
    }
}

// ---- HandleAcceptRequest(i, m) ----
pub open spec fn LHandleAcceptRequest(s: LState, s_: LState, c: LConstants, i: int, m: LMessage) -> bool {
    &&& c.server.contains(i)
    &&& s.messages.contains_key(m)
    &&& m matches LMessage::JetpackAcceptRequest { moepoch, mjepoch, mmax_seen_ballot, mvalue, msource, mdest }
    &&& mdest == i
    &&& {
        let ok = moepoch >= s.oepoch[i]
            && mjepoch >= s.jepoch[i]
            && mmax_seen_ballot >= s.jpool[i].max_seen_ballot;
        let reply = if ok {
            LMessage::JetpackAcceptResponse {
                mok: true,
                mmax_seen_ballot: mmax_seen_ballot,
                msource: i,
                mdest: msource,
            }
        } else {
            LMessage::JetpackAcceptResponse {
                mok: false,
                mmax_seen_ballot: s.jpool[i].max_seen_ballot,
                msource: i,
                mdest: msource,
            }
        };
        let max_oepoch = if s.oepoch[i] >= moepoch { s.oepoch[i] } else { moepoch };
        let max_jepoch = if s.jepoch[i] >= mjepoch { s.jepoch[i] } else { mjepoch };

        &&& s_.oepoch == if ok { s.oepoch.insert(i, max_oepoch) } else { s.oepoch }
        &&& s_.jepoch == if ok { s.jepoch.insert(i, max_jepoch) } else { s.jepoch }
        &&& s_.jpool == if ok {
                s.jpool.insert(i, LJPool {
                    max_seen_ballot: mmax_seen_ballot,
                    accepted_ballot: mmax_seen_ballot,
                    accepted_value: mvalue,
                    ..s.jpool[i]
                })
            } else { s.jpool }
        &&& s_.messages == LWithoutMessage(m, LWithMessage(reply, s.messages))
        &&& LUnchangedBaseVars(s, s_)
        &&& s_.jstate == s.jstate
        &&& s_.old_view == s.old_view
        &&& s_.new_view == s.new_view
        &&& s_.recovery_set == s.recovery_set
        &&& s_.chosen_value == s.chosen_value
        &&& s_.br_responses == s.br_responses
        &&& s_.prep_responses == s.prep_responses
        &&& s_.accept_responses == s.accept_responses
        &&& LUnchangedClientVars(s, s_)
        &&& LUnchangedExecutionVars(s, s_)
    }
}

// ---- HandleAcceptResponse(i, m) ----
pub open spec fn LHandleAcceptResponse(s: LState, s_: LState, c: LConstants, i: int, m: LMessage) -> bool {
    &&& c.server.contains(i)
    &&& s.messages.contains_key(m)
    &&& m matches LMessage::JetpackAcceptResponse { mok, mmax_seen_ballot, msource, mdest }
    &&& mdest == i
    &&& if mok {
        &&& s_.accept_responses == s.accept_responses.insert(i,
                s.accept_responses[i].insert(msource, true))
        &&& s_.oepoch == s.oepoch
        &&& s_.jepoch == s.jepoch
        &&& s_.jpool == s.jpool
    } else {
        let max_ballot = if s.jpool[i].max_seen_ballot >= mmax_seen_ballot {
            s.jpool[i].max_seen_ballot
        } else {
            mmax_seen_ballot
        };
        &&& s_.accept_responses == s.accept_responses
        &&& s_.jpool == s.jpool.insert(i, LJPool { max_seen_ballot: max_ballot, ..s.jpool[i] })
        &&& s_.oepoch == s.oepoch
        &&& s_.jepoch == s.jepoch
    }
    &&& s_.messages == LWithoutMessage(m, s.messages)
    &&& LUnchangedBaseVars(s, s_)
    &&& s_.jstate == s.jstate
    &&& s_.old_view == s.old_view
    &&& s_.new_view == s.new_view
    &&& s_.recovery_set == s.recovery_set
    &&& s_.chosen_value == s.chosen_value
    &&& s_.br_responses == s.br_responses
    &&& s_.prep_responses == s.prep_responses
    &&& LUnchangedClientVars(s, s_)
    &&& LUnchangedExecutionVars(s, s_)
}

// ---- CompleteAccept(i) ----
pub open spec fn LCompleteAccept(s: LState, s_: LState, c: LConstants, i: int) -> bool {
    &&& c.server.contains(i)
    &&& s.jstate[i] == LJState::AfterPrepare
    &&& exists |qs: Set<int>| {
        &&& LJQuorum(s.new_view[i]).contains(qs)
        &&& forall |sv: int| qs.contains(sv) ==>
            s.accept_responses[i].contains_key(sv) && s.accept_responses[i][sv] == true
    }
    &&& s_.jstate == s.jstate.insert(i, LJState::AfterAccept)
    &&& s_.messages == s.messages
    &&& LUnchangedBaseVars(s, s_)
    &&& s_.jepoch == s.jepoch
    &&& s_.oepoch == s.oepoch
    &&& s_.old_view == s.old_view
    &&& s_.new_view == s.new_view
    &&& s_.jpool == s.jpool
    &&& s_.recovery_set == s.recovery_set
    &&& s_.chosen_value == s.chosen_value
    &&& s_.br_responses == s.br_responses
    &&& s_.prep_responses == s.prep_responses
    &&& s_.accept_responses == s.accept_responses
    &&& LUnchangedClientVars(s, s_)
    &&& LUnchangedExecutionVars(s, s_)
}

// ---- Resubmit(i) ----
pub open spec fn LResubmit(s: LState, s_: LState, c: LConstants, i: int) -> bool {
    &&& c.server.contains(i)
    &&& s.jstate[i] == LJState::AfterAccept
    &&& {
        let proposers = s.new_view[i].proposing_replica_ids;
        let msg_set = Set::new(|m: LMessage|
            exists |sv: int, cmd: LCmd|
                proposers.contains(sv)
                && s.chosen_value[i].contains(cmd)
                && m == LMessage::PreacceptRequest {
                    msource: i,
                    mdest: sv,
                    mepoch: s.jepoch[i],
                    mview: s.new_view[i],
                    mcmd: cmd,
                }
        );
        &&& s_.messages == LAddMessages(msg_set, s.messages)
        &&& LUnchangedBaseVars(s, s_)
        &&& LUnchangedJetpackVars(s, s_)
        &&& LUnchangedClientVars(s, s_)
        &&& LUnchangedExecutionVars(s, s_)
    }
}

// ---- CompleteResubmit(i) ----
pub open spec fn LCompleteResubmit(s: LState, s_: LState, c: LConstants, i: int) -> bool {
    &&& c.server.contains(i)
    &&& s.jstate[i] == LJState::AfterAccept
    &&& LChosenExecutedInView(s, c, i)
    &&& s_.jstate == s.jstate.insert(i, LJState::AfterResubmit)
    &&& s_.messages == s.messages
    &&& LUnchangedBaseVars(s, s_)
    &&& s_.jepoch == s.jepoch
    &&& s_.oepoch == s.oepoch
    &&& s_.old_view == s.old_view
    &&& s_.new_view == s.new_view
    &&& s_.jpool == s.jpool
    &&& s_.recovery_set == s.recovery_set
    &&& s_.chosen_value == s.chosen_value
    &&& s_.br_responses == s.br_responses
    &&& s_.prep_responses == s.prep_responses
    &&& s_.accept_responses == s.accept_responses
    &&& LUnchangedClientVars(s, s_)
    &&& LUnchangedExecutionVars(s, s_)
}

// ---- FinishRecovery(i) ----
pub open spec fn LFinishRecovery(s: LState, s_: LState, c: LConstants, i: int) -> bool {
    &&& c.server.contains(i)
    &&& s.jstate[i] == LJState::AfterResubmit
    &&& {
        let view = s.new_view[i];
        let msg_set = Set::new(|m: LMessage|
            exists |sv: int| view.replica_ids.contains(sv) && m == LMessage::FinishRecoveryRequest {
                moepoch: s.oepoch[i],
                mnew_view: view,
                msource: i,
                mdest: sv,
            }
        );
        &&& s_.messages == LAddMessages(msg_set, s.messages)
        &&& s_.jepoch == s.jepoch.insert(i, s.oepoch[i])
        &&& s_.oepoch == s.oepoch.insert(i, s.oepoch[i])
        &&& s_.old_view == s.old_view.insert(i, view)
        &&& s_.new_view == s.new_view.insert(i, view)
        &&& s_.jpool == s.jpool.insert(i, LEmptyJPool(c))
        &&& s_.jstate == s.jstate.insert(i, LJState::Ready)
        &&& s_.recovery_set == s.recovery_set.insert(i, Set::empty())
        &&& s_.chosen_value == s.chosen_value.insert(i, Set::empty())
        &&& s_.br_responses == s.br_responses.insert(i,
                Map::new(|j: int| c.server.contains(j), |j: int| None::<LJPool>))
        &&& s_.prep_responses == s.prep_responses.insert(i,
                Map::new(|j: int| c.server.contains(j), |j: int| None::<LPrepResp>))
        &&& s_.accept_responses == s.accept_responses.insert(i,
                Map::new(|j: int| c.server.contains(j), |j: int| false))
        &&& s_.ostate == s.ostate.insert(i, LOState::Leader)
        &&& s_.current_term == s.current_term
        &&& s_.log == s.log
        &&& s_.commit_index == s.commit_index
        &&& LUnchangedClientVars(s, s_)
        &&& LUnchangedExecutionVars(s, s_)
    }
}

// ---- HandleFinishRecovery(i, m) ----
pub open spec fn LHandleFinishRecovery(s: LState, s_: LState, c: LConstants, i: int, m: LMessage) -> bool {
    &&& c.server.contains(i)
    &&& s.messages.contains_key(m)
    &&& m matches LMessage::FinishRecoveryRequest { moepoch, mnew_view, msource, mdest }
    &&& mdest == i
    &&& s_.jepoch == s.jepoch.insert(i, moepoch)
    &&& s_.oepoch == s.oepoch.insert(i, moepoch)
    &&& s_.old_view == s.old_view.insert(i, mnew_view)
    &&& s_.new_view == s.new_view.insert(i, mnew_view)
    &&& s_.jpool == s.jpool.insert(i, LEmptyJPool(c))
    &&& s_.jstate == s.jstate.insert(i, LJState::Ready)
    &&& s_.recovery_set == s.recovery_set.insert(i, Set::empty())
    &&& s_.chosen_value == s.chosen_value.insert(i, Set::empty())
    &&& s_.ostate == s.ostate.insert(i,
            if s.ostate[i] == LOState::ToBeLeader { LOState::Leader } else { s.ostate[i] })
    &&& s_.messages == LWithoutMessage(m, s.messages)
    &&& s_.current_term == s.current_term
    &&& s_.log == s.log
    &&& s_.commit_index == s.commit_index
    &&& s_.br_responses == s.br_responses
    &&& s_.prep_responses == s.prep_responses
    &&& s_.accept_responses == s.accept_responses
    &&& LUnchangedClientVars(s, s_)
    &&& LUnchangedExecutionVars(s, s_)
}

// =========================================================================
// Top-level Next relation
// =========================================================================

// TLA+: JetpackNext (plus message handlers that are implicitly in the wrapper)
pub open spec fn LNext(s: LState, s_: LState, c: LConstants) -> bool {
    // Client actions
    ||| exists |cl: int| LClientSendPreaccept(s, s_, c, cl)
    ||| exists |cl: int, m: LMessage| LHandlePreacceptResponse(s, s_, c, cl, m)
    // Server preaccept handling
    ||| exists |i: int, m: LMessage| LHandlePreacceptRequest(s, s_, c, i, m)
    // Recovery Phase 1: BeginRecovery
    ||| exists |i: int| LSendBeginRecovery(s, s_, c, i)
    ||| exists |i: int, m: LMessage| LHandleBeginRecoveryRequest(s, s_, c, i, m)
    ||| exists |i: int, m: LMessage| LHandleBeginRecoveryResponse(s, s_, c, i, m)
    ||| exists |i: int| LCompleteBeginRecovery(s, s_, c, i)
    // Recovery Phase 2: Prepare
    ||| exists |i: int| LSendPrepare(s, s_, c, i)
    ||| exists |i: int, m: LMessage| LHandlePrepareRequest(s, s_, c, i, m)
    ||| exists |i: int, m: LMessage| LHandlePrepareResponse(s, s_, c, i, m)
    ||| exists |i: int| LCompletePrepare(s, s_, c, i)
    // Recovery Phase 3: Accept
    ||| exists |i: int| LSendAccept(s, s_, c, i)
    ||| exists |i: int, m: LMessage| LHandleAcceptRequest(s, s_, c, i, m)
    ||| exists |i: int, m: LMessage| LHandleAcceptResponse(s, s_, c, i, m)
    ||| exists |i: int| LCompleteAccept(s, s_, c, i)
    // Resubmit & Finish
    ||| exists |i: int| LResubmit(s, s_, c, i)
    ||| exists |i: int| LCompleteResubmit(s, s_, c, i)
    ||| exists |i: int| LFinishRecovery(s, s_, c, i)
    ||| exists |i: int, m: LMessage| LHandleFinishRecovery(s, s_, c, i, m)
}

// =========================================================================
// Safety Properties
// =========================================================================

// TLA+: CommittedLogAgreement
// Per-proposer committed log entries agree across servers.
pub open spec fn LCommittedLogAgreement(s: LState, c: LConstants) -> bool {
    forall |i: int, i2: int, p: int|
        c.server.contains(i) && c.server.contains(i2) && c.proposer.contains(p)
        ==> {
            let ci = s.commit_index[i][p];
            let ci2 = s.commit_index[i2][p];
            let limit = if ci <= ci2 { ci } else { ci2 };
            forall |k: int| 0 <= k < limit ==> {
                &&& s.log[i][p][k].term == s.log[i2][p][k].term
                &&& s.log[i][p][k].value == s.log[i2][p][k].value
            }
        }
}

// TLA+: LogOrderMatchesExecution
// Per-sequence committed log order matches execution order for conflicting commands.
pub open spec fn LLogOrderMatchesExecution(s: LState, c: LConstants) -> bool {
    forall |i: int, p: int|
        c.server.contains(i) && c.proposer.contains(p)
        ==> {
            let ci = s.commit_index[i][p];
            let cmd_seq = LFilterNoOps(
                Seq::new(ci, |k: int| s.log[i][p][k].value),
                c.no_op_cmd,
            );
            LConflictOrderPreserved(cmd_seq, LFilterNoOps(s.execution_cmds, c.no_op_cmd))
        }
}

// TLA+: ExecutionDedupMatches
// Conflict order between deduplicated original and replicated execution traces.
pub open spec fn LExecutionDedupMatches(s: LState, c: LConstants) -> bool {
    let orig_dedup = LDedup(LFilterNoOps(s.original_execution_cmds, c.no_op_cmd));
    let exec_dedup = LDedup(LFilterNoOps(s.execution_cmds, c.no_op_cmd));
    &&& LConflictOrderPreserved(orig_dedup, exec_dedup)
    &&& LConflictOrderPreserved(exec_dedup, orig_dedup)
}

// TLA+: MultiSequenceLogAgreement == CommittedLogAgreement (backward compat alias)
pub open spec fn LMultiSequenceLogAgreement(s: LState, c: LConstants) -> bool {
    LCommittedLogAgreement(s, c)
}

} // verus!
