package lightpub::Controller::Account;
use feature 'signatures';

use CloudDiary::DB::Conn;

my $handle = CloudDiary::DB::Conn::make_handle();

# This action will render a template
sub register ($self, ) {

  # Render template "example/welcome.html.ep" with message
  $self->render(msg => 'Welcome to the Mojolicious real-time web framework!');
}

1;
