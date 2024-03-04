# umd

In memory database for linux only. Killer feature, it is using io_uring.

## Architecture

All documentation about the architecture is in the `docs` folder and in the comments of the code.

## Development

If you are not on linux you can use the docker image to build and run the tests.

```zsh
make build-docker
make run-docker
```

```zsh
cargo run
```

## Test

It is possibile using redis client or redis protocol for testing.
In benchmark for example we used `redis-benchmark`, but the plan is to working on custom protocol and better client.

#### Testing with curl

With curl only few commands are supported and the plan is to support the very foundamentals commands and retrieve stats.

```zsh
curl --data "value" localhost:9999/key          # set
curl --data "value EX 10" localhost:9999/key    # set with TTL 10s
curl localhost:9999/key                         # get
curl -X POST localhost:9999/key                 # del
```

## Benchmark

Right now we just have [redis-bench](./benches/redis-bench.md)

And a couple of scripts in the `scripts` folder.