.PHONY: install-deps
install-deps:
	go install -tags 'mysql' github.com/golang-migrate/migrate/v4/cmd/migrate@latest

.PHONY: generate-migration
generate-migration:
	migrate create -ext sql -dir migration -seq $(name)

.PHONY: migrate
migrate:
	migrate -path migration -database "mysql://lightpub:lightpub@tcp(127.0.0.1:3306)/lightpub" -verbose up

.PHONY: dev-up
dev-up:
	docker compose -f ./docker-compose.dev.yml up -d lightpub_db lightpub_kv lightpub_typesense lightpub_nats
