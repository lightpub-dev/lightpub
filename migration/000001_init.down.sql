-- Drop tables with dependencies on note and/or user
DROP TABLE IF EXISTS `user_follow`;

DROP TABLE IF EXISTS `user_block`;

DROP TABLE IF EXISTS `remote_public_key`;

DROP TABLE IF EXISTS `push_notification`;

DROP TABLE IF EXISTS `notification`;

DROP TABLE IF EXISTS `note_upload`;

DROP TABLE IF EXISTS `note_tag`;

DROP TABLE IF EXISTS `note_mention`;

DROP TABLE IF EXISTS `note_like`;

-- Drop note table
DROP TABLE IF EXISTS `note`;

-- Drop user table
DROP TABLE IF EXISTS `user`;

-- Drop tables with no dependencies
DROP TABLE IF EXISTS `upload`;

DROP TABLE IF EXISTS `tag`;

DROP TABLE IF EXISTS `apub_error_report`;