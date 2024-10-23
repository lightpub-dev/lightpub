package lightpub::Controller::Account;
use Mojo::Base 'Mojolicious::Controller', -signatures;

# This action will render a template
sub register ($self) {

  # Render template "example/welcome.html.ep" with message
  $self->render(msg => 'Welcome to the Mojolicious real-time web framework!');
}

1;
