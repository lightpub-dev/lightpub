ALTER TABLE
    lightpub.user_tokens DROP FOREIGN KEY fk_users_user_tokens;

ALTER TABLE
    lightpub.user_tokens
ADD
    CONSTRAINT fk_users_user_tokens FOREIGN KEY (user_id) REFERENCES lightpub.users(id) ON DELETE RESTRICT ON UPDATE RESTRICT;