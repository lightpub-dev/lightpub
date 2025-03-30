.PHONY: install-deps
install-deps:
	go install -tags 'mysql' github.com/golang-migrate/migrate/v4/cmd/migrate@latest
	go install github.com/air-verse/air@latest

.PHONY: generate-migration
generate-migration:
	migrate create -ext sql -dir migration -seq $(name)

.PHONY: migrate
migrate:
	migrate -path migration -database "mysql://lightpub:lightpub@tcp(127.0.0.1:3306)/lightpub" -verbose up

.PHONY: dev-up
dev-up:
	docker compose -f ./docker-compose.dev.yml up -d lightpub_db lightpub_kv lightpub_typesense lightpub_nats

.PHONY: generate-keys
generate-keys:
	./scripts/generate-jwt-keys.sh

.PHONY: dev
dev: generate-keys dev-up migrate
	air

.PHONY: build
build:
	docker compose -f ./docker-compose.inc.yml build

.PHONY: fed-up
fed-up:
	docker compose -f ./docker-compose.fed.yml --profile lightpub --profile misskey up -d --build

.PHONY: fed-down
fed-down:
	docker compose -f ./docker-compose.fed.yml --profile lightpub --profile misskey down

.PHONY: fed-down-volume
fed-down-volume:
	docker compose -f ./docker-compose.fed.yml --profile lightpub --profile misskey down -v
