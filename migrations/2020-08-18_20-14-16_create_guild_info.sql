-- Add migration script here
CREATE TABLE public.guild_info
(
    guild_id bigint NOT NULL,
    prefix text,
    PRIMARY KEY (guild_id)
);

ALTER TABLE public.guild_info
    OWNER to postgres;