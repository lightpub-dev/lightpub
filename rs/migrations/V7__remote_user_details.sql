-- lightpub.remote_user_details definition
CREATE TABLE `remote_user_details` (
    `id` char(32) NOT NULL,
    `following_uri` varchar(512) DEFAULT NULL,
    `followers_uri` varchar(512) DEFAULT NULL,
    `liked_uri` varchar(512) DEFAULT NULL,
    `fetched_at` datetime NOT NULL DEFAULT current_timestamp(),
    PRIMARY KEY (`id`)
);

ALTER TABLE
    remote_user_details
ADD
    CONSTRAINT remote_user_details_users_FK FOREIGN KEY (id) REFERENCES users(id) ON DELETE CASCADE ON UPDATE CASCADE;