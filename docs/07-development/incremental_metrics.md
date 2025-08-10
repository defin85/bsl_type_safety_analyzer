# Incremental Metrics (Stabilized Set)

This document freezes the current public / debug telemetry fields returned by `bslAnalyzer.getIncrementalMetrics`.

## Exposure Levels

Two groups of metrics:

- Core (always exported)
- Debug (exported only when environment variable `BSL_ANALYZER_DEBUG_METRICS=1` is set for the LSP server process)

## Core Metrics

| Field | Description |
|-------|-------------|
| totalNodes | Total AST node count of hybrid (post selective rebuild) tree. |
| changedNodes | Count of nodes with differing fingerprints vs previous AST. |
| reusedNodes | Count of nodes whose fingerprints matched previous AST (shallow reuse). |
| reusedSubtrees | Count of subtree roots fully reused (parent changed or root). |
| reuseRatio | reusedNodes / totalNodes. |
| parseNs | Nanoseconds spent in text → parse (token + preliminary). |
| arenaNs | Nanoseconds spent assembling the arena AST. |
| fingerprintNs | Nanoseconds for fingerprint computation (full or partial depending on path). |
| semanticNs | Nanoseconds of semantic analysis (selective or full). |
| totalNs | Optional aggregate (parse+arena+fingerprint). |
| plannedRoutines | Routines targeted by the rebuild plan after dependency expansion. |
| replacedRoutines | Routines actually structurally replaced in the hybrid AST. |
| fallbackReason | `null` when selective path succeeded; otherwise one of: `module`, `heur_fraction`, `heur_absolute`, `exp_fraction`, `exp_absolute`. |
| initialTouched | Routines touched before dependency expansion. |
| expandedTouched | Routines touched after expansion (== plannedRoutines). |

## Debug Metrics (flagged)

| Field | Description |
|-------|-------------|
| innerReusedNodes | Deep internal (non-root) node reuse count inside replaced routines. |
| innerReuseRatio | innerReusedNodes / internal nodes of replaced routines. |
| recomputedFingerprints | Number of nodes whose fingerprints were recalculated during last partial pass. |
| semanticProcessedRoutines | Routines actually re-analyzed semantically in selective mode. |
| semanticReusedRoutines | plannedRoutines - semanticProcessedRoutines. |
| semanticSelectiveRatio | semanticReusedRoutines / plannedRoutines. |

## Semantics & Invariants

- totalNodes = changedNodes + reusedNodes.
- reuseRatio = reusedNodes / totalNodes (0 if totalNodes == 0).
- If fallbackReason != null then: replacedRoutines == 0 (currently) and debug deep reuse metrics may be absent.
- recomputedFingerprints <= totalNodes; in full rebuild path this value may be absent or equal to totalNodes depending on future implementation.

## Environment Flag

Set before launching analyzer / LSP:

```bash
BSL_ANALYZER_DEBUG_METRICS=1
```

On Windows PowerShell:
```powershell
$env:BSL_ANALYZER_DEBUG_METRICS=1
```

## Rationale

The core set is stable for integrations (CI dashboards, editor status bars). Debug set may change or expand without breaking consumers when the flag is disabled.

## Deferred / Frozen Roadmap

Advanced ideas intentionally postponed (see `docs/ROADMAP_FROZEN.md`):

- Semantic artifact cache keyed by routine fingerprint
- Fine-grained diff reason classification (span vs structure vs payload)
- Adaptive heuristic tuning based on historical reuse
- Structural hash pre-check layer
