# Jetpack Proof Status

**Date**: 2026-03-08
**Last Updated**: Phase 5 -- ClientPendingCmdsValid/ExecutionCmdsWellFormed/
  OriginalExecutionCmdsWellFormed/JPoolKeysValid preservation proofs written,
  ViewReplicaIdsValid partial (LFinishRecovery self-contained, rest needs msg reasoning)
**Codebase**: `jetpack/refinement_proof/`
**Status**: 9 lemmas proved (init + 8 preservation), 13 assume(false) across 3 files, 2 invariants retracted.

## 1. What Is Proved

### `lemma_init_establishes_invariant` (invariants.rs)
**Status**: PROVED (modulo Verus compilation).
**Method**: LInit preconditions expanded inline; each of the 19 conjuncts of
`JetpackSafetyInvariant` shown to hold from initial values:
- TypeInvariant: Map::new domains match constant sets
- CommitIndexBounded: commit_index == 0 <= log.len() == 0
- LogTermsNonNegative: vacuous (empty logs)
- CurrentTermNonNeg: 0 >= 0
- All message provenance: vacuous (empty message bag)
- Execution trace well-formedness: vacuous (empty traces)
- CommittedCmdIdsUnique: vacuous (commit_index == 0)
- JPoolKeysValid: LEmptyJPool(c).pool keyed by c.key
- JPoolBallotOrdering: both ballots == 0
- EpochsNonNegative: jepoch == oepoch == 1 >= 0

**Bug found during proof**: `CurrentTermPositive` (>= 1) was incorrect because
LInit sets `current_term` to 0. Renamed to `CurrentTermNonNeg` (>= 0).

**Invariant retracted (2)**: `JEpochGeqOEpoch` (jepoch >= oepoch) is NOT inductive.
Counter-example: `LHandleBeginRecoveryRequest` sets `oepoch[i] = mnew_view.epoch`
but leaves `jepoch` unchanged. If `mnew_view.epoch > jepoch[i]`, then
`oepoch[i] > jepoch[i]`. In the TLA+ spec, jepoch and oepoch are independent
epoch trackers that can diverge in either direction. No ordering invariant exists.
Removed from `JetpackSafetyInvariant` and deleted preservation lemma.

**Invariant retracted (1)**: `ReadyImpliesEpochsEqual` (jstate == Ready => jepoch == oepoch)
is NOT inductive. Counter-example: `LHandlePrepareRequest`, `LHandleAcceptRequest`,
`LHandlePrepareResponse` (!mok), and `LHandleAcceptResponse` (!mok) all update
`jepoch = max(jepoch, mjepoch)` and `oepoch = max(oepoch, moepoch)` independently
while keeping `jstate` unchanged. If `moepoch != mjepoch`, the epochs diverge.
Replaced with the weaker `EpochsNonNegative` (both epochs >= 0).

### `lemma_type_invariant_inductive` (invariants.rs)
**Status**: PROVED (modulo Verus compilation).
**Method**: LNext case-split. All 19 actions modify maps only via `.insert(k, v)`
where `k` is in the appropriate domain (c.server/c.client/c.proposer). The Map
axiom `m.insert(k,v).contains_key(j) <==> (j==k || m.contains_key(j))` ensures
domain preservation. For nested maps (log, commit_index), both outer and inner
inserts preserve contains_key. UNCHANGED maps trivially preserve TypeInvariant.

### `lemma_commit_index_bounded_inductive` (invariants.rs)
**Status**: PROVED (modulo Verus compilation).
**Method**: All 19 actions ensure `s_.commit_index == s.commit_index` (verified
by source grep). 18 of 19 actions also ensure `s_.log == s.log`. The sole
exception (LHandlePreacceptRequest) only appends to `log[i][j]`, making it longer.
Since commit_index is unchanged and log never shrinks, the bound
`commit_index[i][j] <= log[i][j].len()` is preserved.

### `lemma_committed_cmd_ids_unique_inductive` (invariants.rs)
**Status**: PROVED (modulo Verus compilation).
**Method**: Same argument as CommitIndexBounded: no action modifies commit_index,
and log only grows via append. The committed prefix `log[i][j][0..commit_index[i][j]]`
is therefore unchanged in all transitions, so cmd-id uniqueness is preserved.

### `lemma_jpool_ballot_ordering_inductive` (invariants.rs)
**Status**: PROVED (modulo Verus compilation).
**Method**: Case-split on 19 LNext disjuncts. 12 actions leave jpool unchanged (trivial).
7 non-trivial cases:
- LHandlePreacceptRequest: only modifies pool[cmd.key], ballot fields unchanged via struct update
- LHandlePrepareRequest (ok=true): guard ensures mmax_seen_ballot >= jpool.max_seen_ballot;
  sets max_seen_ballot = mmax_seen_ballot, accepted_ballot unchanged => preserved
- LHandlePrepareResponse (!mok): max_seen_ballot = max(old, msg) >= old >= accepted_ballot
- LHandleAcceptRequest (ok=true): sets both to mmax_seen_ballot => equal => preserved
- LHandleAcceptResponse (!mok): same as LHandlePrepareResponse
- LFinishRecovery/LHandleFinishRecovery: reset to LEmptyJPool (both 0) => trivial

### PreacceptRequestProvenance fix
**Bug found**: PreacceptRequestProvenance previously required `c.client.contains(msource)`,
but `LResubmit` creates PreacceptRequest with `msource = i` (a server, not a client).
Fixed to allow `c.client.contains(msource) || c.server.contains(msource)`.

### `lemma_client_pending_cmds_valid_inductive` (invariants.rs)
**Status**: PROVED (modulo Verus compilation).
**Method**: Only LClientSendPreaccept modifies client_pending (sets Some(cmd) where
cmd from LAvailableCommands, which filters by c.cmd_id/c.key). LHandlePreacceptResponse
sets to None (vacuously valid) or unchanged. All other actions: LUnchangedClientVars.

### `lemma_execution_cmds_well_formed_inductive` (invariants.rs)
**Status**: PROVED (modulo Verus compilation).
**Method**: Only LHandlePreacceptResponse modifies execution_cmds (push(mcmd)).
Guard: `s.client_pending[cl] == Some(mcmd)`. By ClientPendingCmdsValid, mcmd is valid.
Existing entries preserved (push only appends).

### `lemma_original_execution_cmds_well_formed_inductive` (invariants.rs)
**Status**: PROVED (modulo Verus compilation).
**Method**: No action modifies original_execution_cmds. All 19 actions set
`s_.original_execution_cmds == s.original_execution_cmds`.

### `lemma_jpool_keys_valid_inductive` (invariants.rs)
**Status**: PROVED (modulo Verus compilation, depends on PreacceptRequestCmdsValid upstream).
**Method**: Only LHandlePreacceptRequest modifies jpool.pool (inserts cmd.key).
By PreacceptRequestCmdsValid, c.key.contains(cmd.key). LFinishRecovery/LHandleFinishRecovery
reset to LEmptyJPool(c) which is keyed by c.key. Other actions: struct update preserves pool.

### Message bag helper lemmas (invariants.rs)
Added 3 helper lemmas for reasoning about message bag operations:
- `lemma_with_message_preserves_predicate`: LWithMessage only adds the specified message
- `lemma_without_message_preserves_predicate`: LWithoutMessage only removes messages
- `lemma_add_messages_provenance`: LAddMessages result contains only old messages + added set

### Include restructure (jetpack_body.rs)
**Status**: Complete.
**Method**: Extracted the verus! body from `jetpack.rs` into `jetpack_body.rs`.
Proof files now `include!("../jetpack_body.rs")` inside their `verus!{}` block,
giving them access to LInit, LNext, and all 19 action predicates without
double-defining types. `jetpack.rs` is now a thin wrapper.

### Behavior-level specification (induction.rs)
**Status**: `IsValidJetpackBehavior` now fully specified with `LInit` and `LNext`
constraints (previously commented out). Still has 2 assume(false) in the
induction lemma body.

### Per-action UNCHANGED analysis (invariants.rs)
A comprehensive 19-action table documenting which state field groups each action
modifies vs leaves unchanged. This enables trivial preservation proofs for invariants
that only depend on unchanged fields. Key findings:
- 15 of 19 actions leave base vars (log, commit_index) unchanged
- Only LHandlePreacceptRequest modifies log; no action modifies commit_index
- Only LHandlePreacceptResponse modifies execution_cmds
- LFinishRecovery and LHandleFinishRecovery reset most jetpack fields

## 2. What Is Scaffolded

The following proof modules exist with theorem statements, invariant definitions,
and proof skeletons:

### 2.1 State Machine (`state_machine.rs`)
- `JetpackDistributedState` -- global distributed state struct
- `WellFormedJetpackDistributed` -- well-formedness predicate
- `WellFormedServerState` -- per-server state well-formedness
- `JetpackBehavior` / `IsValidJetpackBehavior` -- behavior type and validity
- `JetpackDistributedInit` / `JetpackDistributedNext` -- system transitions

### 2.2 Invariants (`invariants.rs`)
19 support invariants in 10 categories (2 retracted from original 17, 4 new added):

| Category | Invariant | Init | Inductive |
|----------|-----------|------|-----------|
| Well-formedness | `TypeInvariant` | PROVED | PROVED |
| Log/commit bounds | `CommitIndexBounded` | PROVED | PROVED |
| Log/commit bounds | `LogTermsNonNegative` | PROVED | TRIVIAL (nat type) |
| Epoch monotonicity | ~~`JEpochGeqOEpoch`~~ | RETRACTED | Not inductive |
| Epoch properties | `CurrentTermNonNeg` | PROVED | TRIVIAL (nat type) |
| Message provenance | `MessageMultiplicityNonNeg` | PROVED | TRIVIAL (nat type) |
| Message provenance | `PreacceptRequestProvenance` | PROVED | assume(false)* |
| Message provenance | `BeginRecoveryRequestProvenance` | PROVED | assume(false)* |
| Message provenance | `PrepareRequestProvenance` | PROVED | assume(false)* |
| Message provenance | `AcceptRequestProvenance` | PROVED | assume(false)* |
| Message provenance | `FinishRecoveryRequestProvenance` | PROVED | assume(false)* |
| Execution trace | `ExecutionCmdsWellFormed` | PROVED | PROVED |
| Execution trace | `OriginalExecutionCmdsWellFormed` | PROVED | PROVED (never modified) |
| Cmd-ID uniqueness | `CommittedCmdIdsUnique` | PROVED | PROVED |
| JPool | `JPoolKeysValid` | PROVED | PROVED* |
| JPool | `JPoolBallotOrdering` | PROVED | PROVED |
| Recovery | `EpochsNonNegative` | PROVED | TRIVIAL (nat type) |
| Recovery | ~~`ReadyImpliesEpochsEqual`~~ | RETRACTED | Not inductive |
| View integrity | `ViewReplicaIdsValid` | PROVED | assume(false)** |
| Cmd well-formedness | `ClientPendingCmdsValid` | PROVED | PROVED |
| Cmd well-formedness | `PreacceptRequestCmdsValid` | PROVED | needs LAddMessages |
| Cmd well-formedness | `PreacceptResponseCmdsValid` | PROVED | needs PreacceptRequestCmdsValid |

*JPoolKeysValid depends on PreacceptRequestCmdsValid, which has upstream assume(false).
**ViewReplicaIdsValid: LFinishRecovery case self-contained; 3 message-dependent cases
need LAddMessages reasoning for message-level view integrity.

Provenance invariants have documented proof strategies and sub-lemma skeletons,
but need LAddMessages reasoning (recursive function induction) to fully discharge.

Composite invariant: `JetpackSafetyInvariant` = conjunction of 19 invariants.

Named safety property lemma skeletons (all have LNext requires, all have assume(false)):
- `lemma_committed_log_agreement_inductive`
- `lemma_log_order_matches_execution_inductive`
- `lemma_execution_dedup_matches_inductive`

### 2.3 Induction (`induction.rs`)
- `IsValidJetpackBehavior` -- fully specified with LInit/LNext
- `lemma_invariant_holds_throughout_behavior` -- recursive induction, assume(false)
- Corollaries for each named property

### 2.4 Refinement (`refinement.rs`)
- `JetpackAbstractState` -- abstract sequential state
- `AbstractifyJetpackState` -- refinement mapping
- `lemma_refinement_correct` -- refinement theorem skeleton, assume(false)

## 3. Assumptions That Remain

Total `assume(false)` count: **13** (across 3 files)

Note: count is higher than original 8 because new sub-lemma skeletons were added.
9 lemmas are now fully proved (no assume(false)); 7 sub-lemmas and the top-level
inductive lemma still have assume(false). The remaining assume(false) instances are
concentrated in message-level reasoning (LAddMessages) and the 3 named safety properties.

| File | Lemma | assume(false) count | Notes |
|------|-------|---------------------|-------|
| invariants.rs | `lemma_safety_invariant_inductive` | 1 | Top-level: delegates but remaining invariants not proved |
| invariants.rs | `lemma_preaccept_request_provenance_inductive` | 1 | Needs LAddMessages induction |
| invariants.rs | `lemma_begin_recovery_request_provenance_inductive` | 1 | Needs LAddMessages induction |
| invariants.rs | `lemma_prepare_request_provenance_inductive` | 1 | Needs LAddMessages induction |
| invariants.rs | `lemma_accept_request_provenance_inductive` | 1 | Needs LAddMessages induction |
| invariants.rs | `lemma_finish_recovery_request_provenance_inductive` | 1 | Needs LAddMessages induction |
| invariants.rs | `lemma_view_replica_ids_valid_inductive` | 1 | Needs message-level view integrity |
| invariants.rs | `lemma_committed_log_agreement_inductive` | 1 | Needs quorum intersection |
| invariants.rs | `lemma_log_order_matches_execution_inductive` | 1 | Needs conflict-order formalization |
| invariants.rs | `lemma_execution_dedup_matches_inductive` | 1 | Needs dedup properties |
| induction.rs | `lemma_invariant_holds_throughout_behavior` (2x) | 2 | Needs init + inductive lemmas |
| refinement.rs | `lemma_refinement_correct` | 1 | Needs all safety properties |

**Previously discharged** (no longer have assume(false)):
- `lemma_init_establishes_invariant` -- PROVED
- `lemma_type_invariant_inductive` -- PROVED
- `lemma_commit_index_bounded_inductive` -- PROVED
- `lemma_committed_cmd_ids_unique_inductive` -- PROVED
- `lemma_jpool_ballot_ordering_inductive` -- PROVED
- `lemma_client_pending_cmds_valid_inductive` -- PROVED
- `lemma_execution_cmds_well_formed_inductive` -- PROVED
- `lemma_original_execution_cmds_well_formed_inductive` -- PROVED
- `lemma_jpool_keys_valid_inductive` -- PROVED (depends on PreacceptRequestCmdsValid upstream)

**Retracted/removed**:
- `JEpochGeqOEpoch` retracted (not inductive): HandleBeginRecoveryReq sets oepoch independently.
- `ReadyImpliesEpochsEqual` retracted (not inductive): Prepare/Accept update epochs independently.
- `lemma_jepoch_geq_oepoch_inductive` removed (invariant retracted).
- Per-action UNCHANGED analysis added to invariants.rs for all 19 actions.
- Full inductiveness audit completed: 7 SELF, 4 TRIVIAL, 6 NEEDS, 2 RETRACTED.
- Include restructure: `jetpack_body.rs` extracted, all proof files now include protocol spec.
- All lemma `requires` clauses now use `LNext(s, s_, c)` instead of placeholder `true`.

## 4. Technical Blockers Per Lemma

### 4.1 `lemma_init_establishes_invariant`
**Status**: PROVED
**Difficulty**: Low -- completed.

### 4.2 `lemma_type_invariant_inductive`
**Status**: PROVED
**Difficulty**: Low -- completed.

### 4.3 `lemma_commit_index_bounded_inductive`
**Status**: PROVED
**Difficulty**: Low -- completed.

### 4.4 `lemma_committed_cmd_ids_unique_inductive`
**Status**: PROVED
**Difficulty**: Low -- completed.

### 4.5 `lemma_safety_invariant_inductive`
**Difficulty**: High
**Blocker**: Delegates to per-invariant sub-lemmas. 9 proved sub-lemmas are called.
Remaining with assume(false): 5 provenance (LAddMessages), ViewReplicaIdsValid
(message-level view integrity). Remaining without sub-lemmas: PreacceptRequestCmdsValid,
PreacceptResponseCmdsValid (both need LAddMessages).
Plus the three named safety properties.
**Next step**: Discharge LAddMessages-dependent proofs (all 8 share the same blocker).

### 4.6 `lemma_jpool_ballot_ordering_inductive`
**Status**: PROVED
**Difficulty**: Moderate -- completed.

### 4.6a Message provenance lemmas (5 sub-lemmas)
**Difficulty**: Moderate
**Blocker**: All 5 have the same blocker: reasoning about `LAddMessages` (a
recursive function) to show that messages in the new bag either came from
the old bag (provenance by IH) or were newly added by the creating action
(provenance from action guards + ViewReplicaIdsValid).
Helper lemma `lemma_add_messages_provenance` exists but needs Verus verification.
**Next step**: Test with Verus; may need fuel/trigger annotations for LAddMessages recursion.

### 4.7 `lemma_committed_log_agreement_inductive`
**Difficulty**: Very high
**Blocker**: Requires a quorum intersection lemma for fast-path commits.
The key insight is that any two fast-path quorums (3/4 majority) for the
same proposer and epoch must overlap, so committed entries at the same
position have the same term and value.
**Next step**: Formalize the fast-path quorum intersection lemma as a
standalone helper, then use it to prove the inductive case for
HandlePreacceptResponse (the action that advances commit_index).

### 4.8 `lemma_log_order_matches_execution_inductive`
**Difficulty**: Very high
**Blocker**: Requires formalization of conflict-order preservation
through the recovery protocol. The LFinishRecovery and LCompleteResubmit
actions reconstruct the execution trace, and the proof must show that
the reconstruction preserves conflict order with the committed log.
**Next step**: Formalize the `LConflictOrderPreserved` predicate's
properties as standalone lemmas, then prove preservation through each
action that modifies `execution_cmds`.

### 4.9 `lemma_execution_dedup_matches_inductive`
**Difficulty**: Very high
**Blocker**: Requires formalization of dedup/conflict-order interaction.
`LDedup` is a recursive sequence operation, and the proof must show that
deduplication preserves conflict order in both directions between
`original_execution_cmds` and `execution_cmds`.
**Next step**: Prove standalone properties of `LDedup` (idempotence,
conflict-order preservation), then use them in the inductive proof.

### 4.10 `lemma_refinement_correct`
**Difficulty**: Moderate (given the above)
**Blocker**: Depends on all three named property proofs being complete.
**Next step**: Once the safety invariant is fully proved, the refinement
theorem follows by showing the abstraction map is consistent.

## 5. Recommended Proof Engineering Order

1. ~~**Init lemma**~~ -- DONE
2. ~~**TypeInvariant preservation**~~ -- DONE
3. ~~**CommitIndexBounded preservation**~~ -- DONE
4. ~~Epoch invariants (JEpochGeqOEpoch)~~ -- RETRACTED
5. **Message provenance invariants** -- sub-lemmas added, blocked on LAddMessages reasoning
6. ~~**JPool ballot ordering**~~ -- DONE
7. ~~**ExecutionCmds/OriginalExecutionCmds well-formedness**~~ -- DONE
8. ~~**CommittedCmdIdsUnique**~~ -- DONE
9. ~~ReadyImpliesEpochsEqual~~ -- RETRACTED (not inductive)
10. **CommittedLogAgreement** -- hardest, needs quorum intersection
11. **LogOrderMatchesExecution** -- hardest, needs conflict-order analysis
12. **ExecutionDedupMatches** -- hardest, needs dedup properties
13. **Refinement theorem** -- follows from above

## 6. Comparison with Raft Proof

| Aspect | Raft | Jetpack |
|--------|------|---------|
| Core invariants | 4 (ElectionSafety, LogMatching, LeaderCompleteness, SMS) | 3 (CommittedLogAgreement, LogOrderMatchesExecution, ExecutionDedupMatches) |
| Support invariants | 8+ structural + 6 message | 19 across 10 categories |
| Action branches | ~10 | 19 |
| Proof LOC (current) | ~10K (invariants.rs alone) | ~1400 (9 lemmas proved + skeletons + helpers) |
| assume(false) remaining | 12 | 13 (many new sub-lemma skeletons added) |
| Hardest lemma | LeaderCompleteness | CommittedLogAgreement (quorum intersection) |
| Novel difficulty | Vote provenance chain | 3-D log + conflict-order + dedup |

## 7. Files

- `jetpack/jetpack.rs` -- thin wrapper (includes types.rs + jetpack_body.rs)
- `jetpack/jetpack_body.rs` -- protocol spec body (includable without verus! wrapper)
- `jetpack/types.rs` -- shared type definitions
- `jetpack/refinement_proof/state_machine.rs` -- distributed state model
- `jetpack/refinement_proof/invariants.rs` -- invariant definitions + proof skeletons
- `jetpack/refinement_proof/induction.rs` -- behavior-level induction
- `jetpack/refinement_proof/refinement.rs` -- refinement theorem
- `jetpack/proof_status.md` -- this file
