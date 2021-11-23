-- Add migration script here
create table logging
(
    guild_id              bigint not null
        constraint logging_pk
            primary key
        constraint "FK_logging_guild_info_guild_id"
            references guild_info
            on delete cascade,
    message_channel_id    bigint,
    server_channel_id     bigint,
    mod_action_channel_id bigint
);

create unique index logging_guild_id_uindex
    on logging (guild_id);

ALTER TABLE public.logging
    OWNER to postgres;
