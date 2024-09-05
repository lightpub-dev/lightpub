CREATE TABLE `posts` (
	`id` text PRIMARY KEY NOT NULL,
	`url` text(512),
	`authorId` text NOT NULL,
	`privacy` text,
	`replyToId` text,
	`repostOfId` text,
	`createdAt` integer NOT NULL,
	`deletedAt` integer,
	FOREIGN KEY (`authorId`) REFERENCES `users`(`id`) ON UPDATE no action ON DELETE no action,
	FOREIGN KEY (`replyToId`) REFERENCES `posts`(`id`) ON UPDATE no action ON DELETE no action,
	FOREIGN KEY (`repostOfId`) REFERENCES `posts`(`id`) ON UPDATE no action ON DELETE no action
);
--> statement-breakpoint
CREATE TABLE `secrets` (
	`key` text PRIMARY KEY NOT NULL,
	`value` text NOT NULL
);
--> statement-breakpoint
CREATE TABLE `userFollows` (
	`id` integer PRIMARY KEY NOT NULL,
	`followerId` text NOT NULL,
	`followeeId` text NOT NULL,
	`followAt` integer NOT NULL,
	FOREIGN KEY (`followerId`) REFERENCES `users`(`id`) ON UPDATE no action ON DELETE no action,
	FOREIGN KEY (`followeeId`) REFERENCES `users`(`id`) ON UPDATE no action ON DELETE no action
);
--> statement-breakpoint
CREATE TABLE `users` (
	`id` text PRIMARY KEY NOT NULL,
	`username` text(128) NOT NULL,
	`hostname` text(512),
	`password` text(256),
	`nickname` text(128) NOT NULL,
	`bio` text(2048) DEFAULT '' NOT NULL,
	`url` text(512),
	`privateKey` text(1024),
	`publicKey` text(1024),
	`createdAt` integer NOT NULL,
	`deletedAt` integer
);
