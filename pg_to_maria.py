#!/usr/bin/env python3
import psycopg2
import mysql.connector
import uuid
import argparse
import sys
import logging
from typing import Callable, Dict, List, Tuple, Any, Optional
import json
import os

# Configure logging
logging.basicConfig(
    level=logging.INFO,
    format="%(asctime)s - %(levelname)s - %(message)s",
    handlers=[logging.StreamHandler(sys.stdout)],
)
logger = logging.getLogger(__name__)


# Built-in type converters
def uuid_to_binary16(value: Optional[str]) -> Optional[bytes]:
    """Convert PostgreSQL UUID to MariaDB BINARY(16)."""
    if value is None:
        return None
    return uuid.UUID(value).bytes


def boolean_to_tinyint(value: Optional[bool]) -> Optional[int]:
    """Convert PostgreSQL BOOLEAN to MariaDB TINYINT(1)."""
    if value is None:
        return None
    return 1 if value else 0


def text_to_text(value: Optional[str]) -> Optional[str]:
    """Pass through text values with no conversion."""
    return value


def json_to_text(value: Optional[dict]) -> Optional[str]:
    if value is None:
        return None
    return json.dumps(value)


def int_to_int(value: Optional[int]) -> Optional[int]:
    """Pass through integer values with no conversion."""
    return value


def timestamp_to_datetime(value: Optional[str]) -> Optional[str]:
    """Pass through timestamp values with no conversion."""
    return value


# Dictionary of built-in converters
BUILTIN_CONVERTERS = {
    "uuid_to_binary16": uuid_to_binary16,
    "boolean_to_tinyint": boolean_to_tinyint,
    "text_to_text": text_to_text,
    "int_to_int": int_to_int,
    "timestamp_to_datetime": timestamp_to_datetime,
    "json_to_text": json_to_text,
}


class TableDefinition:
    def __init__(
        self,
        table_name: str,
        column_converters: List[Tuple[str, Optional[str]]],
        primary_key: Optional[str] = None,
    ):
        self.table_name = table_name
        self.column_converters = []

        # Validate and store column converters
        for col_name, converter_name in column_converters:
            # If converter is None, use identity function (no conversion)
            if converter_name is None:
                converter = lambda x: x
            else:
                if converter_name not in BUILTIN_CONVERTERS and not callable(
                    converter_name
                ):
                    raise ValueError(
                        f"Unknown converter '{converter_name}' for column '{col_name}'"
                    )
                converter = BUILTIN_CONVERTERS.get(converter_name, converter_name)

            self.column_converters.append((col_name, converter))

        self.primary_key = primary_key

    @property
    def column_names(self) -> List[str]:
        return [col_name for col_name, _ in self.column_converters]

    def convert_row(self, row: Dict[str, Any]) -> List[Any]:
        """Convert a row from PostgreSQL to MariaDB format."""
        converted = []
        for col_name, converter in self.column_converters:
            try:
                value = row.get(col_name)
                converted.append(converter(value))
            except Exception as e:
                logger.error(f"Error converting column '{col_name}': {e}")
                logger.error(f"Value was: {value}")
                raise
        return converted


class PostgresToMariaDBMigrator:
    def __init__(
        self,
        postgres_config: Dict[str, Any],
        mariadb_config: Dict[str, Any],
        batch_size: int = 1000,
    ):
        self.postgres_config = postgres_config
        self.mariadb_config = mariadb_config
        self.batch_size = batch_size
        self.pg_conn = None
        self.mariadb_conn = None

    def connect(self):
        """Establish connections to both databases."""
        try:
            self.pg_conn = psycopg2.connect(**self.postgres_config)
            logger.info("Connected to PostgreSQL database")
        except Exception as e:
            logger.error(f"Failed to connect to PostgreSQL: {e}")
            raise

        try:
            self.mariadb_conn = mysql.connector.connect(**self.mariadb_config)
            logger.info("Connected to MariaDB database")
        except Exception as e:
            logger.error(f"Failed to connect to MariaDB: {e}")
            if self.pg_conn:
                self.pg_conn.close()
            raise

    def close(self):
        """Close database connections."""
        if self.pg_conn:
            self.pg_conn.close()
            logger.info("PostgreSQL connection closed")

        if self.mariadb_conn:
            self.mariadb_conn.close()
            logger.info("MariaDB connection closed")

    def migrate_table(self, table_def: TableDefinition, where_clause: str = None):
        """Migrate data from a PostgreSQL table to MariaDB."""
        if not self.pg_conn or not self.mariadb_conn:
            raise RuntimeError(
                "Database connections not established. Call connect() first."
            )

        # Count total rows to migrate
        pg_cursor = self.pg_conn.cursor()
        count_query = f'SELECT COUNT(*) FROM "{table_def.table_name}"'
        if where_clause:
            count_query += f" WHERE {where_clause}"

        pg_cursor.execute(count_query)
        total_rows = pg_cursor.fetchone()[0]
        pg_cursor.close()

        logger.info(
            f"Starting migration of table '{table_def.table_name}' ({total_rows} rows)"
        )

        # Prepare column list with proper quoting
        pg_columns_str = ", ".join(
            [f'"{col_name}"' for col_name in table_def.column_names]
        )
        mariadb_columns_str = ", ".join(
            [f"`{col_name}`" for col_name in table_def.column_names]
        )

        # Prepare PostgreSQL query
        pg_query = f'SELECT {pg_columns_str} FROM "{table_def.table_name}"'
        if where_clause:
            pg_query += f" WHERE {where_clause}"

        # Prepare MariaDB query
        placeholders = ", ".join(["%s"] * len(table_def.column_names))
        mariadb_query = f"INSERT INTO `{table_def.table_name}` ({mariadb_columns_str}) VALUES ({placeholders})"

        # Execute migration in batches
        pg_cursor = self.pg_conn.cursor(name="fetch_large_result")
        pg_cursor.itersize = self.batch_size
        pg_cursor.execute(pg_query)

        mariadb_cursor = self.mariadb_conn.cursor()

        rows_migrated = 0
        batch_data = []

        for pg_row in pg_cursor:
            # Convert row to dictionary for easier column access
            row_dict = dict(zip(table_def.column_names, pg_row))

            # Convert data types
            converted_row = table_def.convert_row(row_dict)
            batch_data.append(converted_row)

            # Insert batch when it reaches batch_size
            if len(batch_data) >= self.batch_size:
                try:
                    mariadb_cursor.executemany(mariadb_query, batch_data)
                    rows_migrated += len(batch_data)
                    logger.info(
                        f"Processed {rows_migrated}/{total_rows} rows from '{table_def.table_name}'"
                    )
                    batch_data = []
                except Exception as e:
                    logger.error(f"Error inserting batch: {e}")
                    logger.error(
                        f"First row in batch: {batch_data[0] if batch_data else None}"
                    )
                    raise

        # Insert any remaining rows
        if batch_data:
            mariadb_cursor.executemany(mariadb_query, batch_data)
            rows_migrated += len(batch_data)

        pg_cursor.close()
        mariadb_cursor.close()

        logger.info(
            f"Migration of table '{table_def.table_name}' completed. {rows_migrated} rows migrated."
        )
        return rows_migrated

    def migrate_tables(
        self,
        table_definitions: List[TableDefinition],
        where_clauses: Dict[str, str] = None,
    ):
        """Migrate multiple tables in a single transaction."""
        where_clauses = where_clauses or {}
        results = {}

        # Disable foreign key checks
        with self.mariadb_conn.cursor() as cursor:
            cursor.execute("SET FOREIGN_KEY_CHECKS=0")

        try:
            # Begin transaction
            self.mariadb_conn.start_transaction()

            for table_def in table_definitions:
                where_clause = where_clauses.get(table_def.table_name)
                try:
                    rows_migrated = self.migrate_table(table_def, where_clause)
                    results[table_def.table_name] = {
                        "status": "success",
                        "rows_migrated": rows_migrated,
                    }
                except Exception as e:
                    logger.error(f"Error migrating table '{table_def.table_name}': {e}")
                    results[table_def.table_name] = {"status": "error", "error": str(e)}
                    raise

            # Commit the transaction
            self.mariadb_conn.commit()
            logger.info("All tables migrated successfully. Transaction committed.")

        except Exception as e:
            # Rollback on error
            self.mariadb_conn.rollback()
            logger.error(f"Transaction rolled back due to error: {e}")
        finally:
            # Re-enable foreign key checks
            with self.mariadb_conn.cursor() as cursor:
                cursor.execute("SET FOREIGN_KEY_CHECKS=1")

        return results


def load_table_definitions(config_file: str) -> List[TableDefinition]:
    """Load table definitions from a JSON config file."""
    with open(config_file, "r") as f:
        config = json.load(f)

    table_defs = []
    for table_config in config["tables"]:
        table_name = table_config["name"]

        # Handle converter which may be None (no conversion)
        column_converters = [
            (col["name"], col.get("converter")) for col in table_config["columns"]
        ]
        primary_key = table_config.get("primary_key")

        table_defs.append(
            TableDefinition(
                table_name=table_name,
                column_converters=column_converters,
                primary_key=primary_key,
            )
        )

    return table_defs


def main():
    parser = argparse.ArgumentParser(
        description="Migrate data from PostgreSQL to MariaDB"
    )
    parser.add_argument("--config", required=True, help="Path to configuration file")
    parser.add_argument(
        "--batch-size", type=int, default=1000, help="Batch size for inserts"
    )
    args = parser.parse_args()

    # Load configuration
    try:
        with open(args.config, "r") as f:
            config = json.load(f)
    except Exception as e:
        logger.error(f"Error loading config file: {e}")
        sys.exit(1)

    # Extract database configurations
    postgres_config = config["postgres"]
    mariadb_config = config["mariadb"]

    # Load table definitions
    table_definitions = load_table_definitions(args.config)

    # Extract where clauses if specified
    where_clauses = config.get("where_clauses", {})

    # Create and run migrator
    migrator = PostgresToMariaDBMigrator(
        postgres_config=postgres_config,
        mariadb_config=mariadb_config,
        batch_size=args.batch_size,
    )

    try:
        migrator.connect()
        results = migrator.migrate_tables(table_definitions, where_clauses)

        # Output summary
        print("\nMigration Summary:")
        print("==================")
        success_count = sum(1 for r in results.values() if r["status"] == "success")
        error_count = sum(1 for r in results.values() if r["status"] == "error")
        total_rows = sum(r.get("rows_migrated", 0) for r in results.values())

        print(f"Tables processed: {len(results)}")
        print(f"Successful: {success_count}")
        print(f"Failed: {error_count}")
        print(f"Total rows migrated: {total_rows}")

        if error_count > 0:
            print("\nFailed tables:")
            for table, result in results.items():
                if result["status"] == "error":
                    print(f"  - {table}: {result['error']}")

    finally:
        migrator.close()


if __name__ == "__main__":
    main()
