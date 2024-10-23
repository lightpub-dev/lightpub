use Mojo::Base -strict;

use Test::More;
use Test::Mojo;

use lightpub::Service::Account;

my $sample_user = {
  username => 'admin',
  password => '1234Abcd!',
  nickname => 'admin dayo'
};

my $t = Test::Mojo->new('lightpub');
$t->post_ok('/register' => json => $sample_user)->status_is(200)->json_is('/status' => 'ok');

done_testing();
