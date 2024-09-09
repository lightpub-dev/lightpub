CREATE TABLE `postBookmarks` (
	`id` serial PRIMARY KEY,
	`userId` char(32) NOT NULL,
	`postId` char(32) NOT NULL,
	`bookmarkedAt` datetime(6) NOT NULL
);
--> statement-breakpoint
CREATE TABLE `postFavorites` (
	`id` serial PRIMARY KEY,
	`userId` char(32) NOT NULL,
	`postId` char(32) NOT NULL,
	`favoritedAt` datetime(6) NOT NULL
);
--> statement-breakpoint
CREATE TABLE `postReactions` (
	`id` serial PRIMARY KEY,
	`userId` char(32) NOT NULL,
	`postId` char(32) NOT NULL,
	`reaction` varchar(64) NOT NULL,
	`reactedAt` datetime(6) NOT NULL
);
--> statement-breakpoint
ALTER TABLE `postBookmarks` ADD CONSTRAINT `postBookmarks_userId_users_id_fk` FOREIGN KEY (`userId`) REFERENCES `users`(`id`) ON DELETE no action ON UPDATE no action;--> statement-breakpoint
ALTER TABLE `postBookmarks` ADD CONSTRAINT `postBookmarks_postId_posts_id_fk` FOREIGN KEY (`postId`) REFERENCES `posts`(`id`) ON DELETE no action ON UPDATE no action;--> statement-breakpoint
ALTER TABLE `postFavorites` ADD CONSTRAINT `postFavorites_userId_users_id_fk` FOREIGN KEY (`userId`) REFERENCES `users`(`id`) ON DELETE no action ON UPDATE no action;--> statement-breakpoint
ALTER TABLE `postFavorites` ADD CONSTRAINT `postFavorites_postId_posts_id_fk` FOREIGN KEY (`postId`) REFERENCES `posts`(`id`) ON DELETE no action ON UPDATE no action;--> statement-breakpoint
ALTER TABLE `postReactions` ADD CONSTRAINT `postReactions_userId_users_id_fk` FOREIGN KEY (`userId`) REFERENCES `users`(`id`) ON DELETE no action ON UPDATE no action;--> statement-breakpoint
ALTER TABLE `postReactions` ADD CONSTRAINT `postReactions_postId_posts_id_fk` FOREIGN KEY (`postId`) REFERENCES `posts`(`id`) ON DELETE no action ON UPDATE no action;