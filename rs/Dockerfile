FROM rust:1.76

WORKDIR /usr/src/myapp
COPY . .
RUN cargo install refinery_cli
ENV PATH="${PATH}:/sbin"
RUN apt-get update && apt-get install netcat-openbsd


CMD echo "sleep 1"&&sleep infinity&&refinery migrate&&cargo run --release
