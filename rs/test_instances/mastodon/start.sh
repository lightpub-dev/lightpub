#!/bin/sh
bundle exec rake db:migrate && bundle exec rails s -p 3000
