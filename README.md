# Lightpub

![server-test](https://github.com/lightpub-dev/lightpub/actions/workflows/server-test.yaml/badge.svg)

## Server 建て方
1. `cd rs`
2. `docker compose up -d`
3. `cargo install refinery_cli`
4. `refinery migrate`
5. `cargo run`

## Frontend 建て方
1. `cd frontend`
2. `yarn install`
3. `yarn run dev`


Rust version is available in rs directory.
