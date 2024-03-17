#!/bin/bash
create_account() {
  echo "Creating account"
  tootctl accounts create massuser --email massuser@mastodon.tinax.local --confirmed
}

bundle exec rake db:migrate && create_account; bundle exec rails s -p 3000
