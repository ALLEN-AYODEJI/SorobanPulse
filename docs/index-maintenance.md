# Index Maintenance Guide

## Overview

Soroban Pulse automatically monitors index health and fragmentation, providing
visibility into index bloat and scheduling opportunities for `REINDEX`
operations. When indexes become bloated (accumulate too many dead tuples
relative to live tuples), query performance degrades due to increased disk I/O
and less efficient index scans.

## Features

### 1. Automatic Fragmentation Detection

The index monitor periodically queries PostgreSQL system catalogs to detect:

- **Bloat ratio**: Estimated dead-to-live tuple ratio per index, computed from
  `pg_stat_user_tables` `n_dead_tup` / `n_live_tup`.
- **Index size**: Size in bytes from `pg_relation_size()`.
- **Vacuum history**: Last manual VACUUM, ANALYZE, and autovacuum timestamps.

### 2. Fragmentation Alert Thresholds

| Environment Variable | Default | Description |
|---|---|---|
| `FRAGMENTATION_WARN_THRESHOLD` | `0.2` | Bloat ratio (0.0–1.0) above which a WARN is logged |
| `FRAGMENTATION_CRITICAL_THRESHOLD` | `0.5` | Bloat ratio above which an ERROR is logged and Prometheus alerts fire |
| `FRAGMENTATION_AUTO_REINDEX` | `false` | When `true`, critically bloated indexes are automatically rebuilt with `REINDEX INDEX CONCURRENTLY` |

**Alert levels:**
- **WARN** (bloat ≥ 20%): Non-blocking; logged for visibility. Consider
  scheduling a manual `REINDEX` during a maintenance window.
- **CRITICAL** (bloat ≥ 50%): Raised as an ERROR-level log entry. The
  `soroban_pulse_fragmented_indexes_total` Prometheus gauge is incremented so
  you can configure Alertmanager rules.

### 3. Prometheus Metrics

| Metric | Labels | Description |
|---|---|---|
| `soroban_pulse_index_bloat_ratio` | `table`, `index` | Current bloat ratio per index |
| `soroban_pulse_index_size_bytes` | `table`, `index` | Index size in bytes |
| `soroban_pulse_index_dead_tuples` | `table`, `index` | Estimated dead tuple count |
| `soroban_pulse_fragmented_indexes_total` | — | Number of indexes exceeding warn threshold |
| `soroban_pulse_fragmentation_checks_total` | — | Counter incremented on each check run |
| `soroban_pulse_reindex_operations_total` | `index` | Counter incremented on each REINDEX |
| `soroban_pulse_reindex_failures_total` | `index` | Counter incremented on REINDEX failures |

#### Example Alertmanager Rule

```yaml
groups:
  - name: soroban_pulse_indexes
    rules:
      - alert: IndexFragmentationCritical
        expr: soroban_pulse_index_bloat_ratio >= 0.5
        for: 1h
        labels:
          severity: warning
        annotations:
          summary: "Index {{ $labels.index }} on {{ $labels.table }} is critically bloated"
          description: "Bloat ratio is {{ $value | humanizePercentage }}. Consider running REINDEX."
```

### 4. Admin API Endpoints

All endpoints require an `ADMIN_API_KEY`.

#### GET /v1/admin/indexes/fragmentation

Returns a JSON array of all user indexes with their fragmentation status.

```bash
curl -H "Authorization: Bearer $ADMIN_API_KEY" \
  https://pulse.example.com/v1/admin/indexes/fragmentation
```

**Example response:**

```json
[
  {
    "table_name": "events",
    "index_name": "idx_events_ledger_desc",
    "bloat_ratio": 0.35,
    "dead_tuples": 50000,
    "live_tuples": 142857,
    "index_size_bytes": 1048576,
    "last_vacuum": "2026-07-20 03:00:00",
    "last_analyze": "2026-07-27 04:00:00",
    "last_autovacuum": "2026-07-26 12:30:00"
  }
]
```

#### POST /v1/admin/indexes/{index_name}/reindex

Manually trigger `REINDEX INDEX CONCURRENTLY` on a specific index.

```bash
curl -X POST -H "Authorization: Bearer $ADMIN_API_KEY" \
  https://pulse.example.com/v1/admin/indexes/idx_events_ledger_desc/reindex
```

**Response (202 Accepted):**

```json
{
  "status": "completed",
  "index": "idx_events_ledger_desc",
  "message": "REINDEX INDEX CONCURRENTLY completed successfully"
}
```

### 5. Automatic REINDEX Scheduling

When `FRAGMENTATION_AUTO_REINDEX=true` is set, the index monitor automatically
schedules `REINDEX INDEX CONCURRENTLY` for any index whose bloat ratio exceeds
the critical threshold.

**Important considerations:**

- `REINDEX INDEX CONCURRENTLY` does not block reads or writes. It builds a new
  index and swaps it in atomically.
- The operation runs in the background task's tick, which defaults to every
  24 hours (configurable via `INDEX_CHECK_INTERVAL_HOURS`).
- Failed REINDEX operations are logged at ERROR level and tracked in the
  `soroban_pulse_reindex_failures_total` metric.
- For large tables, REINDEX may take several minutes or longer. Monitor the
  PostgreSQL logs during this time.

### 6. Manual Maintenance Best Practices

Even without automatic REINDEX, follow these practices:

1. **Check fragmentation weekly**: Hit `GET /v1/admin/indexes/fragmentation`
   and review bloat ratios.
2. **VACUUM before REINDEX**: Run `VACUUM ANALYZE <table>` before reindexing
   to ensure statistics are fresh.
3. **REINDEX during off-peak**: Schedule manual `REINDEX INDEX CONCURRENTLY`
   during low-traffic periods.
4. **Monitor autovacuum**: Ensure `autovacuum_vacuum_scale_factor` and
   `autovacuum_analyze_scale_factor` are tuned appropriately. For large tables,
   consider lowering them (e.g., 0.01 instead of the default 0.05).
5. **Use pgstattuple for precision**: Install the `pgstattuple` extension
   (`CREATE EXTENSION IF NOT EXISTS pgstattuple`) for precise bloat measurement.

### 7. pgstattuple Extension

For the most accurate bloat metrics, install the PostgreSQL `pgstattuple`
extension:

```sql
CREATE EXTENSION IF NOT EXISTS pgstattuple;
```

The index monitor attempts to use `pgstattuple` first and falls back to
`pg_stat_user_tables` dead/live tuple estimation if the extension is not
available.

### 8. Configuration Reference

| Env Variable | Default | Description |
|---|---|---|
| `INDEX_CHECK_INTERVAL_HOURS` | `24` | How often the index monitor runs (also controls REINDEX scheduling interval) |
| `FRAGMENTATION_WARN_THRESHOLD` | `0.2` | Bloat ratio warn threshold |
| `FRAGMENTATION_CRITICAL_THRESHOLD` | `0.5` | Bloat ratio critical threshold |
| `FRAGMENTATION_AUTO_REINDEX` | `false` | Enable automatic REINDEX |

All can also be set in `config.toml` under the top-level keys
`fragmentation_warn_threshold`, `fragmentation_critical_threshold`, and
`fragmentation_auto_reindex`.
