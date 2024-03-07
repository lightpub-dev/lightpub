FROM rust:1.76

WORKDIR /usr/src/myapp
RUN cargo install refinery_cli
RUN apt-get update && apt-get install netcat-openbsd
COPY . .
ENV PATH="${PATH}:/sbin"
ENV DATABASE_URL="mysql://root:lightpub@lightpub-db:3306/lightpub"
RUN mv lightpub.yml.test lightpub.yml
CMD refinery migrate -e DATABASE_URL&&cargo run --release
