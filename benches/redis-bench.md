# Benchmarks

All benches should be performed without persistence features enabled.
We can add benches with persistence in the future.

## Single connection

```bash
redis-benchmark -c 1 -t ping,set,get,incr --csv
```

test | rps | avg_latency_ms | min_latency_ms | p50_latency_ms | p95_latency_ms | p99_latency_ms | max_latency_ms
--- | --- | --- | --- |--- |--- |--- |---
PING_INLINE | 15931.18 | 0.057 | 0.016 | 0.055 | 0.079 | 0.095 | 0.423
PING_MBULK | 15969.34 | 0.057 | 0.016 | 0.055 | 0.079 | 0.111 | 6.703
SET | 15008.25 | 0.061 | 0.024 | 0.063 | 0.071 | 0.095 | 1.631
GET | 15686.27 | 0.059 | 0.024 | 0.063 | 0.071 | 0.087 | 1.743
INCR | 14384.35 | 0.064 | 0.016 | 0.063 | 0.087 | 0.103 | 3.103

## Concurrent connections

```bash
redis-benchmark -t set,getredis-benchmark -t ping,set,get,incr --csv
```

test | rps | avg_latency_ms | min_latency_ms | p50_latency_ms | p95_latency_ms | p99_latency_ms | max_latency_ms
--- | --- | --- | --- |--- |--- |--- |---
PING_INLINE | 140449.44 | 0.276 | 0.120 | 0.263 | 0.367 | 0.487 | 1.103
PING_MBULK | 139275.77 | 0.278 | 0.104 | 0.263 | 0.375 | 0.487 | 5.287
SET | 103412.62 | 0.406 | 0.120 | 0.399 | 0.471 | 0.527 | 7.799
GET | 115340.26 | 0.359 | 0.104 | 0.351 | 0.455 | 0.559 | 6.919
INCR | 101419.88 | 0.414 | 0.120 | 0.407 | 0.503 | 0.567 | 8.031
