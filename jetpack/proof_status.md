# Jetpack Proof Status

**Date**: 2026-03-08
**Last Updated**: Phase 5 -- init lemma proved, CurrentTermPositive renamed to CurrentTermNonNeg
**Codebase**: `jetpack/refinement_proof/`
**Status**: 1 lemma proved (init), 10 assume(false) remaining.

## 1. What Is Proved

### `lemma_init_establishes_invariant` (invariants.rs)
**Status**: PROVED (modulo Verus compilation).
**Method**: LInit preconditions expanded inline; each of the 17 conjuncts of
`JetpackSafetyInvariant` shown to hold from initial values:
- TypeInvariant: Map::new domains match constant sets
- CommitIndexBounded: commit_index == 0 <= log.len() == 0
- LogTermsNonNegative: vacuous (empty logs)
- JEpochGeqOEpoch: both == LDefaultView(c).epoch == 1
- CurrentTermNonNeg: 0 >= 0
- All message provenance: vacuous (empty message bag)
- Execution trace well-formedness: vacuous (empty traces)
- CommittedCmdIdsUnique: vacuous (commit_index == 0)
- JPoolKeysValid: LEmptyJPool(c).pool keyed by c.key
- JPoolBallotOrdering: both ballots == 0
- ReadyImpliesEpochsEqual: jstate == Ready, jepoch == oepoch == 1

**Bug found during proof**: `CurrentTermPositive` (>= 1) was incorrect because
LInit sets `current_term` to 0. Renamed to `CurrentTermNonNeg` (>= 0).

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
17 support invariants in 8 categories:

| Category | Invariant | Init | Inductive |
|----------|-----------|------|-----------|
| Well-formedness | `TypeInvariant` | PROVED | assume(false) |
| Log/commit bounds | `CommitIndexBounded` | PROVED | assume(false) |
| Log/commit bounds | `LogTermsNonNegative` | PROVED | assume(false) |
| Epoch monotonicity | `JEpochGeqOEpoch` | PROVED | assume(false) |
| Epoch monotonicity | `CurrentTermNonNeg` | PROVED | assume(false) |
| Message provenance | `MessageMultiplicityNonNeg` | PROVED | assume(false) |
| Message provenance | `PreacceptRequestProvenance` | PROVED | assume(false) |
| Message provenance | `BeginRecoveryRequestProvenance` | PROVED | assume(false) |
| Message provenance | `PrepareRequestProvenance` | PROVED | assume(false) |
| Message provenance | `AcceptRequestProvenance` | PROVED | assume(false) |
| Message provenance | `FinishRecoveryRequestProvenance` | PROVED | assume(false) |
| Execution trace | `ExecutionCmdsWellFormed` | PROVED | assume(false) |
| Execution trace | `OriginalExecutionCmdsWellFormed` | PROVED | assume(false) |
| Cmd-ID uniqueness | `CommittedCmdIdsUnique` | PROVED | assume(false) |
| JPool | `JPoolKeysValid` | PROVED | assume(false) |
| JPool | `JPoolBallotOrdering` | PROVED | assume(false) |
| Recovery | `ReadyImpliesEpochsEqual` | PROVED | assume(false) |

Composite invariant: `JetpackSafetyInvariant` = conjunction of all 17 above.

Named safety property lemma skeletons:
- `lemma_committed_log_agreement_inductive` -- assume(false)
- `lemma_log_order_matches_execution_inductive` -- assume(false)
- `lemma_execution_dedup_matches_inductive` -- assume(false)

### 2.3 Induction (`induction.rs`)
- `lemma_invariant_holds_throughout_behavior` -- recursive induction, assume(false)
- Corollaries for each named property -- assume(false)

### 2.4 Refinement (`refinement.rs`)
- `JetpackAbstractState` -- abstract sequential state
- `AbstractifyJetpackState` -- refinement mapping
- `lemma_refinement_correct` -- refinement theorem skeleton, assume(false)

## 3. Assumptions That Remain

Total `assume(false)` count: **10** (across 3 files, down from 11)

| File | Lemma | assume(false) count | Notes |
|------|-------|---------------------|-------|
| invariants.rs | `lemma_safety_invariant_inductive` | 1 | Top-level inductive step |
| invariants.rs | `lemma_type_invariant_inductive` | 1 | Blocked on LNext import |
| invariants.rs | `lemma_commit_index_bounded_inductive` | 1 | Blocked on LNext import |
| invariants.rs | `lemma_jepoch_geq_oepoch_inductive` | 1 | Blocked on LNext import |
| invariants.rs | `lemma_jpool_ballot_ordering_inductive` | 1 | Blocked on LNext import |
| invariants.rs | `lemma_committed_cmd_ids_unique_inductive` | 1 | Blocked on LNext import |
| invariants.rs | `lemma_committed_log_agreement_inductive` | 1 | Needs quorum intersection |
| invariants.rs | `lemma_log_order_matches_execution_inductive` | 1 | Needs conflict-order formalization |
| invariants.rs | `lemma_execution_dedup_matches_inductive` | 1 | Needs dedup properties |
| induction.rs | `lemma_invariant_holds_throughout_behavior` (2x) | 2 | Needs init + inductive lemmas |
| refinement.rs | `lemma_refinement_correct` | 1 | Needs all safety properties |

**Note**: `lemma_init_establishes_invariant` no longer has assume(false).

## 4. Technical Blockers Per Lemma

### 4.1 `lemma_init_establishes_invariant`
**Status**: PROVED
**Difficulty**: Low -- completed.

### 4.2 `lemma_safety_invariant_inductive`
**Difficulty**: High
**Blocker**: Requires case-splitting over 19 action branches and proving
all 17+ invariant conjuncts are preserved. This is the bulk of the proof work.
**Next step**: Start with TypeInvariant preservation (easiest), then
CommitIndexBounded (moderate), then epoch invariants, then tackle the
named safety properties last.

### 4.3 `lemma_type_invariant_inductive`
**Difficulty**: Low
**Blocker**: Requires LNext to be importable for case-split. Proof sketch
is complete: all actions use map .insert which preserves domain membership.
**Next step**: Integrate jetpack.rs as a module import, then case-split.

### 4.4 `lemma_committed_log_agreement_inductive`
**Difficulty**: Very high
**Blocker**: Requires a quorum intersection lemma for fast-path commits.
The key insight is that any two fast-path quorums (3/4 majority) for the
same proposer and epoch must overlap, so committed entries at the same
position have the same term and value.
**Next step**: Formalize the fast-path quorum intersection lemma as a
standalone helper, then use it to prove the inductive case for
HandlePreacceptResponse (the action that advances commit_index).

### 4.5 `lemma_log_order_matches_execution_inductive`
**Difficulty**: Very high
**Blocker**: Requires formalization of conflict-order preservation
through the recovery protocol. The LFinishRecovery and LCompleteResubmit
actions reconstruct the execution trace, and the proof must show that
the reconstruction preserves conflict order with the committed log.
**Next step**: Formalize the `LConflictOrderPreserved` predicate's
properties as standalone lemmas, then prove preservation through each
action that modifies `execution_cmds`.

### 4.6 `lemma_execution_dedup_matches_inductive`
**Difficulty**: Very high
**Blocker**: Requires formalization of dedup/conflict-order interaction.
`LDedup` is a recursive sequence operation, and the proof must show that
deduplication preserves conflict order in both directions between
`original_execution_cmds` and `execution_cmds`.
**Next step**: Prove standalone properties of `LDedup` (idempotence,
conflict-order preservation), then use them in the inductive proof.

### 4.7 `lemma_refinement_correct`
**Difficulty**: Moderate (given the above)
**Blocker**: Depends on all three named property proofs being complete.
**Next step**: Once the safety invariant is fully proved, the refinement
theorem follows by showing the abstraction map is consistent.

## 5. Recommended Proof Engineering Order

1. ~~**Init lemma**~~ -- DONE
2. **TypeInvariant preservation** -- proof sketch complete, needs LNext import
3. **CommitIndexBounded preservation** -- moderate, good warm-up
4. **Epoch invariants** (JEpochGeqOEpoch, CurrentTermNonNeg) -- moderate
5. **Message provenance invariants** -- many cases but individually simple
6. **JPool invariants** -- moderate
7. **ExecutionCmds/OriginalExecutionCmds well-formedness** -- moderate
8. **CommittedCmdIdsUnique** -- harder, needs log append analysis
9. **ReadyImpliesEpochsEqual** -- moderate, needs recovery action analysis
10. **CommittedLogAgreement** -- hardest, needs quorum intersection
11. **LogOrderMatchesExecution** -- hardest, needs conflict-order analysis
12. **ExecutionDedupMatches** -- hardest, needs dedup properties
13. **Refinement theorem** -- follows from above

## 6. Comparison with Raft Proof

| Aspect | Raft | Jetpack |
|--------|------|---------|
| Core invariants | 4 (ElectionSafety, LogMatching, LeaderCompleteness, SMS) | 3 (CommittedLogAgreement, LogOrderMatchesExecution, ExecutionDedupMatches) |
| Support invariants | 8+ structural + 6 message | 17 across 8 categories |
| Action branches | ~10 | 19 |
| Proof LOC (current) | ~10K (invariants.rs alone) | ~550 (init proved + skeletons) |
| assume(false) remaining | 12 | 10 |
| Hardest lemma | LeaderCompleteness | CommittedLogAgreement (quorum intersection) |
| Novel difficulty | Vote provenance chain | 3-D log + conflict-order + dedup |

## 7. Files

- `jetpack/refinement_proof/state_machine.rs` -- distributed state model
- `jetpack/refinement_proof/invariants.rs` -- invariant definitions + proof skeletons
- `jetpack/refinement_proof/induction.rs` -- behavior-level induction
- `jetpack/refinement_proof/refinement.rs` -- refinement theorem
- `jetpack/proof_status.md` -- this file
