# Frozen Roadmap (Incremental / Reuse Advanced Features)

This file records ideas deliberately postponed to keep the current incremental implementation stable.

## Deferred Items

1. Semantic cache keyed by routine fingerprint (store symbol tables / resolved types per routine).
2. Diff reason classification for fingerprint mismatches (span shift, payload change, structural change).
3. Adaptive heuristic thresholds (dynamic tuning of touched fraction / absolute caps).
4. Pre-aggregated structural hash layer to short‑circuit subtree recomputation.
5. Extended deep reuse metrics (depth distributions, changed-node depth histogram).
6. Runtime memory accounting (semantic cache size, interner deltas per edit).
7. LSP debug command `bslAnalyzer.debugIncrementalPlan` returning structured plan + diff reasons.
8. Baseline benchmarking harness for large real configurations (aggregate stats & percentiles).

## Exit Criteria for Unfreezing

Unfreeze occurs only if at least one of:
- Measured mean incremental time > target SLA (e.g. > 120ms for small edits) on reference dataset.
- ReuseRatio median < 0.80 across typical edit scenarios.
- User-facing need: feature request requiring visibility into a deferred dimension.

## Governance

Changes to frozen scope require adding a brief proposal section here with: motivation, expected impact, complexity estimate. Keep stability for tooling relying on current core metrics.
