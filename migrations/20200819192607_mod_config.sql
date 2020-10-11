-- Add migration script here
ALTER TABLE public.guild_info
    ADD COLUMN mod_role_id bigint;
