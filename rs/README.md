# Lightpub Rust backend

## Development

### Prerequisites
You need to have a database running to compile or run the codes because SQL statements are checked at compile time.
Issue the following commands to start and configure a development mariadb server:
```bash
docker compose -f docker-compose.fed.yml --profile lightpub-dev up -d # start the database
refinery migrate # run the migrations
```

### Running the server
To start the API server for development, execute `run_dev_server.sh`

### Run the tests
There are two types of tests in this project:
- `run_tests.sh`: This script tests Lightpub API endpoints.
- `run_federation_test.sh`: This script tests the federation between Lightpub and other fediverse software.

## API Documentation

OpenAPI specification is defined at `openapi.yaml`.

## Used software
- [Actix web](https://actix.rs/): Web framework.
- [Refinery](https://github.com/rust-db/refinery): Database migration tool written in Rust.
- [MariaDB](https://mariadb.org/): SQL database.
- [LavinMQ](https://lavinmq.com/): AMQP broker. Used to execute background tasks asynchronously.
- [Memcached](https://memcached.org/): In-memory key-value store. Used for caching.
