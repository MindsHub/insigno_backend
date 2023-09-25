#!/bin/bash
set -o xtrace # print commands being executed

sudo docker stop postgres
sudo docker container rm -f postgres
if [[ $# == 1 ]] && [[ $1 == "stop" ]]; then
    echo "Just stopping the postgres container"
    exit 0
fi

sudo docker run --name postgres -p 5432:5432 -e POSTGRES_PASSWORD=test -e POSTGRES_USER=mindshub -d postgis/postgis
until diesel database reset --database-url="postgres://mindshub:test@localhost:5432/insignorocketdb"; do
    sleep 0.5;
done;
#& $($(echo "$(cargo tarpaulin --print-rust-flags --target-dir=target/tarpaulin)" | grep -v INFO)) cargo build
#$(cargo tarpaulin  --print-rust-flags | grep -v INFO)
#cargo tarpaulin --command build --skip-clean --target-dir=target/tarpaulin
#DATABASE_URL=

if [[ $# != 1 ]]; then
    # terminate gracefully when the user uses Ctrl+C (unless an argument was passed)
    trap 'sudo docker stop postgres; sudo docker container rm -f postgres; exit' INT
elif [[ $1 == "db" ]]; then
    echo "Skipping tests and leaving database open"
    exit 0
elif [[ $1 == "once" ]]; then
    echo "Running tests only once and then leaving database open"
    cargo tarpaulin --target-dir=target/tarpaulin --skip-clean -- --test-threads=1
    exit 0
fi

#cargo test -- --test-threads=1
cargo watch -s "cargo tarpaulin --target-dir=target/tarpaulin --skip-clean -- --test-threads=1"