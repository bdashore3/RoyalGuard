# RoyalGuard

**Invite Link**: [https://discord.com/oauth2/authorize?client_id=698554777981681754&permissions=268827894&scope=bot](https://discord.com/oauth2/authorize?client_id=698554777981681754&permissions=268827894&scope=bot)

**Support Server**: Since top.gg is re-adding most bots (due to discord limitations), here's the support server link if you have an error/weird issue.
[https://discord.gg/pswt7by](https://discord.gg/pswt7by)

This is a discord bot focused on one thing, Administration. Bots such as MEE6 or Dyno provide an all-in-one experience, but tend to lack on the administrative side of things. I decided to make my own bot that allows server administration as minimal and swift as possible.

## Feature List

All commands are within `Modules`, but here is a list of the features if you're too lazy:

-   Ping: Prints "Pong!". Quick and easy way to see if the bot's online.
-   Bans: Used for banning and unbanning a user. Unbans can either be done by Discord user ID or by mention.
-   Warnings: Formally warns a user in the server. Is considered the lesser form of a ban. After three warnings, the user is banned from the server and is unbanned at the admin's discretion.
-   Mutes: Assigns a bot-created role to the user which doesn't allow typing in text channels or reading message history, but the user can still see the channels. These mutes can be timed if a provided time is given in `w, d, h, m, or s`. The user will be automatically muted and unmuted once the timer expires.
-   Purging: Removes up to 100 messages (under 2 weeks old) when given a message ID to start from or the amount of messages to delete before the command.
-   Custom prefixes: If the server owner has a bot that uses a certain prefix, RoyalGuard can easily use a different prefix for your server.
-   Data Recovery: If the server owner accidentally kicks the bot, your data isn't gone! It stays in the database for a week since the kick and clears if you re-add the bot!
-   Emergency Mention: If the server owner makes a bot-conflicting prefix, the bot can be mentioned to get the current prefix, to reset the prefix, or to change the prefix to something else.
-   Reaction roles: Configurable reaction roles that are either accomplished in one command or a setup wizard. If the admin deletes the message/removes all reactions, the roles
    are also gone.
-   Automatic welcome/leave messages: Allows server owners to set welcome and leave messages on server join events.
-   Automatic role assignment: Server owners can automatically assign roles when a new user joins.
-   Member info screen: Shows the details of a member in a server for easy information lookup.
-   A help command that doesn't suck: Typing help gives a list of subcommands. From there, you can get the help per command. If you have any more questions, please join the support server.

### Planned Features

Here are some of the planned features for later releases:

-   Adaptive purging: Manually purge messages if they aren't able to be bulk deleted.
-   Pretty audit logging: Configure a channel where the server can see who edits what, what roles are deleted, etc.

## Preparation

### Client

Head to the [Discord developer website](https://discordapp.com/developers) and create a new app. From there, go under the bot menu and create a new bot. Once you create the bot, you should see a token. Put the bot's token in **BotToken** and the application client ID in **BotIdString** inside info.json.

### Database setup

Follow [this guide](https://www.digitalocean.com/community/tutorials/how-to-install-and-use-postgresql-on-ubuntu-20-04) up until step 3 to get postgres set up on ubuntu. Afterwards, go on pgAdmin4 and follow these steps

1.  Log into a sudo shell and change the postgres user's password by:
    `passwd postgres`
2.  Add a new server using postgres as the username, and the password that you set for postgres. The IP is your VPS IP or localhost depending on where you're hosting.
3.  Once connected, create a new database and call it whatever you want. You will be using this database name in your ConnectionString and leave the database BLANK.

Your connection URL should look like this: `postgres://postgres:{password}@{IP}:5432/{Db Name}"`

If you have a connection refused error, follow [this forum post](https://www.digitalocean.com/community/questions/remote-connect-to-postgresql-with-pgadmin) on DigitalOcean

## Installation

### Downloading the bot

Download the latest binary from the [releases](https://github.com/bdashore3/RoyalGuard/releases) and use FTP or SCP to push the file to your server! (You can also use
wget/curl to directly download the binary to the server itself).

It is HIGHLY recommended to rename the downloaded binary to `royalguard` for startup's sake.

### Configuration

Copy `info_sample.json` to `info.json` in the project directory. From there, add the following credentials:

```
- bot_token
- default_prefix
- db_connection (In URL form. Fill in the {} fields)
```

### Finally:

Once you're done, type the following command in the terminal inside the binary directory:

```
./royalguard info.json
```

## Running in a server

The included systemd service is HIGHLY RECOMMENDED to run this bot in a server. Running in interactive mode is not advised. Copy the royalguard.service file into /etc/systemd/system/royalguard.service. Then, run these commands

```
sudo systemctl reload-daemon
sudo systemctl enable royalguard.service
sudo systemctl start royalguard.service
```

Check with:

```
sudo systemctl status royalguard.service
sudo journalctl -u royalguard -f
```

## Removing the bot

It's easy! All you have to do is delete the bot directory and the systemd file from `/etc/systemd/system/royalguard.service`

# Contributing Modules

The Rust version of this bot features commands that can be swapped out as needed. To successfully have your command added, you need to follow the guidelines:

1. The module must be commented with a description on what each function does.
2. A module is NOT a wrapper! If you want to make a wrapper for something, use the general file in commands.
3. You must be familiar with the Serenity framework and link the command in the commands file within structures.
4. If you are using the database, modify the SQLx migrations accordingly and put a comment as to what you did and why you did this.

# Developers and Permissions

Currently, this bot is allowed for use outside of the developer's server. I try to make the comments as detailed as possible, but if you don't understand something, please contact me via the Discord server! I'm always happy to talk!

Creator/Developer: Brian Dashore

Developer Discord: kingbri#6666

Join the support server here (get the king-updates role to access the channel): [https://discord.gg/pswt7by](https://discord.gg/pswt7by)
