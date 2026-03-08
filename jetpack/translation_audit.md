# Jetpack Translation Audit

This document inventories all constructs in `jetpack/jetpack.tla` and maps them to
intended Verus/tla-rs encodings. It must be completed **before** writing any Jetpack
spec code.

Source of truth: `jetpack/jetpack.tla` (801 lines).

---

## 1. Constants

| TLA+ Constant | Type | Verus Encoding | Notes |
|---------------|------|----------------|-------|
| `Server` | Finite set of server IDs | `Set<int>` field in `LConstants` | Quantifier domain |
| `Client` | Finite set of client IDs | `Set<int>` field in `LConstants` | Quantifier domain |
| `CmdId` | Finite set of command IDs | `Set<int>` field in `LConstants` | Used to construct `Commands` |
| `Key` | Finite set of keys | `Set<int>` field in `LConstants` | Used in `JPool.pool` and conflict detection |
| `NoOpCmd` | Sentinel command value | Field in `LConstants` | Protocol-internal; filtered by `FilterNoOps` |
| `Proposer` | Set of proposer IDs | `Set<int>` field in `LConstants` | 3-D log second dimension |
| `ProposerOf(_)` | `Server -> Proposer` function | `Map<int, int>` field in `LConstants` or spec fn | Maps server to its active proposer |

## 2. Sentinel / Enum Constants

| TLA+ Name | Value | Verus Encoding |
|-----------|-------|----------------|
| `Nil` | `"Nil"` | Enum variant or `Option::None` |
| `NilCmd` | `[tag |-> "NilCmd"]` | Enum variant `LCmd::NilCmd` |
| `NilJPool` | `[tag |-> "NilJPool"]` | `Option<LJPool>::None` or enum variant |
| `NilPrepResp` | `[tag |-> "NilPrepResp"]` | `Option<LPrepResp>::None` or enum variant |
| `Ready` | `"Ready"` | Enum variant `LJState::Ready` |
| `Recovery` | `"Recovery"` | Enum variant `LJState::Recovery` |
| `AfterBeginRecovery` | `"AfterBeginRecovery"` | Enum variant `LJState::AfterBeginRecovery` |
| `AfterPrepare` | `"AfterPrepare"` | Enum variant `LJState::AfterPrepare` |
| `AfterAccept` | `"AfterAccept"` | Enum variant `LJState::AfterAccept` |
| `AfterResubmit` | `"AfterResubmit"` | Enum variant `LJState::AfterResubmit` |
| `ToBeLeader` | `"ToBeLeader"` | Enum variant `LOState::ToBeLeader` |
| `Follower` | `"Follower"` | Enum variant `LOState::Follower` |
| `Candidate` | `"Candidate"` | Enum variant `LOState::Candidate` |
| `Leader` | `"Leader"` | Enum variant `LOState::Leader` |

### Message Type Tags

| TLA+ Tag | Verus Encoding |
|----------|----------------|
| `PreacceptRequest` | `LMessage::PreacceptRequest { ... }` |
| `PreacceptResponse` | `LMessage::PreacceptResponse { ... }` |
| `BeginRecoveryRequest` | `LMessage::BeginRecoveryRequest { ... }` |
| `BeginRecoveryResponse` | `LMessage::BeginRecoveryResponse { ... }` |
| `JetpackPrepareRequest` | `LMessage::JetpackPrepareRequest { ... }` |
| `JetpackPrepareResponse` | `LMessage::JetpackPrepareResponse { ... }` |
| `JetpackAcceptRequest` | `LMessage::JetpackAcceptRequest { ... }` |
| `JetpackAcceptResponse` | `LMessage::JetpackAcceptResponse { ... }` |
| `FinishRecoveryRequest` | `LMessage::FinishRecoveryRequest { ... }` |

## 3. Variable Inventory

### 3.1 Base Protocol Variables (Jetpack reads/writes)

| TLA+ Variable | Shape | Verus Type | Notes |
|---------------|-------|------------|-------|
| `messages` | Multiset (message bag): `Message -> Nat` | `Map<LMessage, nat>` | Key = message record, value = multiplicity |
| `currentTerm` | `[Server -> Nat]` | `Map<int, nat>` | Per-server epoch/term |
| `ostate` | `[Server -> {Follower, Candidate, Leader, ToBeLeader}]` | `Map<int, LOState>` | Per-server role |
| `log` | `[Server -> [Proposer -> Seq(LogEntry)]]` | `Map<int, Map<int, Seq<LLogEntry>>>` | **3-D log**: `log[i][j][k]` |
| `commitIndex` | `[Server -> [Proposer -> Nat]]` | `Map<int, Map<int, nat>>` | Per-server, per-proposer commit progress |

### 3.2 Jetpack Per-Server Variables

| TLA+ Variable | Shape | Verus Type | Notes |
|---------------|-------|------------|-------|
| `jstate` | `[Server -> JState]` | `Map<int, LJState>` | Recovery phase state |
| `jepoch` | `[Server -> Nat]` | `Map<int, nat>` | Jetpack epoch |
| `oepoch` | `[Server -> Nat]` | `Map<int, nat>` | Observed epoch |
| `old_view` | `[Server -> View]` | `Map<int, LView>` | Previous view |
| `new_view` | `[Server -> View]` | `Map<int, LView>` | Current/target view |
| `jpool` | `[Server -> JPool]` | `Map<int, LJPool>` | Per-server Jetpack pool |
| `recovery_set` | `[Server -> SUBSET Commands]` | `Map<int, Set<LCmd>>` | Commands recovered |
| `chosen_value` | `[Server -> SUBSET Commands]` | `Map<int, Set<LCmd>>` | Chosen value for recovery |
| `br_responses` | `[Server -> [Server -> JPool ∪ {NilJPool}]]` | `Map<int, Map<int, Option<LJPool>>>` | BeginRecovery responses |
| `prep_responses` | `[Server -> [Server -> PrepResp ∪ {NilPrepResp}]]` | `Map<int, Map<int, Option<LPrepResp>>>` | Prepare responses |
| `accept_responses` | `[Server -> [Server -> BOOLEAN]]` | `Map<int, Map<int, bool>>` | Accept responses |

### 3.3 Client Variables

| TLA+ Variable | Shape | Verus Type | Notes |
|---------------|-------|------------|-------|
| `client_view` | `[Client -> View]` | `Map<int, LView>` | Client's current view |
| `client_pending` | `[Client -> Cmd ∪ {NilCmd}]` | `Map<int, Option<LCmd>>` | Pending command |
| `client_successes` | `[Client -> SUBSET Server]` | `Map<int, Set<int>>` | Successful responders |
| `client_heard_from` | `[Client -> SUBSET Server]` | `Map<int, Set<int>>` | All responders |

### 3.4 Execution Tracking Variables

| TLA+ Variable | Shape | Verus Type | Notes |
|---------------|-------|------------|-------|
| `original_execution_cmds` | `Seq(Cmd)` | `Seq<LCmd>` | Base protocol execution trace |
| `execution_cmds` | `Seq(Cmd)` | `Seq<LCmd>` | Combined (base + fast-path) trace |

### 3.5 Variable Groups

| TLA+ Group | Variables |
|------------|-----------|
| `baseVars` | `currentTerm`, `ostate`, `log`, `commitIndex` |
| `jetpackVars` | `jstate`, `jepoch`, `oepoch`, `old_view`, `new_view`, `jpool`, `recovery_set`, `chosen_value`, `br_responses`, `prep_responses`, `accept_responses` |
| `clientVars` | `client_view`, `client_pending`, `client_successes`, `client_heard_from` |
| `executionVars` | `original_execution_cmds`, `execution_cmds` |

## 4. Record / Data Types

| TLA+ Type | Fields | Verus Struct | Notes |
|-----------|--------|--------------|-------|
| `Commands` | `cmd_id: CmdId`, `key: Key` | `LCmd { cmd_id: int, key: int }` | Plus `NilCmd` variant |
| `View` | `epoch: Nat`, `proposing_replica_ids: SUBSET Server`, `replica_ids: SUBSET Server` | `LView { epoch: nat, proposing_replica_ids: Set<int>, replica_ids: Set<int> }` | |
| `JPool` | `max_seen_ballot: Nat`, `accepted_ballot: Nat`, `accepted_value: SUBSET Commands`, `pool: [Key -> Cmd ∪ {NilCmd}]` | `LJPool { max_seen_ballot: nat, accepted_ballot: nat, accepted_value: Set<LCmd>, pool: Map<int, Option<LCmd>> }` | `pool` maps key to command or nil |
| `PrepResp` | `accepted_ballot: Nat`, `accepted_value: SUBSET Commands` | `LPrepResp { accepted_ballot: nat, accepted_value: Set<LCmd> }` | |
| `LogEntry` | `term: Nat`, `value: Cmd` | `LLogEntry { term: nat, value: LCmd }` | Implicit from `log` usage |

## 5. Helper / Operator Inventory

### 5.1 Quorum Helpers

| TLA+ Operator | Signature | Verus Encoding | Complexity |
|---------------|-----------|----------------|------------|
| `Quorum` | `-> SUBSET(SUBSET Server)` | `spec fn LQuorum(c: LConstants) -> Set<Set<int>>` | Set comprehension with cardinality |
| `JQuorum(v)` | `View -> SUBSET(SUBSET Server)` | `spec fn LJQuorum(v: LView) -> Set<Set<int>>` | Like Quorum but over `v.replica_ids` |
| `FastpathQuorum(v)` | `View -> SUBSET(SUBSET Server)` | `spec fn LFastpathQuorum(v: LView) -> Set<Set<int>>` | Complex: JQuorum subset with intersection property |

### 5.2 Utility Helpers

| TLA+ Operator | Signature | Verus Encoding | Notes |
|---------------|-----------|----------------|-------|
| `Min(s)` | `Set<Nat> -> Nat` | `spec fn LMin(s: Set<nat>) -> nat` | CHOOSE-based |
| `Max(s)` | `Set<Nat> -> Nat` | `spec fn LMax(s: Set<nat>) -> nat` | CHOOSE-based |
| `SeqToSet(s)` | `Seq<T> -> Set<T>` | `spec fn LSeqToSet(s: Seq<T>) -> Set<T>` | Set comprehension |
| `Commands` | `-> Set<Cmd>` | Derived from constants | `{ [cmd_id |-> id, key |-> k] : id ∈ CmdId, k ∈ Key }` |
| `DefaultView` | `-> View` | Constant/spec fn | `[epoch |-> 1, ...]` |
| `EmptyJPool` | `-> JPool` | Constant/spec fn | Zero-initialized pool |

### 5.3 Message Bag Helpers

| TLA+ Operator | Signature | Verus Encoding | Notes |
|---------------|-----------|----------------|-------|
| `WithMessage(m, msgs)` | `Msg × Bag -> Bag` | `spec fn LWithMessage(m: LMessage, msgs: Map<LMessage, nat>) -> Map<LMessage, nat>` | Increment multiplicity |
| `WithoutMessage(m, msgs)` | `Msg × Bag -> Bag` | `spec fn LWithoutMessage(m: LMessage, msgs: Map<LMessage, nat>) -> Map<LMessage, nat>` | Decrement/remove |
| `Send(m)` | Action helper | `messages' = LWithMessage(m, messages)` | |
| `Discard(m)` | Action helper | `messages' = LWithoutMessage(m, messages)` | |
| `Reply(resp, req)` | Action helper | `messages' = LWithoutMessage(req, LWithMessage(resp, messages))` | |
| `AddMessages(ms, msgs)` | `RECURSIVE` | `spec fn LAddMessages(ms: Set<LMessage>, msgs: Map<LMessage, nat>) -> Map<LMessage, nat> decreases ms.len()` | **Recursive**; adds a set of messages to bag |

### 5.4 Sequence / Command Helpers

| TLA+ Operator | Signature | Verus Encoding | Notes |
|---------------|-----------|----------------|-------|
| `RemoveCmd(seq, cmd)` | `RECURSIVE` | `spec fn LRemoveCmd(seq: Seq<LCmd>, cmd: LCmd) -> Seq<LCmd> decreases seq.len()` | **Recursive**; remove all occurrences |
| `Dedup(seq)` | `RECURSIVE` | `spec fn LDedup(seq: Seq<LCmd>) -> Seq<LCmd> decreases seq.len()` | **Recursive**; calls RemoveCmd |
| `FilterNoOps(seq)` | `Seq -> Seq` | `spec fn LFilterNoOps(seq: Seq<LCmd>, no_op: LCmd) -> Seq<LCmd>` | Uses `SelectSeq` / filter |
| `IndexOf(s, e)` | `RECURSIVE` | `spec fn LIndexOf(s: Seq<LCmd>, e: LCmd) -> int decreases s.len()` | **Recursive**; 0 = not found, 1-based index |
| `CmdConflicts(a, b)` | `Cmd × Cmd -> Bool` | `spec fn LCmdConflicts(a: LCmd, b: LCmd) -> bool` | Same key, different cmd |
| `ConflictOrderPreserved(s1, s2)` | `Seq × Seq -> Bool` | `spec fn LConflictOrderPreserved(s1: Seq<LCmd>, s2: Seq<LCmd>) -> bool` | Universal quantifier over indices |

### 5.5 Log / Commit Helpers

| TLA+ Operator | Signature | Verus Encoding | Notes |
|---------------|-----------|----------------|-------|
| `LogCmdIds` | `-> Set<CmdId>` | `spec fn LLogCmdIds(s: LState) -> Set<int>` | UNION across all servers × proposers |
| `ExecCmdIds` | `-> Set<CmdId>` | `spec fn LExecCmdIds(s: LState) -> Set<int>` | From `execution_cmds` |
| `OriginalExecCmdIds` | `-> Set<CmdId>` | `spec fn LOriginalExecCmdIds(s: LState) -> Set<int>` | From `original_execution_cmds` |
| `UsedCmdIds` | `-> Set<CmdId>` | Union of above three | |
| `AvailableCommands` | `-> Set<Cmd>` | `spec fn LAvailableCommands(s: LState, c: LConstants) -> Set<LCmd>` | Commands with unused cmd_id |
| `HasConflict(pool, cmd)` | `-> Bool` | `spec fn LHasConflict(pool: Map<int, Option<LCmd>>, cmd: LCmd) -> bool` | |
| `RecoveryCommands(i, qs)` | `-> Set<Cmd>` | `spec fn LRecoveryCommands(...)` | Majority-reported commands |
| `CommittedCmds(i, j)` | `-> Seq<Cmd>` | `spec fn LCommittedCmds(s: LState, i: int, j: int) -> Seq<LCmd>` | |
| `AllCommittedCmds(i)` | `-> Set<Cmd>` | `spec fn LAllCommittedCmds(s: LState, i: int) -> Set<LCmd>` | Union across proposers |
| `ChosenExecutedInView(i)` | `-> Bool` | `spec fn LChosenExecutedInView(s: LState, i: int) -> bool` | |
| `JPoolCommands(p)` | `-> Set<Cmd>` | `spec fn LJPoolCommands(p: LJPool) -> Set<LCmd>` | Non-nil pool values |

## 6. Action Inventory

### 6.1 Initialization

| TLA+ Action | Verus Function | Variables Modified |
|-------------|----------------|-------------------|
| `InitJetpackVars` | `spec fn LInitJetpackVars(s: LState, c: LConstants) -> bool` | All jetpackVars |
| `InitClientVars` | `spec fn LInitClientVars(s: LState, c: LConstants) -> bool` | All clientVars |
| `InitExecutionVars` | `spec fn LInitExecutionVars(s: LState) -> bool` | executionVars |

### 6.2 Client Actions

| TLA+ Action | Parameters | Changed | Unchanged |
|-------------|------------|---------|-----------|
| `ClientSendPreaccept(c)` | `c ∈ Client` | `messages`, `client_pending`, `client_successes`, `client_heard_from` | `baseVars`, `jetpackVars`, `client_view`, `executionVars` |
| `HandlePreacceptResponse(c, m)` | `c ∈ Client`, `m ∈ messages` | `client_*`, `execution_cmds`, `original_execution_cmds`, `messages` | `baseVars`, `jetpackVars` |

### 6.3 Server Preaccept

| TLA+ Action | Parameters | Changed | Unchanged |
|-------------|------------|---------|-----------|
| `HandlePreacceptRequest(i, m)` | `i ∈ Server`, `m ∈ messages` | `jpool`, `log`, `messages` | `currentTerm`, `ostate`, `commitIndex`, most jetpackVars, clientVars, executionVars |

### 6.4 Recovery Phase 1: BeginRecovery

| TLA+ Action | Parameters | Changed | Unchanged |
|-------------|------------|---------|-----------|
| `SendBeginRecovery(i)` | `i ∈ Server` | `messages`, `jstate`, `br_responses` | `baseVars`, other jetpackVars, clientVars, executionVars |
| `HandleBeginRecoveryRequest(i, m)` | `i ∈ Server`, `m` | `old_view`, `new_view`, `oepoch`, `jstate`, `messages` | `baseVars`, other jetpackVars, clientVars, executionVars |
| `HandleBeginRecoveryResponse(i, m)` | `i ∈ Server`, `m` | `br_responses`, `messages` | `baseVars`, other jetpackVars, clientVars, executionVars |
| `CompleteBeginRecovery(i)` | `i ∈ Server` | `recovery_set`, `chosen_value`, `jstate` | `messages`, `baseVars`, other jetpackVars, clientVars, executionVars |

### 6.5 Recovery Phase 2: Prepare

| TLA+ Action | Parameters | Changed | Unchanged |
|-------------|------------|---------|-----------|
| `SendPrepare(i)` | `i ∈ Server` | `messages`, `prep_responses` | `baseVars`, other jetpackVars, clientVars, executionVars |
| `HandlePrepareRequest(i, m)` | `i ∈ Server`, `m` | `oepoch`, `jepoch`, `jpool`, `messages` | `baseVars`, `jstate`, `old_view`, `new_view`, others |
| `HandlePrepareResponse(i, m)` | `i ∈ Server`, `m` | `prep_responses` (or `oepoch`, `jepoch`, `jpool` on reject) | `baseVars`, `jstate`, `old_view`, `new_view`, others |
| `CompletePrepare(i)` | `i ∈ Server` | `chosen_value`, `jstate` | `messages`, `baseVars`, other jetpackVars, clientVars, executionVars |

### 6.6 Recovery Phase 3: Accept

| TLA+ Action | Parameters | Changed | Unchanged |
|-------------|------------|---------|-----------|
| `SendAccept(i)` | `i ∈ Server` | `messages`, `accept_responses` | `baseVars`, other jetpackVars, clientVars, executionVars |
| `HandleAcceptRequest(i, m)` | `i ∈ Server`, `m` | `oepoch`, `jepoch`, `jpool`, `messages` | `baseVars`, `jstate`, `old_view`, `new_view`, others |
| `HandleAcceptResponse(i, m)` | `i ∈ Server`, `m` | `accept_responses` (or `jpool` on reject) | `baseVars`, `jstate`, `old_view`, `new_view`, others |
| `CompleteAccept(i)` | `i ∈ Server` | `jstate` | `messages`, `baseVars`, other jetpackVars, clientVars, executionVars |

### 6.7 Resubmit & Finish Recovery

| TLA+ Action | Parameters | Changed | Unchanged |
|-------------|------------|---------|-----------|
| `Resubmit(i)` | `i ∈ Server` | `messages` | `baseVars`, all jetpackVars, clientVars, executionVars |
| `CompleteResubmit(i)` | `i ∈ Server` | `jstate` | `messages`, `baseVars`, other jetpackVars, clientVars, executionVars |
| `FinishRecovery(i)` | `i ∈ Server` | `messages`, `jepoch`, `oepoch`, `old_view`, `new_view`, `jpool`, `jstate`, `recovery_set`, `chosen_value`, `br_responses`, `prep_responses`, `accept_responses`, `ostate` | `currentTerm`, `log`, `commitIndex`, clientVars, executionVars |
| `HandleFinishRecovery(i, m)` | `i ∈ Server`, `m` | `jepoch`, `oepoch`, `old_view`, `new_view`, `jpool`, `jstate`, `recovery_set`, `chosen_value`, `ostate`, `messages` | `currentTerm`, `log`, `commitIndex`, `br_responses`, `prep_responses`, `accept_responses`, clientVars, executionVars |

### 6.8 Top-Level Next

```
JetpackNext ==
    \/ \E c \in Client : ClientSendPreaccept(c)
    \/ \E i \in Server : SendBeginRecovery(i)
    \/ \E i \in Server : CompleteBeginRecovery(i)
    \/ \E i \in Server : SendPrepare(i)
    \/ \E i \in Server : CompletePrepare(i)
    \/ \E i \in Server : SendAccept(i)
    \/ \E i \in Server : CompleteAccept(i)
    \/ \E i \in Server : Resubmit(i)
    \/ \E i \in Server : CompleteResubmit(i)
    \/ \E i \in Server : FinishRecovery(i)
```

**Note**: Message-receive actions (`HandlePreacceptRequest`, `HandleBeginRecoveryRequest`,
etc.) are triggered by `\E m \in DOMAIN messages` inside each handler. The top-level
`JetpackNext` does **not** list these separately because they share existential
quantification over messages with their parent actions. In the Verus spec, the
message-handling actions should be folded into `LNext` with explicit `exists |m|`
binders.

**Missing from JetpackNext** (provided by wrapper):
- `HandlePreacceptRequest(i, m)` — triggered by message receipt, implicitly part of server step
- `HandlePreacceptResponse(c, m)` — client processes preaccept response
- `HandleBeginRecoveryRequest/Response` — server processes recovery messages
- `HandlePrepareRequest/Response` — server processes prepare messages
- `HandleAcceptRequest/Response` — server processes accept messages
- `HandleFinishRecovery(i, m)` — server processes finish recovery
- `BecomeToBeLeader(i)` — wrapper-provided
- `ApplyCommitted(i)` — wrapper-provided

For the standalone Jetpack spec, all message handlers should be included in `LNext`.

## 7. Safety Properties

| TLA+ Property | Verus Spec Function | Description |
|---------------|---------------------|-------------|
| `CommittedLogAgreement` | `spec fn LCommittedLogAgreement(s: LState, c: LConstants) -> bool` | For each proposer, committed prefixes agree across servers. Quantifies over `Server × Server × Proposer × position`. |
| `LogOrderMatchesExecution` | `spec fn LLogOrderMatchesExecution(s: LState, c: LConstants) -> bool` | Per-server, per-proposer committed sequence preserves conflict order in execution trace. |
| `ExecutionDedupMatches` | `spec fn LExecutionDedupMatches(s: LState, c: LConstants) -> bool` | Bidirectional conflict-order preservation between `Dedup(FilterNoOps(original_execution_cmds))` and `Dedup(FilterNoOps(execution_cmds))`. |

## 8. Nested Log Shape (Critical)

The log is genuinely 3-dimensional:

```
log[i][j][k]
  i: server (where copy is stored)
  j: proposer (which logical sequence)
  k: position within that sequence
```

Each entry is `[term |-> Nat, value |-> Cmd]`.

`commitIndex[i][j]` tracks per-server, per-proposer commit progress.

**Verus encoding**:
```rust
// In LState:
pub log: Map<int, Map<int, Seq<LLogEntry>>>,
pub commit_index: Map<int, Map<int, nat>>,
```

This 3-D structure must NOT be flattened. The TODO explicitly forbids 2-D projections.

## 9. Message Bag Representation

Messages use a **multiset** (bag) encoding:

```
messages : Message -> Nat
```

- `DOMAIN messages` = set of message records with nonzero multiplicity
- `WithMessage(m, msgs)` = increment count (or add with count 1)
- `WithoutMessage(m, msgs)` = decrement count (remove if reaches 0)

**Verus encoding**: `Map<LMessage, nat>` where:
- `m ∈ DOMAIN messages` → `messages.contains_key(m) && messages[m] > 0`
- The `LMessage` enum must implement structural equality for map keying.

This is distinct from a simple `Set<LMessage>` — duplicate messages are possible.

## 10. Recursive Helper Definitions

Four `RECURSIVE` operators need special handling in Verus:

1. **`AddMessages(ms, msgs)`** — Set iteration: picks arbitrary element, adds to bag, recurses on remainder. Decreases: `ms.len()` (requires `ms.finite()`).

2. **`RemoveCmd(seq, cmd)`** — Filters a sequence. Decreases: `seq.len()`.

3. **`Dedup(seq)`** — Deduplicates by removing later occurrences. Calls `RemoveCmd` internally. Decreases: `seq.len()`.

4. **`IndexOf(s, e)`** — Finds 1-based position. Returns 0 if absent. Decreases: `s.len()`.

Verus requires explicit `decreases` clauses. All four have natural structural recursion on sequence/set length.

## 11. UNCHANGED Obligations Per Action

Every action must specify the post-state for ALL variables. The TLA+ spec uses
`UNCHANGED <<group>>` shorthand. In Verus, each action's spec function must explicitly
constrain all `LState` fields, either via update or `s_.field == s.field`.

The variable groups (§3.5) help organize this: each action changes a subset of groups
and leaves others unchanged. The Verus encoding should use a combined `LState` struct
containing all variables, and each action constrains all fields.

## 12. Constants and Finite Domains for Model Checking

### Minimum Viable Config

| Constant | Suggested Small Domain | Rationale |
|----------|----------------------|-----------|
| `Server` | `{0, 1}` (2 servers) | Minimum for quorum semantics |
| `Client` | `{0}` (1 client) | Minimum for preaccept flow |
| `CmdId` | `{0}` (1 command ID) | Minimum for command tracking |
| `Key` | `{0}` (1 key) | Minimum for conflict detection |
| `Proposer` | `{0}` (1 proposer) | Minimum for 3-D log |
| `ProposerOf` | `{0 |-> 0, 1 |-> 0}` | All servers map to sole proposer |

### Model Checker Concerns

- **Message bag**: The multiset representation creates large state spaces. Each message
  variant × field combination is a distinct map key. Consider bounding max message count.
- **Set-valued fields**: `recovery_set`, `chosen_value`, `accepted_value` are command sets.
  With small `CmdId × Key` this stays bounded.
- **Quorum computation**: `Quorum`, `JQuorum`, `FastpathQuorum` use `SUBSET` which is
  exponential. With 2 servers, `SUBSET({0,1}) = {{}, {0}, {1}, {0,1}}` — manageable.
- **`AvailableCommands`**: Depends on global state scan (`LogCmdIds ∪ ExecCmdIds ∪ OriginalExecCmdIds`).
  May cause existential expansion issues in source-first model checker.
- **Recursive helpers**: The source-first model checker has bounded recursion depth.
  With small domains this should be fine.

## 13. Likely `translate-tla` Pain Points

Based on the transpiler limitations documented in `docs/tla-to-verus-guide.md` and
`docs/tla-transpiler-limitations.md`:

1. **`RECURSIVE` definitions** — The transpiler has limited support for recursive functions.
   All four recursive helpers will likely need manual translation.

2. **`INSTANCE` / module parameterization** — `jetpack.tla` is designed to be `INSTANCE`'d
   by a wrapper. The transpiler does not support `INSTANCE`. For standalone translation,
   this is not a direct blocker since we translate jetpack.tla itself, but the parameterized
   constants (`ProposerOf(_)`) need manual handling.

3. **Multi-line conjunction/disjunction lists** — The TLA+ spec uses vertical bullet
   format (`/\ ... /\ ...`). The transpiler reportedly supports this, but complex nested
   structures may cause issues.

4. **Complex `EXCEPT` syntax** — Nested EXCEPT like
   `[jpool EXCEPT ![i].pool[cmd.key] = cmd]` and multi-path updates like
   `[jpool EXCEPT ![i].max_seen_ballot = ..., ![i].accepted_ballot = ..., ![i].accepted_value = ...]`
   are likely unsupported.

5. **`CHOOSE` operator** — Used in `Min`, `Max`, and `CompletePrepare`. Limited transpiler
   support.

6. **Set comprehensions with record construction** — `Commands == { [cmd_id |-> id, key |-> k] : id \in CmdId, k \in Key }` may not translate cleanly.

7. **`Cardinality` from `FiniteSets`** — Used in quorum definitions. May need manual
   encoding.

8. **`SelectSeq`** — Used in `FilterNoOps`. May need manual spec function.

9. **`@@` (function merge) and `:>` (singleton function)** — Used in `WithMessage`.
   Not in standard Verus; needs `Map::insert`.

10. **Record set type notation** — `View == [epoch: Nat, ...]` defines a record type
    set. The transpiler may not handle this.

**Recommendation**: Given the density of unsupported constructs, the translator output
will be a rough starting point at best. Plan for extensive manual repair. The translation
should proceed as: (1) try translator, (2) record failures, (3) hand-write the spec
using translator output where usable and manual translation elsewhere.

## 14. Intended Verus State/Constants Structure

```rust
verus! {

pub enum LJState {
    Ready,
    Recovery,
    AfterBeginRecovery,
    AfterPrepare,
    AfterAccept,
    AfterResubmit,
}

pub enum LOState {
    Follower,
    Candidate,
    Leader,
    ToBeLeader,
}

pub struct LCmd {
    pub cmd_id: int,
    pub key: int,
}

pub struct LLogEntry {
    pub term: nat,
    pub value: LCmd,
}

pub struct LView {
    pub epoch: nat,
    pub proposing_replica_ids: Set<int>,
    pub replica_ids: Set<int>,
}

pub struct LJPool {
    pub max_seen_ballot: nat,
    pub accepted_ballot: nat,
    pub accepted_value: Set<LCmd>,
    pub pool: Map<int, Option<LCmd>>,
}

pub struct LPrepResp {
    pub accepted_ballot: nat,
    pub accepted_value: Set<LCmd>,
}

// 9 message types as enum variants
pub enum LMessage {
    PreacceptRequest { msource: int, mdest: int, mepoch: nat, mview: LView, mcmd: LCmd },
    PreacceptResponse { msuccess: bool, mjepoch: nat, mview: LView, mcmd: LCmd, msource: int, mdest: int },
    BeginRecoveryRequest { msource: int, mdest: int, mold_view: LView, mnew_view: LView },
    BeginRecoveryResponse { mjpool: LJPool, msource: int, mdest: int },
    JetpackPrepareRequest { moepoch: nat, mjepoch: nat, mmax_seen_ballot: nat, msource: int, mdest: int },
    JetpackPrepareResponse { mok: bool, maccepted_ballot: nat, maccepted_value: Set<LCmd>, moepoch: nat, mjepoch: nat, mmax_seen_ballot: nat, msource: int, mdest: int },
    JetpackAcceptRequest { moepoch: nat, mjepoch: nat, mmax_seen_ballot: nat, mvalue: Set<LCmd>, msource: int, mdest: int },
    JetpackAcceptResponse { mok: bool, mmax_seen_ballot: nat, msource: int, mdest: int },
    FinishRecoveryRequest { moepoch: nat, mnew_view: LView, msource: int, mdest: int },
}

pub struct LConstants {
    pub server: Set<int>,
    pub client: Set<int>,
    pub cmd_id: Set<int>,
    pub key: Set<int>,
    pub no_op_cmd: LCmd,
    pub proposer: Set<int>,
    pub proposer_of: Map<int, int>,
}

pub struct LState {
    // Message bag
    pub messages: Map<LMessage, nat>,

    // Base protocol variables
    pub current_term: Map<int, nat>,
    pub ostate: Map<int, LOState>,
    pub log: Map<int, Map<int, Seq<LLogEntry>>>,
    pub commit_index: Map<int, Map<int, nat>>,

    // Jetpack per-server
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

    // Client
    pub client_view: Map<int, LView>,
    pub client_pending: Map<int, Option<LCmd>>,
    pub client_successes: Map<int, Set<int>>,
    pub client_heard_from: Map<int, Set<int>>,

    // Execution
    pub original_execution_cmds: Seq<LCmd>,
    pub execution_cmds: Seq<LCmd>,
}

} // verus!
```

## 15. Translation Strategy

1. **Phase 2**: Try `translate-tla` on `jetpack.tla`. Record all failures.
2. **Phase 3**: Hand-write `types.rs` with the structures above. Hand-write `jetpack.rs`
   with all spec functions, using translator output where applicable.
3. For each action, translate as `spec fn LActionName(s: LState, s_: LState, c: LConstants, ...) -> bool`
   with explicit pre/post state and unchanged constraints.
4. Recursive helpers: write as `spec fn` with `decreases` clauses.
5. Message bag: use `Map<LMessage, nat>` throughout.
6. Properties: translate directly as `spec fn` predicates over `LState` and `LConstants`.
