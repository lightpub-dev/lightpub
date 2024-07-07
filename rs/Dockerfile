FROM rust:1.76-bullseye AS builder

WORKDIR /usr/src/app
COPY . .

RUN mkdir /out
ENV CARGO_HOME=/cache/cargo
ARG BUILD_TYPE
RUN --mount=type=cache,target=/usr/src/app/target --mount=type=cache,target=/cache/cargo ./docker_build.sh ${BUILD_TYPE}

FROM debian:bullseye AS runtime

RUN apt-get update -qq && apt-get install -qq -y netcat-openbsd

WORKDIR /lightpub
RUN mkdir db
COPY migrations ./migrations
COPY docker_start.sh .
COPY --from=builder /out/lightpub_api /usr/bin/lightpub_api

CMD ["./docker_start.sh"]
