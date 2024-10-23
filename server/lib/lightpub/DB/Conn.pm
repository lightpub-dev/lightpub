package lightpub::DB::Conn;

use DBIx::Connector;

my $dsn = 'dbi:SQLite:dbname=./db/db.sqlite3';
my $conn = DBIx::Connector->new(
  $dsn,
  undef,
  undef,
  {
    RaiseError => 1,
    PrintError => 0,
    AutoCommit => 1,
    sqlite_unicode => 1
  }
);

sub make_handle {
  return $conn->dbh;
}

1;
