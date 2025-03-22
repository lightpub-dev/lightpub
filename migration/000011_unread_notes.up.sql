CREATE
OR REPLACE VIEW `unread_notification_counts` AS
select
    `u`.`id` AS `user_id`,
    coalesce(`n`.`unread_count`, 0) AS `unread_count`
from
    (
        `users` `u`
        left join (
            select
                `notifications`.`user_id` AS `user_id`,
                count(0) AS `unread_count`
            from
                `notifications`
            where
                `notifications`.`read_at` is null
            group by
                `notifications`.`user_id`
        ) `n` on (`u`.`id` = `n`.`user_id`)
    );