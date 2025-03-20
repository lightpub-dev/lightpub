-- Rename tables from singular to plural for GORM convention
RENAME TABLE `apub_error_report` TO `apub_error_reports`;

RENAME TABLE `note` TO `notes`;

RENAME TABLE `note_like` TO `note_likes`;

RENAME TABLE `note_mention` TO `note_mentions`;

RENAME TABLE `note_tag` TO `note_tags`;

RENAME TABLE `note_upload` TO `note_uploads`;

RENAME TABLE `notification` TO `notifications`;

RENAME TABLE `push_notification` TO `push_notifications`;

RENAME TABLE `remote_public_key` TO `remote_public_keys`;

RENAME TABLE `tag` TO `tags`;

RENAME TABLE `upload` TO `uploads`;

RENAME TABLE `user` TO `users`;

RENAME TABLE `user_block` TO `user_blocks`;

RENAME TABLE `user_follow` TO `user_follows`;