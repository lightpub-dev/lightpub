CREATE TABLE `posts` (
	`id` char(32) NOT NULL,
	`url` varchar(512),
	`authorId` char(32) NOT NULL,
	`privacy` enum('public','unlisted','follower','private') NOT NULL,
	`replyToId` char(32),
	`repostOfId` char(32),
	`createdAt` datetime(6) NOT NULL,
	`deletedAt` datetime(6),
	CONSTRAINT `posts_id` PRIMARY KEY(`id`)
);
--> statement-breakpoint
CREATE TABLE `secrets` (
	`key` varchar(64) NOT NULL,
	`value` text NOT NULL,
	CONSTRAINT `secrets_key` PRIMARY KEY(`key`)
);
--> statement-breakpoint
CREATE TABLE `userFollows` (
	`id` bigint AUTO_INCREMENT NOT NULL,
	`followerId` char(32) NOT NULL,
	`followeeId` char(32) NOT NULL,
	`followAt` datetime(6) NOT NULL,
	CONSTRAINT `userFollows_id` PRIMARY KEY(`id`),
	CONSTRAINT `userFollows_followerId_followeeId_unique` UNIQUE(`followerId`,`followeeId`)
);
--> statement-breakpoint
CREATE TABLE `users` (
	`id` char(32) NOT NULL,
	`username` varchar(64) NOT NULL,
	`hostname` varchar(128),
	`password` varchar(60),
	`nickname` varchar(255) NOT NULL,
	`bio` text NOT NULL DEFAULT (''),
	`url` varchar(512),
	`privateKey` varchar(1024),
	`publicKey` varchar(1024),
	`createdAt` datetime(6) NOT NULL,
	`deletedAt` datetime(6),
	CONSTRAINT `users_id` PRIMARY KEY(`id`),
	CONSTRAINT `users_username_hostname_unique` UNIQUE(`username`,`hostname`)
);
--> statement-breakpoint
ALTER TABLE `posts` ADD CONSTRAINT `posts_authorId_users_id_fk` FOREIGN KEY (`authorId`) REFERENCES `users`(`id`) ON DELETE no action ON UPDATE no action;--> statement-breakpoint
ALTER TABLE `posts` ADD CONSTRAINT `posts_replyToId_posts_id_fk` FOREIGN KEY (`replyToId`) REFERENCES `posts`(`id`) ON DELETE no action ON UPDATE no action;--> statement-breakpoint
ALTER TABLE `posts` ADD CONSTRAINT `posts_repostOfId_posts_id_fk` FOREIGN KEY (`repostOfId`) REFERENCES `posts`(`id`) ON DELETE no action ON UPDATE no action;--> statement-breakpoint
ALTER TABLE `userFollows` ADD CONSTRAINT `userFollows_followerId_users_id_fk` FOREIGN KEY (`followerId`) REFERENCES `users`(`id`) ON DELETE no action ON UPDATE no action;--> statement-breakpoint
ALTER TABLE `userFollows` ADD CONSTRAINT `userFollows_followeeId_users_id_fk` FOREIGN KEY (`followeeId`) REFERENCES `users`(`id`) ON DELETE no action ON UPDATE no action;