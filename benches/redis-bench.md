# Benchmarks

All benches should be performed without persistence features enabled.
We can add benches with persistence in the future.

## Single connection

```bash
redis-benchmark -c 1 -t ping,set,get,incr --csv
```

test | rps | avg_latency_ms | min_latency_ms | p50_latency_ms | p95_latency_ms | p99_latency_ms | max_latency_ms
--- | --- | --- | --- |--- |--- |--- |---
PING_INLINE | 19186.49 | 0.048 | 0.008 | 0.047 | 0.063 | 0.071 | 0.823
PING_MBULK  | 19596.31 | 0.047 | 0.016 | 0.047 | 0.055 | 0.063 | 1.959
SET         | 17467.25 | 0.053 | 0.024 | 0.055 | 0.063 | 0.079 | 0.911
GET         | 17972.68 | 0.052 | 0.016 | 0.055 | 0.063 | 0.071 | 0.151
INCR        | 15417.82 | 0.061 | 0.024 | 0.055 | 0.087 | 0.095 | 0.623

## Concurrent connections

```bash
redis-benchmark -t set,getredis-benchmark -t ping,set,get,incr --csv
```

test | rps | avg_latency_ms | min_latency_ms | p50_latency_ms | p95_latency_ms | p99_latency_ms | max_latency_ms
--- | --- | --- | --- |--- |--- |--- |---
PING_INLINE | 176366.86 | 0.242 | 0.120 | 0.231 | 0.335 | 0.439 | 6.471"
PING_MBULK  | 187265.92 | 0.229 | 0.056 | 0.223 | 0.287 | 0.319 | 6.007"
SET         | 98328.42  | 0.464 | 0.104 | 0.463 | 0.495 | 0.535 | 8.351"
GET"        | 110253.59 | 0.408 | 0.064 | 0.399 | 0.487 | 0.575 | 9.031"
INCR        | 94786.73  | 0.484 | 0.120 | 0.479 | 0.543 | 0.591 | 11.183"
