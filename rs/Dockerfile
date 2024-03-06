FROM rust:1.76

WORKDIR /usr/src/myapp
COPY . .
RUN cargo install refinery_cli

CMD refinery migrate;cargo run --release