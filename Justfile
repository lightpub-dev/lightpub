generate-entity:
    sea-orm-cli generate entity -u mysql://root:lightpub@127.0.0.1:3306/lightpub --lib -o entity/src

generate-migration name:
    sea-orm-cli migrate generate {{name}}

migrate:
    sea-orm-cli migrate

generate-jwt:
    mkdir -p data
    openssl genrsa -out data/jwt.pem 4096
    openssl rsa -in data/jwt.pem -pubout -out data/jwtpub.pem

generate-jwt-if-not-exists:
    test -f data/jwt.pem || just generate-jwt

dev: dev-up generate-jwt-if-not-exists
    just migrate
    just dev-app

dev-down:
    docker compose -f docker-compose.dev.yml down lightpub_db lightpub_kv lightpub_nats

dev-down-volume:
    docker compose -f docker-compose.dev.yml down lightpub_db lightpub_kv lightpub_nats -v

dev-up:
    docker compose -f docker-compose.dev.yml up lightpub_db lightpub_kv lightpub_nats -d

dev-app: generate-jwt-if-not-exists
    env REGISTRATION_OPEN=true DEV_MODE=true RUST_LOG=debug JWT_PUBLIC_KEY_FILE=data/jwtpub.pem JWT_SECRET_KEY_FILE=data/jwt.pem cargo run

dev-watch:
    watchexec -r -e rs -- just dev

dev-release: dev-up generate-jwt-if-not-exists
    just migrate
    env REGISTRATION_OPEN=true DEV_MODE=true RUST_LOG=warning JWT_PUBLIC_KEY_FILE=data/jwtpub.pem JWT_SECRET_KEY_FILE=data/jwt.pem cargo run --release

fed-up:
    docker compose -f docker-compose.fed.yml --profile misskey --profile lightpub up --build -d

fed-down:
    docker compose -f docker-compose.fed.yml --profile misskey --profile lightpub down

fed-up-reload:
    docker compose -f docker-compose.fed.yml --profile misskey --profile lightpub up lightpub_web --build -d
