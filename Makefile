.PHONY: generate-entity
generate-entity:
	sea-orm-cli generate entity -u mysql://root:lightpub@127.0.0.1:3306/lightpub --lib -o entity/src

.PHONY: generate-migration
generate-migration:
	sea-orm-cli migrate generate $(name)

.PHONY: migrate
migrate:
	sea-orm-cli migrate

.PHONY: generate-jwt
generate-jwt:
	mkdir -p data
	sh ./generate-jwt-keys.sh

.PHONY: generate-vapid
generate-vapid:
	mkdir -p data
	sh ./generate-vapid-keys.sh

.PHONY: dev
dev: dev-up generate-jwt generate-vapid
	$(MAKE) migrate
	$(MAKE) dev-app

.PHONY: dev-down
dev-down:
	docker compose -f docker-compose.dev.yml down lightpub_db lightpub_kv lightpub_nats lightpub_mathjax lightpub_typesense

.PHONY: dev-down-volume
dev-down-volume:
	docker compose -f docker-compose.dev.yml down lightpub_db lightpub_kv lightpub_nats lightpub_mathjax lightpub_typesense -v

.PHONY: dev-up
dev-up:
	docker compose -f docker-compose.dev.yml up lightpub_db lightpub_kv lightpub_nats lightpub_mathjax lightpub_typesense -d --build

.PHONY: dev-app
dev-app: generate-jwt
	env REGISTRATION_OPEN=true DEV_MODE=true RUST_LOG='debug,handlebars=info,sqlx=warn,html5ever=warn,async_nats=info' cargo run

.PHONY: dev-watch
dev-watch:
	watchexec -r -e rs -- $(MAKE) dev

.PHONY: dev-release
dev-release: dev-up generate-jwt generate-vapid
	$(MAKE) migrate
	env REGISTRATION_OPEN=true DEV_MODE=true RUST_LOG=warning JWT_PUBLIC_KEY_FILE=data/jwtpub.pem JWT_SECRET_KEY_FILE=data/jwt.pem cargo run --release

.PHONY: fed-up
fed-up:
	docker compose -f docker-compose.fed.yml --profile misskey --profile lightpub up --build -d

.PHONY: fed-down
fed-down:
	docker compose -f docker-compose.fed.yml --profile misskey --profile lightpub down

.PHONY: fed-up-reload
fed-up-reload:
	docker compose -f docker-compose.fed.yml --profile misskey --profile lightpub up lightpub_web --build -d
