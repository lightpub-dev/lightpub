FROM oven/bun:latest

RUN apt-get update && apt-get install -y netcat


WORKDIR /usr/src/app
COPY package.json ./
COPY bun.lockb ./

RUN bun install

COPY tsconfig.json ./
COPY drizzle ./drizzle
COPY src ./src
COPY start.sh ./

CMD ["./start.sh"]

