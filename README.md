# umd

In memory database for linux only. Killer feature, it is using io_uring.

## Development

If you are not on linux you can use the docker image to build and run the tests.

```zsh
make build-docker
make run-docker
```

```zsh
cargo run
```

#### Testing with curl
    
```zsh
curl --data "key=value" localhost:9999 # set
curl localhost:9999/key # get
```

