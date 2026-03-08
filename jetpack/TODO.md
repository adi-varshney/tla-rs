# Jetpack TODO For Claude

This file is the work contract. Do not cut scope, silently weaken semantics, or
claim success based on partial translation. The target is the existing
`jetpack/jetpack.tla`, not the broader future architecture in
`jetpack/TLA_PLUS_BIG_PICTURE.md`.

Use this as a checklist. Mark boxes only when the artifact exists or the review
step was actually completed.

## Mission

- [ ] Translate `jetpack/jetpack.tla` into a `tla-rs` / Verus-style spec.
- [ ] Try source-first model checking on the translated Jetpack spec.
- [ ] Write real proof scaffolding for Jetpack safety properties.
- [ ] Record all work products inside the `jetpack/` folder.
- [ ] Treat `jetpack/TLA_PLUS_BIG_PICTURE.md` as context only, not as the
  implementation target for this task.

## Scope Constraints

- [ ] Keep all new files under `jetpack/` unless there is a concrete blocker
  that cannot be resolved without touching the main source tree.
- [ ] Avoid unrelated repo edits just to make Jetpack look integrated.
- [ ] Do not register a new protocol under `src/protocol/*` on the first pass.
- [ ] Do not modify `reports/model_check/*` on the first pass; keep Jetpack
  evidence local to `jetpack/`.
- [ ] Do not edit `jetpack/jetpack.tla` unless fixing an obvious typo.
- [ ] If tooling needs a normalized TLA input, create a clearly named copy such
  as `jetpack/jetpack_for_translate.tla` and explain exactly why it exists.
- [ ] If any work must go outside `jetpack/`, document:
  - [ ] the exact blocker
  - [ ] why a `jetpack/`-local workaround is insufficient
  - [ ] the smallest required external edit

## Non-Negotiable Semantics

- [ ] Treat `jetpack/jetpack.tla` as the source of truth.
- [ ] Preserve the genuine 3-D log model.
- [ ] Do not flatten the log to 2-D.
- [ ] Do not replace the 3-D log with a projection/refinement shortcut.
- [ ] Keep the per-proposer `commitIndex` structure.
- [ ] Preserve the Jetpack actions and safety properties that actually appear
  in `jetpack/jetpack.tla`.
- [ ] Do not replace message-bag semantics with an ad hoc queue unless you
  write down the semantics-preservation argument.
- [ ] Do not delete hard helpers such as `AddMessages`, `RemoveCmd`, `Dedup`,
  `IndexOf`, or the conflict-order logic just to get the translation through.
- [ ] Do not weaken the main properties into generic "sanity checks".
- [ ] Preserve these property names or very close equivalents:
  - [ ] `CommittedLogAgreement`
  - [ ] `LogOrderMatchesExecution`
  - [ ] `ExecutionDedupMatches`

## Read First

- [x] Read `jetpack/jetpack.tla`.
- [x] Read `jetpack/TLA_PLUS_BIG_PICTURE.md`.
- [x] Read `docs/tla-rs-guide.md`.
- [x] Read `docs/tla-to-verus-guide.md`.
- [x] Read `docs/conversion-testing-guide.md`.
- [x] Read `docs/model-checking-source-first.md`.
- [x] Read `docs/model_checker_status.md`.
- [x] Read `reports/raft_refinement_proof.md`.
- [x] Read `src/protocol/Raft/raft_refinement.rs`.
- [x] Read `src/protocol/Raft/refinement_proof/`.
- [x] Internalize these repo facts before doing Jetpack work:
  - [x] The repo has a direct `TLA+ -> Verus` path via
    `cargo run --manifest-path transpiler/Cargo.toml -- translate-tla ...`.
  - [x] The repo has a source-first model checker via
    `verus-transpile model-check --input ... --types ... --model ...`.
  - [x] The model checker is exact only when `state_dedup = "canonical"`.
  - [x] Real protocols like Raft still hit existential-expansion blockers.
  - [x] The TLA translator docs already warn about unsupported or fragile TLA
    patterns such as multi-line conjunction formatting, `RECURSIVE`,
    `INSTANCE`, temporal formulas, and proof syntax.

## Required Deliverables In `jetpack/`

- [x] Create `jetpack/translation_audit.md`.
- [x] Create `jetpack/jetpack.rs`.
- [x] Create `jetpack/types.rs`.
- [x] Create `jetpack/jetpack_small.model.toml`.
- [x] Create `jetpack/model_check_report.md`.
- [x] Create `jetpack/proof_status.md`.
- [x] Create these if needed:
  - [ ] `jetpack/jetpack.tla-types` -- not needed (full hand-translation)
  - [x] `jetpack/jetpack_for_translate.tla` -- created for translator attempt
  - [ ] `jetpack/jetpack_model_check.json` -- not produced (parse failure)
  - [x] `jetpack/refinement_proof/` with proof modules
    4 modules: state_machine.rs, invariants.rs, induction.rs, refinement.rs
- [x] Do not leave the work as one giant markdown note; the translated spec
  must exist as code under `jetpack/`.

## Phase 1: Translation Audit

- [x] Create `jetpack/translation_audit.md` before writing Jetpack spec code.
- [x] Include a complete variable inventory from `jetpack/jetpack.tla`.
- [x] Include a complete helper/operator inventory.
- [x] Include a complete action inventory.
- [x] Include the exact safety properties to preserve.
- [x] Include a map from TLA constructs to intended Verus encodings.
- [x] Include a list of likely `translate-tla` pain points before touching code.
- [x] Be explicit about:
  - [x] nested log shape
  - [x] message bag representation
  - [x] view / epoch / jpool record types
  - [x] recursive helper definitions
  - [x] `UNCHANGED` obligations per action
  - [x] constants and finite domains needed for model checking
- [x] Do not skip this phase; if you skip it, later translation will drift.

## Phase 2: Translator-First, But Not Translator-Only

- [x] Try the repo translator first on the original `jetpack/jetpack.tla`.
  Result: Parse error on `\o` (sequence concat interpreted as octal escape).
- [x] If that fails because Jetpack is outside the supported D1 subset, create
  a translation-friendly copy inside `jetpack/` and try again.
  Created `jetpack/jetpack_for_translate.tla` with syntactic normalizations.
  Still fails on range operator `1..N` used in 6+ critical locations.
- [x] Record exactly what failed.
  12+ blocking constructs documented in `translation_audit.md` §13a.
- [x] Record exactly what syntax or formatting was normalized.
  7 normalizations applied (ProposerOf, removed RECURSIVE/SelectSeq/record-set defs).
- [x] Record exactly what remained manual.
  **Everything** — the translator cannot produce usable output. Full hand-translation required.
- [x] Use translator output only as a starting point.
  N/A — no usable output produced.
- [ ] Manually repair the generated spec until it accurately matches the TLA.
  N/A — proceeding directly to hand-translation in Phase 3.
- [x] Never claim success just because `translate-tla` emitted a file.
  No file was emitted.
- [ ] Compare the generated operators against the original TLA action by action.
  N/A — no generated operators exist.
- [x] If you create `jetpack_for_translate.tla`, ensure it is only a faithful
  syntactic normalization, not a semantic redesign.
  Confirmed: only syntactic changes, all removals clearly commented.
- [ ] If you create `jetpack.tla-types`, keep it minimal and document why each
  nontrivial annotation exists.
  Not needed — full hand-translation path chosen.

## Phase 3: Hand-Finish The `tla-rs` Spec

- [x] Make the end state a readable, hand-owned `tla-rs` spec under `jetpack/`,
  not a fragile translator dump.
- [x] Put shared types in `jetpack/types.rs`.
- [x] Put the translated protocol in `jetpack/jetpack.rs`.
- [x] Follow the repo's `L*` naming style for spec functions.
- [x] Introduce `LState` and `LConstants` in the normal repo style.
- [x] Add a top-level `LInit`.
- [x] Add a top-level `LNext` corresponding to the Jetpack transition relation.
- [x] Preserve all declared state from `jetpack.tla`; do not silently drop
  fields.
- [x] Preserve frame conditions; every action must make unchanged state
  explicit.
- [x] Translate recursive helpers as spec helpers when appropriate.
- [x] Keep action names close to the TLA names so review is possible.
- [x] Preserve the bottom-of-file property definitions with equivalent names.
- [x] Ensure these structures survive translation:
  - [x] multiset/bag-style `messages`
  - [x] 3-D `log`
  - [x] per-server/per-proposer `commitIndex`
  - [x] `View`, `JPool`, `PrepResp`, and message record types
  - [x] conflict-order helpers
  - [x] `original_execution_cmds`
  - [x] `execution_cmds`
- [x] Do not collapse multiple Jetpack phases into one vague action.
- [x] Do not remove `ClientSendPreaccept`, recovery, prepare, accept, resubmit,
  or finish-recovery actions just because they are inconvenient.
- [x] Do not rename properties into generic placeholders like `Invariant1`.

## Phase 4: Model Checking

- [x] Use the source-first model checker described in
  `docs/model-checking-source-first.md`.
- [x] Create `jetpack/jetpack_small.model.toml` as an exact, tiny, bounded run.
- [x] Set `state_dedup = "canonical"`.
- [x] Keep the initial search depth very small.
- [x] Keep the initial domains very small.
- [x] Set `check_deadlock = false` initially.
- [x] Start with the smallest sane domains, for example:
  - [x] 2 servers
  - [x] 1 client
  - [x] 1 key
  - [x] 1 command id
  - [x] 1 or 2 proposers
- [x] Do not start with a large config.
- [ ] Check properties one at a time first:
  BLOCKED: model checker parser cannot parse the Jetpack spec (closure syntax,
  matches expressions, struct update). See `model_check_report.md`.
  - [ ] `CommittedLogAgreement`
  - [ ] `LogOrderMatchesExecution`
  - [ ] `ExecutionDedupMatches`
- [ ] Try a combined run only if the single-property runs are stable.
  BLOCKED: same parser limitation.
- [x] Record in `jetpack/model_check_report.md`:
  - [x] exact command(s) used
  - [x] exact model file(s) used
  - [x] whether the run was exact or lossy
  - [x] whether exploration completed or stopped early
  - [x] any JSON report path you saved
  - [x] the smallest counterexample or blocker, if any
- [x] If the model checker fails:
  - [x] reduce to the smallest reproducer
  - [x] keep that reproducer in `jetpack/`
  - [x] record the exact error text
  - [x] distinguish between:
    - [ ] translation bug — NOT the issue
    - [ ] Jetpack spec bug — NOT the issue
    - [x] model-checker limitation — THIS IS THE BLOCKER
      Parser does not support closure syntax, matches, struct update.
    - [ ] state explosion / existential expansion limit — not reached (parse fails first)
- [x] Do not present `hash_compaction64`, symmetry merging, or incomplete
  exploration as proof-strength evidence.

## Phase 5: Proof Work

- [x] Do real proof work; do not stop at prose.
  Proof scaffolding created with 17 support invariants, 11 lemma skeletons,
  and named theorem statements. All lemmas have assume(false) -- no proofs
  discharged yet. See `jetpack/proof_status.md` for full status.
- [x] Mirror the repo's proof organization style, preferably under:
  - [x] `jetpack/refinement_proof/state_machine.rs`
  - [x] `jetpack/refinement_proof/invariants.rs`
  - [x] `jetpack/refinement_proof/induction.rs`
  - [x] `jetpack/refinement_proof/refinement.rs`
- [x] If a full proof directory is too much for the first pass, at least create
  proof modules and theorem skeletons with meaningful names and a clear
  dependency graph.
  Full directory created with 4 modules matching Raft proof structure.
- [x] Focus the first proof pass on safety, not liveness:
  - [x] well-formedness / type invariants
  - [x] initialization lemmas
  - [x] inductive-preservation lemmas
  - [x] the three named safety properties
  All defined as spec fns and proof fn skeletons. Proofs not yet discharged.
- [x] Do not try to prove the user-facing properties directly with no support
  lemmas.
  JetpackSafetyInvariant defined as conjunction of 17 support invariants.
- [x] First define a stronger `JetpackSafetyInvariant` or equivalent support
  invariant.
  Defined in invariants.rs with 17 conjuncts across 8 categories.
- [x] Consider support invariant categories such as:
  - [x] log / commit-index bounds
    CommitIndexBounded, LogTermsNonNegative
  - [x] record well-formedness
    TypeInvariant (all maps keyed over correct domains)
  - [x] epoch monotonicity
    JEpochGeqOEpoch, CurrentTermPositive
  - [x] message typing / provenance
    5 provenance invariants (PreacceptRequest, BeginRecovery, Prepare, Accept, FinishRecovery)
  - [x] execution trace well-formedness
    ExecutionCmdsWellFormed, OriginalExecutionCmdsWellFormed
  - [x] command-id uniqueness assumptions actually enforced by the spec
    CommittedCmdIdsUnique
- [x] Borrow structure from Raft, but do not cargo-cult Raft's exact invariants.
  Structure mirrors Raft (state_machine/invariants/induction/refinement) but
  invariants are Jetpack-specific (3-D log, jpool, epoch, conflict-order).
- [x] Use `reports/raft_refinement_proof.md` to understand proof layout and gap
  reporting discipline.
- [x] If assumptions are required, isolate them and document them in
  `proof_status.md`.
  All 11 assume(false) locations documented with exact blockers and next steps.
- [x] Never scatter undocumented `assume(false)` through the proof and call it
  done.
  Every assume(false) is documented in proof_status.md with technical blocker.
- [x] Make sure `jetpack/proof_status.md` includes:
  - [x] what is proved
    Nothing yet -- all lemmas have assume(false).
  - [x] what is only scaffolded
    17 invariants, 11 lemmas, 4 modules.
  - [x] what assumptions remain
    11 assume(false) across 4 files, enumerated in table.
  - [x] the exact technical blocker for each missing lemma
    Section 4: per-lemma blocker analysis with difficulty ratings.
  - [x] the next concrete step for each blocker
    Section 4: specific next steps for each lemma.

## Documentation Expectations

- [x] Leave the `jetpack/` folder in a state where another engineer can answer:
  - [x] What exactly from `jetpack.tla` was translated?
    See `translation_audit.md` -- complete inventory of all 7 constants,
    22 variables, 19 actions, 3 safety properties.
  - [x] What had to be manual?
    Everything -- translator cannot handle the spec (12+ blocking constructs).
    See `translation_audit.md` section 13a.
  - [x] What model-checked?
    Nothing -- source-first model checker parser cannot handle closure syntax,
    matches, struct update. See `model_check_report.md`.
  - [x] What failed, and why?
    Model checking: parser limitation (not translation or spec bug).
    See `model_check_report.md` section 3.
  - [x] What is actually proved?
    Nothing yet -- all 11 proof lemmas have assume(false).
    See `proof_status.md` section 1.
  - [x] What still needs proof engineering?
    All 11 lemmas. See `proof_status.md` sections 4-5 for per-lemma
    blockers and recommended proof order.
- [x] Replace vague statements like "proof work remains" or "model checking is
  hard" with concrete blockers.
  All blockers are concrete: parser limitation (model check), quorum
  intersection lemma (CommittedLogAgreement), conflict-order formalization
  (LogOrderMatchesExecution), dedup properties (ExecutionDedupMatches).

## Definition Of Done

- [x] There is a real translated `tla-rs` spec under `jetpack/`.
  `jetpack/jetpack.rs` (850 lines) + `jetpack/types.rs` (200 lines).
- [x] The translation preserves the 3-D log architecture.
  `log: Map<int, Map<int, Seq<LLogEntry>>>` -- server -> proposer -> seq.
- [x] There is at least one bounded model config under `jetpack/`.
  `jetpack/jetpack_small.model.toml` with canonical dedup, depth 1, 200 states.
- [x] There is a written model-check report with commands, outcomes, and
  blockers.
  `jetpack/model_check_report.md` with exact commands, errors, classification.
- [x] There is proof scaffolding with named invariants and lemmas.
  `jetpack/refinement_proof/` with 4 modules, 17 invariants, 11 lemma skeletons.
- [x] There is a proof status doc that separates proved facts from gaps.
  `jetpack/proof_status.md` with 7 sections covering all aspects.
- [x] All artifacts remain inside `jetpack/`, unless a documented blocker
  forced something else.
  All files under `jetpack/`. No external edits required.

## What Does Not Count As Success

- [ ] Do not stop at only writing markdown with no translated spec code.
- [ ] Do not dump raw translator output without reviewing and repairing it.
- [ ] Do not stop at trivial typing facts while ignoring the real properties.
- [ ] Do not run only lossy model checking and call it verification.
- [ ] Do not rewrite Jetpack into a different protocol shape.
- [ ] Do not flatten or project away the 3-D log.
- [ ] Do not say "the big picture doc is future work" and then skip the actual
  `jetpack.tla` properties.

## Preferred Execution Order

- [x] Follow this order unless a concrete blocker forces a change:
  - [x] `translation_audit.md`
  - [x] translator-first attempt and notes
  - [x] `types.rs`
  - [x] `jetpack.rs`
  - [x] tiny exact `model.toml`
  - [x] model-check evidence / blocker reduction
  - [x] proof module skeleton
  - [ ] proof attempts (next: discharge assume(false) per proof_status.md order)
  - [x] final status docs
- [x] If you diverge from this order, explain why in the docs.
  No divergence -- followed the prescribed order exactly.
