#! /bin/bash

cargo run -- 127.0.0.1:6379 &
pid=$!

sleep 5

echo "Server started with pid $pid"

set=$(redis-cli set foo bar)
if [ "$set" != "OK" ]; then
    echo "Failed to set foo bar"
    kill $pid
    exit 1
fi

get=$(redis-cli get foo)
if [ "$get" != "bar" ]; then
    echo "Failed to get foo"
    kill $pid
    exit 1
fi

flushdb=$(redis-cli flushdb)
if [ "$flushdb" != "OK" ]; then
    echo "Failed to flushdb"
    kill $pid
    exit 1
fi

get=$(redis-cli get foo)
if [ "$get" != "not found" ]; then
    echo "Error flushing db $get"
    kill $pid
    exit 1
fi

echo "Tests passed"
kill $pid
