package lightpub::Service::Account;
use v5.30;
use feature 'signatures';

use lightpub::DB::Conn;

my $handle = lightpub::DB::Conn::make_handle();

sub new ($class) {
  return bless {}, $class;
}

sub username_ok ($self, $username) {
  if ($username !~ m/^[a-zA-Z](?:[a-zA-Z0-9\_\-]{2,})$/) {
    return 0;
  }

  my $sth = $handle->prepare('SELECT COUNT(*) AS count FROM users WHERE username = ?');
  $sth->execute($username);
  my $count = $sth->fetchrow_hashref()->{count};
  return $count == 0;
}

# This action will render a template
sub register ($self, $username, $password, $nickname) {

  # Render template "example/welcome.html.ep" with message
  $self->render(msg => 'Welcome to the Mojolicious real-time web framework!');
}

1;
