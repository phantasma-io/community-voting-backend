[private]
just:
    just -l

run:
    cargo run

build:
    cargo build

docker-run:
    docker compose up -d

docker-stop:
    docker compose stop

docker-build:
    docker compose build
