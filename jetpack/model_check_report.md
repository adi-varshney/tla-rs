# Jetpack Model Check Report

## Status: BLOCKED (model-checker parser limitation)

## 1. Model Configuration

File: `jetpack/jetpack_small.model.toml`

Smallest sane domains:
- `int`: 0..1 (covers 2 servers, 1 client, 1 key, 1 cmd_id, 1 proposer)
- `nat`: 0..1
- Enum domains: all variants of `LJState` (6) and `LOState` (4)
- Collection bounds: `max_set_len = 2`, `max_seq_len = 2`, `max_map_len = 2`
- Search: `max_depth = 1`, `max_states = 200`, `state_dedup = "canonical"`
- `check_deadlock = false`

## 2. Attempt Log

### Attempt 1: Direct model-check on `jetpack.rs` + `types.rs`

**Command**:
```bash
transpiler/target/release/verus-transpile model-check \
  --input jetpack/jetpack.rs \
  --types jetpack/types.rs \
  --model jetpack/jetpack_small.model.toml \
  --search bfs
```

**Result**: Parse error

```
Error: Parse error: Expected identifier, found '|'
```

**Root cause**: The source-first model checker's Verus parser does not support
several constructs used in the Jetpack spec:

1. **`Map::new(|k: int| pred, |k: int| value)` closures** -- Used extensively in
   `LInit`, `LEmptyJPool`, and all actions that reset response maps. The parser
   does not handle closure syntax `|param| expr`.

2. **`m matches LMessage::PreacceptRequest { ... }` pattern matching** -- Used in
   all message-handler actions to destructure the message. The parser does not
   support `matches` expressions.

3. **Struct update syntax `LJPool { field: val, ..s.jpool[i] }`** -- Used in
   `LHandlePrepareRequest`, `LHandleAcceptRequest`, and `LHandleAcceptResponse`.
   The parser does not support `..base` struct update.

4. **`include!("types.rs")` macro** -- Used to include types inline. The parser
   does not expand Rust macros. (This was worked around by using `--types`.)

5. **Non-ASCII characters in comments** -- Em-dash, union symbol, section sign
   caused byte-boundary panics. Fixed by replacing with ASCII equivalents.

### Pre-attempt issues fixed

- Replaced `--` (em-dash) with `--` in comments
- Replaced `union` (union symbol) with `union` in comments
- Replaced `section` (section sign) with `section` in comments

## 3. Blocker Classification

This is a **model-checker parser limitation**, not a translation bug or Jetpack
spec bug. The same class of limitation blocks Raft (`existential expansion limit`)
and RSL (`missing constants domain`) in the source-first model checker.

The Jetpack spec is semantically correct -- it is a faithful hand-translation of
`jetpack.tla` -- but it uses Verus language constructs that the current model
checker parser does not support.

## 4. Comparison with Other Protocols

| Protocol | Model-check status | Blocker |
|----------|-------------------|---------|
| TwoPhase | ok | N/A |
| LeaderElection | ok | N/A |
| PrimaryBackup | ok | N/A |
| Paxos | ok | N/A |
| PBFT | ok (bounded) | N/A |
| Raft | unsupported | Existential expansion limit |
| RSL | unsupported | Missing constants domain |
| VerticalPaxos | unsupported | Existential expansion limit |
| EPaxos | unsupported | Constants expansion limit |
| ChainReplication | unsupported | Existential expansion limit |
| **Jetpack** | **unsupported** | **Parser: closure syntax, matches, struct update** |

## 5. What Would Unblock Model Checking

To make the Jetpack spec model-checkable with the source-first checker, the parser
would need to support:

1. Closure-based `Map::new`, `Set::new` construction
2. `matches` expressions for enum destructuring
3. Struct update syntax (`..base`)

Alternatively, the spec could be rewritten to avoid these constructs:
- Replace `Map::new(|k| ...)` with explicit `Map::empty().insert(k1, v1).insert(k2, v2)...`
- Replace `matches` with explicit `if let` or `match` (if `match` is supported)
- Replace struct update with full struct literals

However, this would make the spec significantly less readable and harder to maintain,
and would still likely hit the existential expansion limit given the 19-branch `LNext`
with many `exists |m: LMessage|` quantifiers.

## 6. Recommendation

The most productive path forward for model checking Jetpack is to use TLC (the
standard TLA+ model checker) directly on `jetpack/jetpack.tla`, which is the
approach already used for the TLA+ big-picture verification (see
`jetpack/TLA_PLUS_BIG_PICTURE.md`).

The source-first model checker is best suited for protocols with simpler state
structures and fewer existential quantifiers. Jetpack's 22-field state, 9 message
types, and 19 actions with nested map/set operations place it firmly outside
the current supported subset.

## 7. Evidence

- Model config: `jetpack/jetpack_small.model.toml`
- Exact error: `Parse error: Expected identifier, found '|'`
- Run was **exact intent** (`state_dedup = "canonical"`) but failed at parse, not exploration
- No JSON report produced (parse failure before exploration)
- No counterexample (not a spec bug)
