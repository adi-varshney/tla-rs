# Jetpack Proof Status

**Date**: 2026-03-08
**Codebase**: `jetpack/refinement_proof/`
**Status**: Scaffolded -- all lemmas have `assume(false)`, no proofs discharged yet.

## 1. What Is Proved

Nothing is proved yet. All proof lemmas contain `assume(false)` placeholders.

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

| Category | Invariant | Status |
|----------|-----------|--------|
| Well-formedness | `TypeInvariant` | Defined, assume(false) |
| Log/commit bounds | `CommitIndexBounded` | Defined, assume(false) |
| Log/commit bounds | `LogTermsNonNegative` | Defined, assume(false) |
| Epoch monotonicity | `JEpochGeqOEpoch` | Defined, assume(false) |
| Epoch monotonicity | `CurrentTermPositive` | Defined, assume(false) |
| Message provenance | `MessageMultiplicityNonNeg` | Defined, assume(false) |
| Message provenance | `PreacceptRequestProvenance` | Defined, assume(false) |
| Message provenance | `BeginRecoveryRequestProvenance` | Defined, assume(false) |
| Message provenance | `PrepareRequestProvenance` | Defined, assume(false) |
| Message provenance | `AcceptRequestProvenance` | Defined, assume(false) |
| Message provenance | `FinishRecoveryRequestProvenance` | Defined, assume(false) |
| Execution trace | `ExecutionCmdsWellFormed` | Defined, assume(false) |
| Execution trace | `OriginalExecutionCmdsWellFormed` | Defined, assume(false) |
| Cmd-ID uniqueness | `CommittedCmdIdsUnique` | Defined, assume(false) |
| JPool | `JPoolKeysValid` | Defined, assume(false) |
| JPool | `JPoolBallotOrdering` | Defined, assume(false) |
| Recovery | `ReadyImpliesEpochsEqual` | Defined, assume(false) |

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

All proof bodies contain `assume(false)`. No intermediate assumptions are
isolated -- the entire proof is placeholder.

Total `assume(false)` count: **11** (across 4 files)

| File | Lemma | assume(false) count |
|------|-------|---------------------|
| invariants.rs | `lemma_init_establishes_invariant` | 1 |
| invariants.rs | `lemma_safety_invariant_inductive` | 1 |
| invariants.rs | `lemma_type_invariant_inductive` | 1 |
| invariants.rs | `lemma_commit_index_bounded_inductive` | 1 |
| invariants.rs | `lemma_jepoch_geq_oepoch_inductive` | 1 |
| invariants.rs | `lemma_jpool_ballot_ordering_inductive` | 1 |
| invariants.rs | `lemma_committed_cmd_ids_unique_inductive` | 1 |
| invariants.rs | `lemma_committed_log_agreement_inductive` | 1 |
| invariants.rs | `lemma_log_order_matches_execution_inductive` | 1 |
| invariants.rs | `lemma_execution_dedup_matches_inductive` | 1 |
| induction.rs | `lemma_invariant_holds_throughout_behavior` (2x) | 2 |
| refinement.rs | `lemma_refinement_correct` | 1 |

## 4. Technical Blockers Per Lemma

### 4.1 `lemma_init_establishes_invariant`
**Difficulty**: Low
**Blocker**: None -- straightforward from LInit definitions.
**Next step**: Expand LInit precondition, verify each conjunct of
JetpackSafetyInvariant holds for initial values (empty maps, epoch=1, etc.).

### 4.2 `lemma_safety_invariant_inductive`
**Difficulty**: High
**Blocker**: Requires case-splitting over 19 action branches and proving
all 17+ invariant conjuncts are preserved. This is the bulk of the proof work.
**Next step**: Start with TypeInvariant preservation (easiest), then
CommitIndexBounded (moderate), then epoch invariants, then tackle the
named safety properties last.

### 4.3 `lemma_committed_log_agreement_inductive`
**Difficulty**: Very high
**Blocker**: Requires a quorum intersection lemma for fast-path commits.
The key insight is that any two fast-path quorums (3/4 majority) for the
same proposer and epoch must overlap, so committed entries at the same
position have the same term and value.
**Next step**: Formalize the fast-path quorum intersection lemma as a
standalone helper, then use it to prove the inductive case for
HandlePreacceptResponse (the action that advances commit_index).

### 4.4 `lemma_log_order_matches_execution_inductive`
**Difficulty**: Very high
**Blocker**: Requires formalization of conflict-order preservation
through the recovery protocol. The LFinishRecovery and LCompleteResubmit
actions reconstruct the execution trace, and the proof must show that
the reconstruction preserves conflict order with the committed log.
**Next step**: Formalize the `LConflictOrderPreserved` predicate's
properties as standalone lemmas, then prove preservation through each
action that modifies `execution_cmds`.

### 4.5 `lemma_execution_dedup_matches_inductive`
**Difficulty**: Very high
**Blocker**: Requires formalization of dedup/conflict-order interaction.
`LDedup` is a recursive sequence operation, and the proof must show that
deduplication preserves conflict order in both directions between
`original_execution_cmds` and `execution_cmds`.
**Next step**: Prove standalone properties of `LDedup` (idempotence,
conflict-order preservation), then use them in the inductive proof.

### 4.6 `lemma_refinement_correct`
**Difficulty**: Moderate (given the above)
**Blocker**: Depends on all three named property proofs being complete.
**Next step**: Once the safety invariant is fully proved, the refinement
theorem follows by showing the abstraction map is consistent.

## 5. Recommended Proof Engineering Order

1. **TypeInvariant preservation** -- straightforward, builds confidence
2. **CommitIndexBounded preservation** -- moderate, good warm-up
3. **Epoch invariants** (JEpochGeqOEpoch, CurrentTermPositive) -- moderate
4. **Message provenance invariants** -- many cases but individually simple
5. **JPool invariants** -- moderate
6. **ExecutionCmds/OriginalExecutionCmds well-formedness** -- moderate
7. **CommittedCmdIdsUnique** -- harder, needs log append analysis
8. **ReadyImpliesEpochsEqual** -- moderate, needs recovery action analysis
9. **CommittedLogAgreement** -- hardest, needs quorum intersection
10. **LogOrderMatchesExecution** -- hardest, needs conflict-order analysis
11. **ExecutionDedupMatches** -- hardest, needs dedup properties
12. **Refinement theorem** -- follows from above

## 6. Comparison with Raft Proof

| Aspect | Raft | Jetpack |
|--------|------|---------|
| Core invariants | 4 (ElectionSafety, LogMatching, LeaderCompleteness, SMS) | 3 (CommittedLogAgreement, LogOrderMatchesExecution, ExecutionDedupMatches) |
| Support invariants | 8+ structural + 6 message | 17 across 8 categories |
| Action branches | ~10 | 19 |
| Proof LOC (current) | ~10K (invariants.rs alone) | ~350 (skeleton) |
| assume(false) remaining | 12 | 11 |
| Hardest lemma | LeaderCompleteness | CommittedLogAgreement (quorum intersection) |
| Novel difficulty | Vote provenance chain | 3-D log + conflict-order + dedup |

## 7. Files

- `jetpack/refinement_proof/state_machine.rs` -- distributed state model
- `jetpack/refinement_proof/invariants.rs` -- invariant definitions + proof skeletons
- `jetpack/refinement_proof/induction.rs` -- behavior-level induction
- `jetpack/refinement_proof/refinement.rs` -- refinement theorem
- `jetpack/proof_status.md` -- this file
