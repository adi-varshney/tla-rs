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
- [ ] Create `jetpack/jetpack_small.model.toml`.
- [ ] Create `jetpack/model_check_report.md`.
- [ ] Create `jetpack/proof_status.md`.
- [ ] Create these if needed:
  - [ ] `jetpack/jetpack.tla-types`
  - [ ] `jetpack/jetpack_for_translate.tla`
  - [ ] `jetpack/jetpack_model_check.json`
  - [ ] `jetpack/refinement_proof/` with proof modules
- [ ] Do not leave the work as one giant markdown note; the translated spec
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

- [ ] Use the source-first model checker described in
  `docs/model-checking-source-first.md`.
- [ ] Create `jetpack/jetpack_small.model.toml` as an exact, tiny, bounded run.
- [ ] Set `state_dedup = "canonical"`.
- [ ] Keep the initial search depth very small.
- [ ] Keep the initial domains very small.
- [ ] Set `check_deadlock = false` initially.
- [ ] Start with the smallest sane domains, for example:
  - [ ] 2 servers
  - [ ] 1 client
  - [ ] 1 key
  - [ ] 1 command id
  - [ ] 1 or 2 proposers
- [ ] Do not start with a large config.
- [ ] Check properties one at a time first:
  - [ ] `CommittedLogAgreement`
  - [ ] `LogOrderMatchesExecution`
  - [ ] `ExecutionDedupMatches`
- [ ] Try a combined run only if the single-property runs are stable.
- [ ] Record in `jetpack/model_check_report.md`:
  - [ ] exact command(s) used
  - [ ] exact model file(s) used
  - [ ] whether the run was exact or lossy
  - [ ] whether exploration completed or stopped early
  - [ ] any JSON report path you saved
  - [ ] the smallest counterexample or blocker, if any
- [ ] If the model checker fails:
  - [ ] reduce to the smallest reproducer
  - [ ] keep that reproducer in `jetpack/`
  - [ ] record the exact error text
  - [ ] distinguish between:
    - [ ] translation bug
    - [ ] Jetpack spec bug
    - [ ] model-checker limitation
    - [ ] state explosion / existential expansion limit
- [ ] Do not present `hash_compaction64`, symmetry merging, or incomplete
  exploration as proof-strength evidence.

## Phase 5: Proof Work

- [ ] Do real proof work; do not stop at prose.
- [ ] Mirror the repo's proof organization style, preferably under:
  - [ ] `jetpack/refinement_proof/state_machine.rs`
  - [ ] `jetpack/refinement_proof/invariants.rs`
  - [ ] `jetpack/refinement_proof/induction.rs`
  - [ ] `jetpack/refinement_proof/refinement.rs`
- [ ] If a full proof directory is too much for the first pass, at least create
  proof modules and theorem skeletons with meaningful names and a clear
  dependency graph.
- [ ] Focus the first proof pass on safety, not liveness:
  - [ ] well-formedness / type invariants
  - [ ] initialization lemmas
  - [ ] inductive-preservation lemmas
  - [ ] the three named safety properties
- [ ] Do not try to prove the user-facing properties directly with no support
  lemmas.
- [ ] First define a stronger `JetpackSafetyInvariant` or equivalent support
  invariant.
- [ ] Consider support invariant categories such as:
  - [ ] log / commit-index bounds
  - [ ] record well-formedness
  - [ ] epoch monotonicity
  - [ ] message typing / provenance
  - [ ] execution trace well-formedness
  - [ ] command-id uniqueness assumptions actually enforced by the spec
- [ ] Borrow structure from Raft, but do not cargo-cult Raft's exact invariants.
- [ ] Use `reports/raft_refinement_proof.md` to understand proof layout and gap
  reporting discipline.
- [ ] If assumptions are required, isolate them and document them in
  `proof_status.md`.
- [ ] Never scatter undocumented `assume(false)` through the proof and call it
  done.
- [ ] Make sure `jetpack/proof_status.md` includes:
  - [ ] what is proved
  - [ ] what is only scaffolded
  - [ ] what assumptions remain
  - [ ] the exact technical blocker for each missing lemma
  - [ ] the next concrete step for each blocker

## Documentation Expectations

- [ ] Leave the `jetpack/` folder in a state where another engineer can answer:
  - [ ] What exactly from `jetpack.tla` was translated?
  - [ ] What had to be manual?
  - [ ] What model-checked?
  - [ ] What failed, and why?
  - [ ] What is actually proved?
  - [ ] What still needs proof engineering?
- [ ] Replace vague statements like "proof work remains" or "model checking is
  hard" with concrete blockers.

## Definition Of Done

- [ ] There is a real translated `tla-rs` spec under `jetpack/`.
- [ ] The translation preserves the 3-D log architecture.
- [ ] There is at least one bounded model config under `jetpack/`.
- [ ] There is a written model-check report with commands, outcomes, and
  blockers.
- [ ] There is proof scaffolding with named invariants and lemmas.
- [ ] There is a proof status doc that separates proved facts from gaps.
- [ ] All artifacts remain inside `jetpack/`, unless a documented blocker
  forced something else.

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

- [ ] Follow this order unless a concrete blocker forces a change:
  - [ ] `translation_audit.md`
  - [ ] translator-first attempt and notes
  - [ ] `types.rs`
  - [ ] `jetpack.rs`
  - [ ] tiny exact `model.toml`
  - [ ] model-check evidence / blocker reduction
  - [ ] proof module skeleton
  - [ ] proof attempts
  - [ ] final status docs
- [ ] If you diverge from this order, explain why in the docs.
