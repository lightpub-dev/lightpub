generate-entity:
    sea-orm-cli generate entity -u mysql://root:lightpub@127.0.0.1:3306/lightpub --lib -o entity/src

generate-migration name:
    sea-orm-cli migrate generate {{name}}

migrate:
    sea-orm-cli migrate

generate-jwt:
    mkdir -p data
    sh ./generate-jwt-keys.sh

generate-vapid:
    mkdir -p data
    sh ./generate-vapid-keys.sh

dev: dev-up generate-jwt generate-vapid
    just migrate
    just dev-app

dev-down:
    docker compose -f docker-compose.dev.yml down lightpub_db lightpub_kv lightpub_nats lightpub_mathjax lightpub_typesense

dev-down-volume:
    docker compose -f docker-compose.dev.yml down lightpub_db lightpub_kv lightpub_nats lightpub_mathjax lightpub_typesense -v

dev-up:
    docker compose -f docker-compose.dev.yml up lightpub_db lightpub_kv lightpub_nats lightpub_mathjax lightpub_typesense -d --build

dev-app: generate-jwt
    env REGISTRATION_OPEN=true DEV_MODE=true RUST_LOG='debug,handlebars=info,sqlx=warn,html5ever=warn,async_nats=info' cargo run

dev-watch:
    watchexec -r -e rs -- just dev

dev-release: dev-up generate-jwt generate-vapid
    just migrate
    env REGISTRATION_OPEN=true DEV_MODE=true RUST_LOG=warning JWT_PUBLIC_KEY_FILE=data/jwtpub.pem JWT_SECRET_KEY_FILE=data/jwt.pem cargo run --release

fed-up:
    docker compose -f docker-compose.fed.yml --profile misskey --profile lightpub up --build -d

fed-down:
    docker compose -f docker-compose.fed.yml --profile misskey --profile lightpub down

fed-up-reload:
    docker compose -f docker-compose.fed.yml --profile misskey --profile lightpub up lightpub_web --build -d
