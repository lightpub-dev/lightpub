# use the official Bun image
# see all versions at https://hub.docker.com/r/oven/bun/tags
FROM oven/bun:1 as base
WORKDIR /usr/src/app
COPY . .
RUN bun install

# run the tests
# USER bun
ENTRYPOINT [ "bun", "test" ]