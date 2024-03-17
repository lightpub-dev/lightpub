# use the official Bun image
# see all versions at https://hub.docker.com/r/oven/bun/tags
FROM oven/bun:debian as base

RUN apt-get update -qq && apt-get install -y -qq mariadb-client

WORKDIR /usr/src/app
COPY . .
RUN bun install

# run the tests
# USER bun
ENTRYPOINT [ "bun", "test" ]
