use v5.30;
use Test::More;
use lightpub::Service::Account;

my $s = lightpub::Service::Account->new();
is($s->username_ok('admin'), 1);
is($s->username_ok('admin dayo'), 0);
is($s->username_ok('1234root'), 0);
is($s->username_ok('_hacker'), 0);
is($s->username_ok('admin!'), 0);
is($s->username_ok('user1'), 1);

done_testing;
