DROP TABLE remote_user_details;

ALTER TABLE
    remote_users DROP FOREIGN KEY fk_users_remote_user;

ALTER TABLE
    remote_users
ADD
    CONSTRAINT fk_users_remote_user FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE ON UPDATE CASCADE;