# Kafka Event Publishing (Issue #705)

SorobanPulse supports publishing events to Apache Kafka for downstream consumption (issue #705). Events are published asynchronously to a configurable topic with automatic error handling and partition key strategy.

## Architecture

```
Indexer ──► Event stored in DB ──► Kafka publisher ──► Kafka broker ──► Consumer applications
                                  (async, fire-and-forget)
```

## Features

- **Asynchronous Publishing**: Non-blocking event publishing that doesn't delay indexer processing
- **Topic Management**: Support for configurable Kafka topics
- **Partition Key Strategy**: Uses contract_id as partition key for ordering within contracts
- **Error Handling**: Automatic error logging and metrics; failed publishes don't block indexing
- **Feature-Gated**: Compiled only when the `kafka` feature is enabled
- **Configuration**: Batch size and linger time tuning for throughput optimization

## Configuration

### Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `KAFKA_BROKERS` | — | Comma-separated Kafka broker addresses (e.g., `localhost:9092,localhost:9093`) |
| `KAFKA_TOPIC` | — | Kafka topic name for event publishing |
| `KAFKA_BATCH_SIZE` | 16384 | Producer batch size in bytes |
| `KAFKA_LINGER_MS` | 100 | Time to wait for batching (milliseconds) |

### Example: Enable Kafka Publishing

```bash
# Enable Kafka publishing with local broker
export KAFKA_BROKERS="localhost:9092"
export KAFKA_TOPIC="soroban-events"
export KAFKA_BATCH_SIZE=16384
export KAFKA_LINGER_MS=100

# Start the indexer
cargo run --features kafka
```

### Example: Production Setup

```bash
# Three-node Kafka cluster
export KAFKA_BROKERS="kafka-1.example.com:9092,kafka-2.example.com:9092,kafka-3.example.com:9092"
export KAFKA_TOPIC="soroban-events-prod"

# Optimize for throughput: larger batches, longer linger
export KAFKA_BATCH_SIZE=32768
export KAFKA_LINGER_MS=500

# Start the indexer
cargo run --features kafka
```

## Event Publishing

### Event Flow

1. **Indexer processes event**: Event is fetched from Soroban RPC and validated
2. **Event stored in database**: Event persists to PostgreSQL with timestamp
3. **Kafka publish (async)**: Event is serialized to JSON and sent to Kafka
4. **Acknowledgment**: Broker acknowledges receipt (fire-and-forget with acks=1)
5. **Error handling**: If publishing fails, error is logged but indexing continues

### Published Event Format

Events published to Kafka are serialized as JSON:

```json
{
  "id": "123e4567-e89b-12d3-a456-426614174000",
  "contract_id": "CABC123456789ABCDEF0123456789ABCDEF012345678",
  "event_type": "contract",
  "tx_hash": "abc123def456...",
  "ledger": 12345,
  "ledger_closed_at": "2026-03-14T10:30:00Z",
  "value": {
    "amount": "1000000",
    "recipient": "GABC123"
  },
  "topic": ["transfer"]
}
```

### Partition Key Strategy

Events are partitioned by `contract_id`:

```
contract_id: CABC123... ──► Partition 0
contract_id: CDEF456... ──► Partition 1
contract_id: CABC123... ──► Partition 0 (same contract → same partition)
```

**Benefits**:
- **Ordering**: Events from the same contract maintain order in a partition
- **Downstream processing**: Consumers can process contract events sequentially
- **Load distribution**: Multiple contracts spread across partitions

## Docker Setup

### Docker Compose with Kafka

```yaml
version: '3.8'

services:
  zookeeper:
    image: confluentinc/cp-zookeeper:7.5.0
    environment:
      ZOOKEEPER_CLIENT_PORT: 2181
      ZOOKEEPER_SYNC_LIMIT: 2
      ZOOKEEPER_INIT_LIMIT: 5

  kafka:
    image: confluentinc/cp-kafka:7.5.0
    depends_on:
      - zookeeper
    ports:
      - "9092:9092"
    environment:
      KAFKA_BROKER_ID: 1
      KAFKA_ZOOKEEPER_CONNECT: zookeeper:2181
      KAFKA_ADVERTISED_LISTENERS: PLAINTEXT://kafka:29092,PLAINTEXT_HOST://localhost:9092
      KAFKA_LISTENER_SECURITY_PROTOCOL_MAP: PLAINTEXT:PLAINTEXT,PLAINTEXT_HOST:PLAINTEXT
      KAFKA_INTER_BROKER_LISTENER_NAME: PLAINTEXT
      KAFKA_OFFSETS_TOPIC_REPLICATION_FACTOR: 1
      KAFKA_AUTO_CREATE_TOPICS_ENABLE: "true"

  api:
    build: .
    depends_on:
      - postgres
      - kafka
    environment:
      DATABASE_URL: postgres://postgres:postgres@postgres/soroban_pulse
      KAFKA_BROKERS: kafka:29092
      KAFKA_TOPIC: soroban-events
```

## Kubernetes Deployment

### StatefulSet with Kafka

```yaml
apiVersion: v1
kind: Service
metadata:
  name: kafka-broker
  labels:
    app: kafka
spec:
  ports:
    - port: 9092
      name: client
  clusterIP: None
  selector:
    app: kafka
---
apiVersion: apps/v1
kind: StatefulSet
metadata:
  name: kafka
spec:
  serviceName: kafka-broker
  replicas: 3
  selector:
    matchLabels:
      app: kafka
  template:
    metadata:
      labels:
        app: kafka
    spec:
      containers:
        - name: kafka
          image: confluentinc/cp-kafka:7.5.0
          ports:
            - containerPort: 9092
          env:
            - name: KAFKA_BROKER_ID
              valueFrom:
                fieldRef:
                  fieldPath: metadata.name
            - name: KAFKA_ZOOKEEPER_CONNECT
              value: "zookeeper-0.zookeeper.default.svc.cluster.local:2181"
---
apiVersion: apps/v1
kind: Deployment
metadata:
  name: soroban-pulse-api
spec:
  template:
    spec:
      containers:
        - name: api
          env:
            - name: KAFKA_BROKERS
              value: "kafka-0.kafka-broker.default.svc.cluster.local:9092,kafka-1.kafka-broker.default.svc.cluster.local:9092,kafka-2.kafka-broker.default.svc.cluster.local:9092"
            - name: KAFKA_TOPIC
              value: "soroban-events"
```

## Consumer Examples

### Python Consumer

```python
from kafka import KafkaConsumer
import json

# Create consumer
consumer = KafkaConsumer(
    'soroban-events',
    bootstrap_servers=['localhost:9092'],
    group_id='event-processor',
    auto_offset_reset='earliest',
    value_deserializer=lambda m: json.loads(m.decode('utf-8'))
)

# Process events
for message in consumer:
    event = message.value
    print(f"Contract: {event['contract_id']}, Ledger: {event['ledger']}")
    
    # Process event data
    if event['value']:
        amount = event['value'].get('amount')
        print(f"Amount: {amount}")
```

### Go Consumer

```go
package main

import (
    "encoding/json"
    "fmt"
    "github.com/segmentio/kafka-go"
)

func main() {
    reader := kafka.NewReader(kafka.ReaderConfig{
        Brokers: []string{"localhost:9092"},
        Topic:   "soroban-events",
        GroupID: "event-processor",
    })
    defer reader.Close()

    for {
        message, err := reader.ReadMessage(context.Background())
        if err != nil {
            panic(err)
        }

        var event map[string]interface{}
        json.Unmarshal(message.Value, &event)
        fmt.Printf("Contract: %v\n", event["contract_id"])
    }
}
```

### Node.js Consumer

```javascript
const { Kafka } = require('kafkajs');

const kafka = new Kafka({
  clientId: 'event-processor',
  brokers: ['localhost:9092'],
});

const consumer = kafka.consumer({ groupId: 'event-processor' });

(async () => {
  await consumer.connect();
  await consumer.subscribe({ topic: 'soroban-events' });
  
  await consumer.run({
    eachMessage: async ({ topic, partition, message }) => {
      const event = JSON.parse(message.value.toString());
      console.log(`Contract: ${event.contract_id}, Ledger: ${event.ledger}`);
    },
  });
})();
```

## Topic Management

### Create Topic (Manual)

```bash
# Using Kafka CLI
kafka-topics.sh --create \
  --topic soroban-events \
  --bootstrap-server localhost:9092 \
  --partitions 3 \
  --replication-factor 3
```

### Auto Topic Creation

Kafka can automatically create topics on first message (default enabled in docker-compose):

```yaml
kafka:
  environment:
    KAFKA_AUTO_CREATE_TOPICS_ENABLE: "true"
```

### List Topics

```bash
kafka-topics.sh --list \
  --bootstrap-server localhost:9092
```

### Monitor Topic

```bash
kafka-console-consumer.sh \
  --bootstrap-server localhost:9092 \
  --topic soroban-events \
  --from-beginning
```

## Monitoring and Metrics

### Metrics Published

SorobanPulse publishes Kafka metrics:

- `kafka_publish_success`: Count of successful event publications
- `kafka_publish_error`: Count of failed publications
- `kafka_publish_latency`: Publication latency in milliseconds

### Example: Prometheus Configuration

```yaml
scrape_configs:
  - job_name: soroban-pulse
    static_configs:
      - targets: ['localhost:3000']
    metrics_path: /metrics
```

### Querying Metrics

```bash
# Check publication success rate
curl http://localhost:3000/metrics | grep kafka_publish_success

# Check error rate
curl http://localhost:3000/metrics | grep kafka_publish_error
```

## Error Handling

### Publisher Errors

Failed Kafka publishes are logged and tracked but don't block indexing:

```
[WARN] Kafka publish failed: broker unavailable
  contract_id: "CABC123...",
  error: "delivery: broker unavailable (Local: Broker transport failure)"
```

### Retry Strategy

Currently uses fire-and-forget with timeout. For guaranteed delivery:

1. **Implement persistent queue**: Buffer events in database if Kafka is unavailable
2. **Retry logic**: Periodically retry failed publishes
3. **Dead letter topic**: Send failed events to a dead-letter topic for analysis

### Broker Unavailability

If Kafka broker is unavailable:
- Events are still indexed in PostgreSQL
- Kafka publishes fail silently
- Indexer continues processing
- Monitor logs for `kafka_publish_error` metrics

## Performance Tuning

### Batch Size and Linger Time

Adjust for your use case:

**Throughput (default)**:
```bash
export KAFKA_BATCH_SIZE=16384      # 16 KB
export KAFKA_LINGER_MS=100         # 100 ms
```

**High Throughput**:
```bash
export KAFKA_BATCH_SIZE=65536      # 64 KB
export KAFKA_LINGER_MS=500         # 500 ms
```

**Low Latency**:
```bash
export KAFKA_BATCH_SIZE=1024       # 1 KB
export KAFKA_LINGER_MS=10          # 10 ms
```

### Network Optimization

For high-volume deployments:

- **Increase partitions**: More partitions = more parallel processing
- **Tune replica factor**: 3 for production (balance between durability and latency)
- **Enable compression**: Enable snappy/gzip compression in broker

## Troubleshooting

### "Broker unavailable"

Verify Kafka broker is running:
```bash
# Test connectivity
nc -zv localhost 9092

# Check broker status
kafka-broker-api-versions.sh --bootstrap-server localhost:9092
```

### "Topic does not exist"

Manually create the topic:
```bash
kafka-topics.sh --create \
  --topic soroban-events \
  --bootstrap-server localhost:9092
```

### "Connection refused"

Verify `KAFKA_BROKERS` environment variable:
```bash
echo $KAFKA_BROKERS
# Should output: localhost:9092,localhost:9093,...
```

### No events published

Check logs for errors:
```bash
# If kafka feature not compiled
# Rebuild with feature flag
cargo build --features kafka
```

## Feature Flag

Kafka support is optional and compiled only when enabled:

```bash
# Include Kafka
cargo build --features kafka

# Exclude Kafka (default)
cargo build
```

## See Also

- [Multi-Tenancy Deployment](./multi-tenancy.md)
- [Event Encryption](./event-encryption.md)
- [Monitoring](./monitoring.md)
- [Kafka Documentation](https://kafka.apache.org/)
