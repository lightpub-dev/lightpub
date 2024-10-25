#!/bin/bash
MYSQL_HOST=lightpub_db
MYSQL_USER=root
MYSQL_PASS=lightpub
MYSQL_DATABASE=lightpub
mysql -h $MYSQL_HOST -u $MYSQL_USER -p$MYSQL_PASS -Nse 'show tables' $MYSQL_DATABASE | while read table; do echo "truncating $table"; mysql -h $MYSQL_HOST -u $MYSQL_USER -p$MYSQL_PASS -e "set FOREIGN_KEY_CHECKS = 0; truncate table $table;" $MYSQL_DATABASE; done
