-- Rename tables from plural to singular, reversing the previous migration
RENAME TABLE `apub_error_reports` TO `apub_error_report`;

RENAME TABLE `notes` TO `note`;

RENAME TABLE `note_likes` TO `note_like`;

RENAME TABLE `note_mentions` TO `note_mention`;

RENAME TABLE `note_tags` TO `note_tag`;

RENAME TABLE `note_uploads` TO `note_upload`;

RENAME TABLE `notifications` TO `notification`;

RENAME TABLE `push_notifications` TO `push_notification`;

RENAME TABLE `remote_public_keys` TO `remote_public_key`;

RENAME TABLE `tags` TO `tag`;

RENAME TABLE `uploads` TO `upload`;

RENAME TABLE `users` TO `user`;

RENAME TABLE `user_blocks` TO `user_block`;

RENAME TABLE `user_follows` TO `user_follow`;