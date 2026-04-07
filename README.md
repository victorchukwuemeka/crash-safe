# Crash-Safe KV Lab (Proof of Work)

We’re going to build a tiny replicated key‑value store and *explicitly* document every unit of work, how each unit interacts with others, and why the system behaves the way it does under faults.

This README is the detailed breakdown. It is the “source of truth” for the design and for what to implement.

## 1) System Goal (Single Sentence)
A 3‑node (1 leader, 2 followers) key‑value store that remains correct under one node failure and prevents duplicate writes during client retries.

## 2) Top‑Level Units (Macro Components)
Each unit below is a *separable building block* with a clear contract.

1. **Client API**
   - Sends `Put(key, value, request_id)` to leader.
   - Sends `Get(key)` to leader.
   - Retries `Put` with the same `request_id` when no response.

2. **Leader Node**
   - Accepts writes from clients.
   - Deduplicates `request_id`.
   - Appends to local log.
   - Replicates to followers.
   - Commits after majority (2/3) acks.
   - Replies to client with stable result.

3. **Follower Nodes**
   - Receive append entries from leader.
   - Store log entries and apply committed ones.
   - Acknowledge leader.

4. **Replication Log**
   - Ordered list of entries `(index, key, value, request_id)`.
   - Leader appends; followers mirror.

5. **Commit & Apply Engine**
   - Maintains `commit_index` (highest safely committed entry).
   - Applies committed entries to KV map.

6. **Deduplication Store**
   - Maps `request_id -> {index, committed, result}`.
   - Ensures retries do not duplicate writes.

7. **Fault Injection Layer**
   - Simulates crash, packet loss, and slow nodes.
   - Applies to RPC calls between nodes.

8. **Test Harness**
   - Executes scenarios: crash, packet loss, slow node.
   - Validates required guarantees.

9. **Postmortem Report**
   - 1–2 pages.
   - Explains failure modes, tradeoffs, and fixes.

## 3) Unit Breakdown (Micro Components)
Below we break each macro unit into smaller units. Each micro unit has **inputs**, **outputs**, and **dependencies**.

### 3.1 Client API
**Units**
1. `PutRequest`
   - Input: `key`, `value`, `request_id`.
   - Output: RPC call to leader.
   - Depends on: `Leader RPC Interface`.

2. `PutRetryPolicy`
   - Input: timeout/no response.
   - Output: re‑send same `request_id`.
   - Depends on: `Deduplication Store` in leader.

3. `GetRequest`
   - Input: `key`.
   - Output: RPC call to leader.
   - Depends on: `Leader KV Read`.

### 3.2 Leader Node
**Units**
1. `Leader RPC Handler`
   - Input: client `Put` or `Get` request.
   - Output: dispatch to correct internal unit.
   - Depends on: `Deduplication Store`, `Replication Log`, `Commit Engine`.

2. `Dedup Check`
   - Input: `request_id`.
   - Output: `new` / `pending` / `committed`.
   - Depends on: `Deduplication Store`.

3. `Append Local Log`
   - Input: `key`, `value`, `request_id`.
   - Output: new log entry with index.
   - Depends on: `Replication Log`.

4. `Replicate To Followers`
   - Input: log entry and commit index.
   - Output: RPCs to followers.
   - Depends on: `Fault Injection Layer`, `Follower Append Handler`.

5. `Majority Ack Gate`
   - Input: follower responses.
   - Output: commit decision.
   - Depends on: `Replication To Followers`.

6. `Commit Local Entry`
   - Input: commit decision.
   - Output: update commit index + apply entry.
   - Depends on: `Commit & Apply Engine`.

7. `Reply To Client`
   - Input: commit status or previous result.
   - Output: response to client.
   - Depends on: `Deduplication Store`.

### 3.3 Follower Node
**Units**
1. `Follower RPC Handler`
   - Input: `append_entries` or `apply_commit`.
   - Output: ack or error.
   - Depends on: `Replication Log` and `Commit Engine`.

2. `Append Entry`
   - Input: log entry from leader.
   - Output: stored entry or conflict error.
   - Depends on: `Replication Log`.

3. `Apply Commit`
   - Input: leader commit index.
   - Output: apply entries to KV map.
   - Depends on: `Commit & Apply Engine`.

### 3.4 Replication Log
**Units**
1. `LogEntry`
   - Fields: `index`, `key`, `value`, `request_id`.

2. `AppendEntry`
   - Input: `LogEntry`.
   - Output: stored in array/list.
   - Depends on: ordered integrity.

3. `Conflict Check`
   - Input: incoming entry index.
   - Output: accept or reject if different request_id.

### 3.5 Commit & Apply Engine
**Units**
1. `CommitIndex`
   - Input: majority ack decision.
   - Output: updated commit index.

2. `LastApplied`
   - Input: commit index.
   - Output: applies entries to KV map.

3. `KV Apply`
   - Input: log entry.
   - Output: update `kv[key] = value`.

### 3.6 Deduplication Store
**Units**
1. `DedupRecord`
   - Fields: `index`, `committed`, `result`.

2. `Lookup`
   - Input: `request_id`.
   - Output: record or `not found`.

3. `Insert`
   - Input: `request_id` + record.
   - Output: stored record.

4. `MarkCommitted`
   - Input: `request_id`.
   - Output: record updated.

### 3.7 Fault Injection Layer
**Units**
1. `PacketLoss`
   - Input: RPC call.
   - Output: drop or deliver.

2. `Delay`
   - Input: RPC call.
   - Output: slow delivery.

3. `CrashToggle`
   - Input: node id.
   - Output: node stops responding.

### 3.8 Test Harness
**Units**
1. `Crash Test`
   - Input: normal writes, then crash follower.
   - Output: validate commit and data.

2. `Packet Loss Test`
   - Input: drop rate set on follower RPC.
   - Output: ensure retries + commit.

3. `Slow Node Test`
   - Input: delay follower RPC.
   - Output: leader waits for majority only.

### 3.9 Postmortem Report
**Units**
1. `Failure Mode Analysis`
   - Describe crash, packet loss, slow node.

2. `Fixes & Guarantees`
   - Explain how dedup and majority commit prevent errors.

3. `Tradeoff Discussion`
   - Explain why reads go to leader: stronger consistency, lower availability.

## 4) Interaction Map (Who Talks to Whom)
1. Client → Leader: `Put` / `Get`.
2. Leader → Followers: `append_entries`, `apply_commit`.
3. Followers → Leader: ack or error.
4. Fault Injection Layer wraps all RPC calls.

## 5) Core Guarantees (Mapped to Units)
1. **Survive one node crash**
   - Depends on: `Majority Ack Gate`, `CommitIndex`.

2. **Retries do not duplicate writes**
   - Depends on: `Deduplication Store` + `Dedup Check`.

3. **Consistent reads (no stale data)**
   - Depends on: `Read path = leader only`.

## 6) Consistency vs Availability Tradeoff (Explicit)
We choose **consistency** over availability.
- Reads and writes go to leader only.
- If leader is down, system refuses operations.
- This avoids stale reads and split‑brain errors but reduces availability.

## 7) Implementation Plan (Executable Steps)
1. Build in‑memory node model (leader + followers).
2. Implement log append + commit apply.
3. Implement replication and majority commit.
4. Add dedup store and retry behavior.
5. Add fault injection for crash / drop / delay.
6. Write tests for crash, drop, delay.
7. Write postmortem report.

## 8) Files We Will Create
- `kvstore/` — implementation code.
- `tests/` — fault injection tests.
- `REPORT.md` — postmortem write‑up.

---

If you want me to start coding now, say: **“go build”**.

## 9) Why Each Unit Exists (Function + Link)
This section explains the *purpose* of each unit and how it connects to other units.

**Client API**
- Function: The only way users interact with the system; standardizes `Put` and `Get` operations.
- Link: Talks only to the Leader Node; depends on the Leader’s RPC handler and dedup logic to make retries safe.

**Leader Node**
- Function: The single decision‑maker that orders writes and guarantees consistency.
- Link: Receives requests from Client API and coordinates Followers via replication RPCs.

**Follower Nodes**
- Function: Provide redundancy so the system survives one crash.
- Link: Accept replicated log entries from Leader and acknowledge back; apply commit updates.

**Replication Log**
- Function: The authoritative sequence of writes; enables replay and consistent ordering.
- Link: Written first by Leader, then mirrored by Followers; drives Commit & Apply Engine.

**Commit & Apply Engine**
- Function: Turns ordered log entries into durable state in the KV map.
- Link: Reads from Replication Log; only advances after Leader confirms majority ack.

**Deduplication Store**
- Function: Prevents duplicate writes when clients retry with the same `request_id`.
- Link: Used by Leader Node before logging; updated after commit to remember results.

**Fault Injection Layer**
- Function: Simulates failure modes so correctness is tested, not assumed.
- Link: Wraps RPC calls between Leader and Followers (and optionally Client → Leader).

**Test Harness**
- Function: Proves guarantees by running controlled fault scenarios.
- Link: Uses Fault Injection Layer to create crashes, drops, and delays; inspects log and KV state.

**Postmortem Report**
- Function: Explains *why* the system holds up and where it might fail.
- Link: Summarizes observed behaviors from Test Harness and design tradeoffs.

## 10) Arrow Representation (Unit Flow)
Use these arrows to visualize how units depend on each other and how data moves.

**Write Path**
Client API → Leader RPC Handler → Dedup Check → Append Local Log → Replicate To Followers → Majority Ack Gate → Commit & Apply Engine → Reply To Client

**Read Path**
Client API → Leader RPC Handler → KV Read → Reply To Client

**Replication Path**
Leader Replication RPC → Follower RPC Handler → Append Entry → Apply Commit → ACK → Leader

**Fault Injection Overlay**
Fault Injection Layer → (wraps all RPC arrows above)

**Testing & Analysis**
Test Harness → Fault Injection Layer → System Units → Postmortem Report

## 11) Arrow Sketch (Visual Workflow Map)
Below is a compact, visible arrow sketch that shows the workflow at a glance.

```text
WRITE WORKFLOW
Client
  | Put(key, value, request_id)
  v
Leader
  | Dedup → Append Log → Replicate
  v                 \
Followers (2)        \
  | Ack               \
  v                   \
Leader (Majority Ack)  
  | Commit → Apply
  v
Client (Reply)

READ WORKFLOW
Client → Leader → KV Read → Client

FAULT INJECTION (wraps every RPC arrow)
[Crash / Drop / Delay] → (RPC call)

TESTING & REPORT
Test Harness → Fault Injection → System → Postmortem Report
```
