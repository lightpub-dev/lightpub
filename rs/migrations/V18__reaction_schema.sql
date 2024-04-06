ALTER TABLE
    post_reactions CHANGE reaction_id custom_reaction_id bigint(20) unsigned NULL;

ALTER TABLE
    post_reactions
MODIFY
    COLUMN custom_reaction_id bigint(20) unsigned NULL DEFAULT NULL;

ALTER TABLE
    post_reactions
ADD
    reaction_str varchar(255) DEFAULT NULL NULL;

ALTER TABLE
    post_reactions CHANGE reaction_str reaction_str varchar(255) DEFAULT NULL NULL
AFTER
    post_id;