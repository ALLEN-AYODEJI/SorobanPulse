# Environment Variable Reference


This file is **auto-generated** from `src/config.rs`. Do not edit it by hand — run `cargo run --bin gen_env_docs -- --write` instead.

| Variable | Default | Description |
|---|---|---|
| `DATABASE_URL` | `(required)` | PostgreSQL connection string. Also loadable from a file path via DATABASE_URL_FILE. |
| `DATABASE_REPLICA_URL` | `(none)` | Optional read-replica URL. When set, HTTP handlers use this pool; the indexer uses the primary. |
| `STELLAR_RPC_URL` | `https://soroban-testnet.stellar.org` | Soroban RPC endpoint. Must be HTTPS unless ALLOW_INSECURE_RPC=true. |
| `STELLAR_RPC_FALLBACK_URLS` | `(none)` | Comma-separated list of fallback RPC endpoints tried when the primary fails. |
| `STELLAR_RPC_HEADERS` | `(none)` | Semicolon-separated "Name: Value" pairs injected into every RPC request. Values are never logged. |
| `START_LEDGER` | `0` | Ledger sequence to start indexing from. 0 = start from the latest ledger at boot. |
| `START_LEDGER_FALLBACK` | `false` | When true, fall back to the latest ledger if START_LEDGER is no longer available on the RPC. |
| `PORT` | `3000` | HTTP server listening port. |
| `API_KEY` | `(disabled)` | Optional API key for request authentication. Also accepts API_KEY_SECONDARY for rotation. When unset all requests pass through. |
| `ADMIN_API_KEY` | `(disabled)` | Key required for /v1/admin/* endpoints, independent of API_KEY. Also accepts ADMIN_API_KEY_SECONDARY. When unset, admin endpoints fall back to API_KEY. |
| `DB_MAX_CONNECTIONS` | `10` | Maximum number of connections in the PostgreSQL connection pool. |
| `DB_MIN_CONNECTIONS` | `2` | Minimum number of idle connections kept open in the PostgreSQL pool. |
| `DB_IDLE_TIMEOUT_SECS` | `600` | Seconds before an idle connection is closed and removed from the pool. |
| `DB_MAX_LIFETIME_SECS` | `1800` | Maximum lifetime in seconds for a pooled database connection. |
| `DB_TEST_BEFORE_ACQUIRE` | `true` | When true, a cheap health-check query is run before returning a connection from the pool. |
| `BEHIND_PROXY` | `false` | Set to true when running behind a reverse proxy so the real client IP is extracted from X-Forwarded-For. |
| `RPC_CONNECT_TIMEOUT_SECS` | `5` | TCP connection timeout in seconds for RPC requests. |
| `RPC_REQUEST_TIMEOUT_SECS` | `30` | Total request timeout in seconds for RPC calls. |
| `ALLOWED_ORIGINS` | `*` | Comma-separated CORS allowed origins. Wildcard (*) is rejected in production-like environments. |
| `RATE_LIMIT_PER_MINUTE` | `60` | Maximum HTTP requests per IP per minute. Set to 0 to disable rate limiting. |
| `INDEXER_LAG_WARN_THRESHOLD` | `100` | Number of ledgers behind the tip before a warning is emitted. |
| `INDEXER_STALL_TIMEOUT_SECS` | `60` | Seconds without a successful indexer poll before the health check reports unhealthy. |
| `DB_STATEMENT_TIMEOUT_MS` | `5000` | PostgreSQL statement timeout in milliseconds applied to every query. |
| `INDEXER_POLL_INTERVAL_MS` | `5000` | Milliseconds between indexer polls when the chain is caught up. Range: [100, 60000]. |
| `INDEXER_ERROR_BACKOFF_MS` | `10000` | Milliseconds to wait after an indexer error before retrying. Range: [100, 60000]. |
| `SSE_KEEPALIVE_SECS` | `15` | SSE keep-alive ping interval in seconds (1–60). Prevents proxies from closing idle streams. |
| `SSE_MAX_CONNECTIONS` | `1000` | Global maximum number of concurrent SSE connections the server will accept. |
| `SSE_DRAIN_TIMEOUT_SECS` | `5` | Seconds to wait for SSE clients to drain before forcefully closing on shutdown. Range: [1, 60]. |
| `SSE_MULTI_MAX_CONTRACT_IDS` | `100` | Maximum number of contract IDs per multiplexed SSE connection. |
| `ENVIRONMENT` | `development` | Deployment environment: development, staging, or production. Controls strictness of config validation. |
| `MAX_BODY_SIZE_BYTES` | `1048576` | Maximum HTTP request body size in bytes (default 1 MiB). |
| `LOG_SAMPLE_RATE` | `1` | Fraction of requests to emit trace-level logs for (1 = all, 10 = every 10th). |
| `WEBHOOK_URL` | `(none)` | Target URL for webhook event delivery. When unset, webhook notifications are disabled. |
| `WEBHOOK_SECRET` | `(none)` | HMAC-SHA256 secret used to sign webhook payloads (X-Webhook-Signature header). |
| `WEBHOOK_CONTRACT_FILTER` | `(none)` | Comma-separated list of contract IDs to deliver via webhook. Empty = deliver all. |
| `WEBHOOK_RATE_LIMIT_PER_MINUTE` | `(none)` | Maximum webhook notifications per minute. 0 or unset disables the limit. |
| `WEBHOOK_RATE_LIMIT_PER_HOUR` | `(none)` | Maximum webhook notifications per hour. 0 or unset disables the limit. |
| `WEBHOOK_REQUIRE_HTTPS` | `false (true in production)` | When true, reject webhook URLs that do not use HTTPS. Automatically true in production-like environments. |
| `INDEXER_EVENT_TYPES` | `(all types)` | Comma-separated list of event types to index: contract, diagnostic, system. Empty = index all. |
| `EVENT_DATA_ENCRYPTION_KEY` | `(none)` | 64-hex-character AES-GCM key (32 bytes) for encrypting event_data at rest. Requires the encryption feature flag. |
| `EVENT_DATA_ENCRYPTION_KEY_OLD` | `(none)` | Previous encryption key for zero-downtime key rotation. Requires the encryption feature flag. |
| `INDEX_CHECK_INTERVAL_HOURS` | `24` | How often (in hours) the index usage monitor runs to report on PostgreSQL index utilisation. |
| `HEALTH_CHECK_TIMEOUT_MS` | `2000` | Timeout in milliseconds for the database ping performed by the health check endpoint. |
| `TLS_CERT_FILE` | `(none)` | Path to the PEM-encoded TLS certificate file for HTTPS termination. |
| `TLS_KEY_FILE` | `(none)` | Path to the PEM-encoded TLS private key file for HTTPS termination. |
| `BLOOM_FILTER_FP_RATE` | `0.001` | Target false-positive rate for the bloom filter used in contract existence checks. |
| `BLOOM_FILTER_CAPACITY` | `1000000` | Expected maximum number of items in the bloom filter before the FP rate degrades. |
| `ENABLE_CONTENT_DEDUP` | `false` | When true, a SHA-256 fingerprint of each event is checked before insert to prevent content-identical duplicates. |
| `DEDUP_WINDOW_SECS` | `3600` | Lookback window in seconds for content-fingerprint deduplication checks. |
| `KINESIS_STREAM_NAME` | `(none)` | AWS Kinesis stream name for event delivery. Requires the kinesis feature flag. |
| `AWS_REGION` | `(none)` | AWS region for Kinesis/SQS streaming integrations. |
| `SQS_QUEUE_URL` | `(none)` | AWS SQS queue URL for event delivery. Requires the sqs feature flag. |
| `SQS_DLQ_URL` | `(none)` | AWS SQS dead-letter queue URL for failed event deliveries. |
| `SQS_BATCH_SIZE` | `10` | Number of events per SQS SendMessageBatch call (max 10). |
| `EVENT_HUBS_CONNECTION_STRING` | `(none)` | Azure Event Hubs connection string for event streaming. |
| `EVENT_HUB_NAME` | `(none)` | Azure Event Hub name within the namespace. |
| `EVENT_HUBS_PARTITION_STRATEGY` | `contract_id` | Partition key strategy for Azure Event Hubs: contract_id or round_robin. |
| `PUBSUB_PROJECT_ID` | `(none)` | GCP project ID for Pub/Sub event delivery. Requires the pubsub feature flag. |
| `PUBSUB_TOPIC_ID` | `(none)` | GCP Pub/Sub topic ID for event delivery. |
| `PUBSUB_ENABLE_MESSAGE_ORDERING` | `true` | When true, Pub/Sub messages are published with an ordering key so subscribers receive them in order. |
| `MAX_EVENT_DATA_BYTES` | `65536` | Maximum allowed size of serialised event_data in bytes (default 64 KiB). |
| `EXPORT_MAX_ROWS` | `10000` | Maximum number of rows returned by the CSV/JSON export endpoints. |
| `CONTRACT_COUNT_CACHE_SIZE` | `1000` | Maximum number of entries in the per-contract event count cache. |
| `CONTRACT_COUNT_CACHE_TTL_SECS` | `30` | TTL in seconds for entries in the per-contract event count cache. |
| `INDEXER_LOCK_RETRY_SECS` | `30` | How often standby replicas retry acquiring the advisory lock to become the active indexer. |
| `MULTI_TENANT` | `false` | When true, every API key is mapped to a tenant_id and all queries are scoped to that tenant. |
| `TENANT_RATE_LIMIT_PER_MINUTE` | `(none)` | Per-tenant request rate limit (requests per minute). Only effective when MULTI_TENANT=true. |
| `TENANT_CONTRACT_FILTER` | `(none)` | Per-tenant contract allow-list. Format: "tenant1:C1,C2;tenant2:C3". Only effective when MULTI_TENANT=true. |
| `INDEXER_TENANT_ID` | `(none)` | The tenant_id stamped on every event this indexer instance inserts. Only meaningful when MULTI_TENANT=true. |
| `EMAIL_SMTP_HOST` | `(none)` | SMTP hostname for outbound email notifications. When unset, email notifications are disabled. |
| `EMAIL_SMTP_PORT` | `587` | SMTP port for outbound email (typically 587 for STARTTLS or 465 for implicit TLS). |
| `EMAIL_SMTP_USER` | `(none)` | SMTP authentication username. |
| `EMAIL_SMTP_PASSWORD` | `(none)` | SMTP authentication password. |
| `EMAIL_FROM` | `(none)` | Sender address used in the From header of notification emails. |
| `EMAIL_TO` | `(none)` | Comma-separated list of recipient addresses for email notifications. |
| `EMAIL_CONTRACT_FILTER` | `(none)` | Comma-separated contract IDs to send email notifications for. Empty = notify for all contracts. |
| `EMAIL_PUBLIC_BASE_URL` | `http://localhost:<PORT>` | Public base URL used to build unsubscribe links in notification emails (e.g. https://pulse.example.com). |
| `TWILIO_ACCOUNT_SID` | `(none)` | Twilio Account SID for SMS notifications. When unset, SMS notifications are disabled. |
| `TWILIO_AUTH_TOKEN` | `(none)` | Twilio Auth Token for SMS notifications. |
| `TWILIO_FROM_NUMBER` | `(none)` | Twilio sender phone number in E.164 format (e.g. +15551234567). |
| `SMS_TO_NUMBERS` | `(none)` | Comma-separated list of E.164 phone numbers to receive SMS notifications. |
| `SMS_CONTRACT_FILTER` | `(none)` | Comma-separated contract IDs to send SMS notifications for. Empty = notify for all contracts. |
| `REDIS_URL` | `(none)` | Redis connection URL for event streaming (e.g. redis://localhost:6379). When unset, Redis streaming is disabled. |
| `REDIS_STREAM_KEY` | `(none)` | Redis stream key name for event publishing (e.g. soroban:events). |
| `REDIS_BUFFER_MAX_SIZE` | `10000` | Maximum number of events buffered in memory during a Redis outage before dropping. |
| `STATS_REFRESH_INTERVAL_SECS` | `3600` | How often (in seconds) the background statistics refresh job runs. |
| `RETENTION_DAYS` | `90` | Number of days to retain indexed events before they are pruned. |
| `PRUNING_INTERVAL_HOURS` | `24` | How often (in hours) the event pruning job runs to enforce RETENTION_DAYS. |
| `SSE_REPLAY_MAX_EVENTS` | `1000` | Maximum number of past events replayed to a reconnecting SSE client via Last-Event-ID. |
| `SSE_MAX_CONNECTIONS_PER_IP` | `10` | Maximum concurrent SSE connections allowed per client IP address. |
| `SSE_MAX_LAG_BEFORE_DISCONNECT` | `0` | Number of events a slow SSE consumer may fall behind before being forcefully disconnected. 0 = disabled. |
| `INDEXER_IGNORE_CHECKPOINT` | `false` | When true, the indexer ignores any persisted ledger checkpoint and always starts from START_LEDGER. |
| `RPC_MAX_EVENTS_PER_PAGE` | `200` | Maximum number of events to request per RPC getEvents page. |
| `MAX_LEDGER_RANGE` | `100000` | Maximum ledger range allowed for diff/range query endpoints. |
| `SLOW_QUERY_THRESHOLD_MS` | `1000` | Queries exceeding this duration (ms) are logged at WARN level and counted in metrics. |
| `EVENT_TRANSFORM_SCRIPT` | `(none)` | Path to a Lua script for transforming events before storage. Requires the lua feature flag. |
| `EVENT_TRANSFORM_TIMEOUT_MS` | `100` | Maximum execution time in milliseconds for the Lua event transformation script. |
| `WEBHOOK_NOTIFICATION_FORMAT` | `raw` | Webhook payload format: raw (full event JSON) or template (rendered via WEBHOOK_MESSAGE_TEMPLATE). |
| `WEBHOOK_MESSAGE_TEMPLATE` | `(none)` | Handlebars template string for webhook payload rendering when WEBHOOK_NOTIFICATION_FORMAT=template. |
| `PAGERDUTY_ROUTING_KEY` | `(none)` | PagerDuty Events API v2 routing key (integration key). When unset, PagerDuty notifications are disabled. |
| `PAGERDUTY_SERVICE_NAME` | `Soroban Pulse` | Service name displayed in PagerDuty incidents. |
| `PAGERDUTY_CONTRACT_FILTER` | `(none)` | Comma-separated contract IDs to trigger PagerDuty alerts for. Empty = alert on all contracts. |
| `PAGERDUTY_EVENT_TYPE_FILTER` | `(none)` | Comma-separated event types to trigger PagerDuty alerts for. Empty = alert on all event types. |
| `PAGERDUTY_SEVERITY_MAPPING` | `{"contract":"error","diagnostic":"warning","system":"info"}` | JSON object mapping event types to PagerDuty severity levels (critical, error, warning, info). |
| `PAGERDUTY_AUTO_RESOLVE` | `true` | When true, PagerDuty incidents are automatically resolved after PAGERDUTY_AUTO_RESOLVE_THRESHOLD_MINUTES. |
| `PAGERDUTY_AUTO_RESOLVE_THRESHOLD_MINUTES` | `30` | Minutes of inactivity after which a PagerDuty incident is automatically resolved. |
| `STATS_CACHE_TTL_SECS` | `30` | TTL in seconds for cached statistics responses. |
| `SSE_SHUTDOWN_GRACE_PERIOD_MS` | `200` | Milliseconds between broadcasting the SSE close event and dropping the broadcast sender on shutdown. |
| `EMAIL_MAX_CONTRACTS_IN_DIGEST` | `20` | Maximum number of distinct contracts grouped in a single email digest. |
| `NOTIFICATION_DEFAULT_PRIORITY` | `medium` | Default notification priority level: low, medium, high, or critical. |
| `NOTIFICATION_PRIORITY_RULE_PATH` | `(none)` | JSONPath expression used to extract a value for priority rule evaluation. |
| `NOTIFICATION_PRIORITY_RULE_VALUE` | `(none)` | Value to match against the extracted JSONPath result for priority escalation. |
| `NOTIFICATION_PRIORITY_RULE_PRIORITY` | `(none)` | Priority level assigned when the rule matches: low, medium, high, or critical. |
| `ESCALATION_PRIMARY_CHANNEL` | `(none)` | Notification channel (webhook, email, sms, pagerduty) to try first before escalating. |
| `ESCALATION_CHANNEL` | `(none)` | Fallback notification channel to use after ESCALATION_DELAY_MINUTES if the primary channel fails. |
| `ESCALATION_DELAY_MINUTES` | `30` | Minutes to wait before escalating a failed notification to the ESCALATION_CHANNEL. |
| `ONCALL_PROVIDER` | `(none)` | On-call rotation provider: pagerduty. When unset, on-call routing is disabled. |
| `ONCALL_PAGERDUTY_API_KEY` | `(none)` | PagerDuty REST API key used to query on-call schedules. |
| `ONCALL_SCHEDULE_ID` | `(none)` | PagerDuty schedule ID to query for the current on-call responder. |
| `ONCALL_SCHEDULE_CACHE_TTL_SECS` | `300` | TTL in seconds for the cached on-call schedule lookup result. |
| `EVENT_COMPRESSION_ENABLED` | `false` | When true, event_data is gzip-compressed before writing to the event_data_compressed column. |
| `EVENT_COMPRESSION_MIGRATION_BATCH_SIZE` | `500` | Number of events processed per batch when migrating existing events to compressed format. |
| `CHAIN_ID` | `mainnet` | Chain identifier stamped on every indexed event (e.g. mainnet, testnet, futurenet). |
| `ADDITIONAL_CHAIN_IDS` | `(none)` | Comma-separated list of additional network chain_ids to spin up parallel indexer workers for. |
| `LEDGER_HASH_TRACKING_ENABLED` | `false` | When true, ledger hashes are stored and chain continuity is verified on startup. |
| `LEDGER_HASH_VALIDATION_DEPTH` | `1000` | Number of ledgers to check during startup hash-chain validation. |
| `ABI_CACHE_TTL_SECS` | `86400` | TTL in seconds for in-process contract ABI cache entries (default 24 h). |
| `ABI_CACHE_MAX_ENTRIES` | `1000` | Maximum number of contract ABIs kept in the in-process cache. |
| `SSE_RING_BUFFER_CAPACITY` | `10000` | Maximum number of events stored in the in-memory SSE replay ring buffer. Oldest events are evicted when full. |
| `QUERY_CACHE_TTL_SECS` | `300` | TTL in seconds for cached query results. Clamped to [300, 3600]. |
| `QUERY_CACHE_MAX_CAPACITY` | `1000` | Maximum number of entries in the query result cache. |
| `ENABLE_PUSH_PRELOAD` | `false` | When true, responses include HTTP Link preload headers for contract schema/ABI and the /v1/push/* endpoints are enabled. |

