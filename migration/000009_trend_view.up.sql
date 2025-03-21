CREATE
OR REPLACE VIEW `trending_tags` AS
select
    `nt`.`tag_id` AS `tag_id`,
    `t`.`name` AS `name`,
    count(`n`.`id`) AS `note_count`
from
    (
        (
            `note_tags` `nt`
            join `notes` `n` on (`n`.`id` = `nt`.`note_id`)
        )
        join `tags` `t` on (`t`.`id` = `nt`.`tag_id`)
    )
where
    `n`.`created_at` >= current_timestamp() - interval 1 day
    and `n`.`visibility` in ('public', 'unlisted')
group by
    `nt`.`tag_id`,
    `t`.`name`
order by
    count(`n`.`id`) desc;