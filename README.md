# RoyalGuard

**The bot invite link is currently restricted to certain servers due to beta testing. Documentation is given for self-hosting the bot and a general overview of commands.**

This is a discord bot focused on one thing, Administration. Bots such as MEE6 or Dyno provide an all-in-one experience, but tend to lack on the administrative side of things. I decided to make my own bot that allows server administration as minimal and swift as possible. 

## Feature List
All commands are within `Modules`, but here is a list of the features if you're too lazy:

- Ping: Prints "Pong!". Quick and easy way to see if the bot's online.
- Bans: Used for banning and unbanning a user. Unbans can either be done by Discord user ID or by mention.
- Warnings: Formally warns a user in the server. Is considered the lesser form of a ban. After three warnings, the user is banned from the server and is unbanned at the admin's discretion.
- Mutes: Assigns a bot-created role to the user which doesn't allow typing in text channels or reading message history, but the user can still see the channels. These mutes can be timed if a provided time is given in `w, d, h, m, or s`. The user will be automatically muted and unmuted once the timer expires.
- Custom prefixes: If the server owner has a bot that uses a certain prefix, RoyalGuard can easily use a different prefix for your server.
- A help command that doesn't suck: Typing help gives a list of subcommands. From there, you can get the help per command. If you have any more questions, please join the support server.

## Preparation

### Client

Head to the [Discord developer website](https://discordapp.com/developers) and create a new app. From there, go under the bot menu and create a new bot. Once you create the bot, you should see a token. Put the bot's token in **BotToken** inside info.json.

### Database setup
Follow [this guide](https://www.digitalocean.com/community/tutorials/how-to-install-and-use-postgresql-on-ubuntu-18-04) up until step 3 to get postgres set up on ubuntu. Afterwards, go on pgAdmin4 and follow these steps

 1. Log into a sudo shell and change the postgres user's password by:
	 `passwd postgres`
	 
 2. Add a new server using postgres as the username, and the password that you set for postgres. The IP is your VPS IP or localhost depending on where you're hosting.
 3. Once connected, create a new database and call it whatever you want. You will be using this database name in your ConnectionString and leave the database BLANK.
 
 Your ConnectionString should look like this: `"Host=*Your IP*;Database=*Your DB name*;Username=postgres;Password=*Password you set for postgres user*"`

If you have a connection refused error, follow [this forum post](https://www.digitalocean.com/community/questions/remote-connect-to-postgresql-with-pgadmin) on DigitalOcean:

## Installation

All package hooks ARE included by default. You just need the dotnet runtime, a postgres database, and an EF Core migration set up. Follow the instructions in Preparation to get started with EF Core.

### Setting up the dotnet runtime

To set up the dotnet runtime for ubuntu: [MSDN docs](https://docs.microsoft.com/en-us/dotnet/core/install/linux-package-manager-ubuntu-1804)

Then, copy **info.sample.json** to **info.json** in the project directory. From there, add all of your credentials.

### Entity Framework Core Setup
Once you clone the repository, change into the project directory (KingBot/Kingbot), install the EF Core tools by:
`dotnet tool install --global dotnet-ef`

Then run the following commands:
```
dotnet ef migrations add InitialCreate
dotnet ef database update
```
If you have errors, run `dotnet build` and show them to me in the [discord server](https://discord.gg/pswt7by) if you can't figure out the reason.

This initializes the database for the first time with all the required tables, rows, and columns. If you plan on updating the model, please read the [Entity Framework Core docs](https://docs.microsoft.com/en-us/ef/core/).

### Setting the Default Prefix
The default prefix can be set in `info.json` under the `DefaultPrefix` line. The best practice is to use only one character for the prefix since two or more characters is currently not supported.

### Finally:
Once you're done, type the following command in the terminal inside the project directory (RoyalGuard/RoyalGuard):
```
dotnet build -c Release
dotnet publish -c Release -f netcoreapp3.1 -r linux-x64
dotnet run info.json
```

## Running in a server

The included systemd service is REQUIRED to run this bot in a server. Running in interactive mode is not advised. Copy the twitch.service file into /etc/systemd/system/twitch.service. Then, run these commands
```
sudo systemctl reload-daemon
sudo systemctl enable twitch.service
sudo systemctl start twitch.service
```

Check with:
```
sudo systemctl status twitch.service
sudo journalctl -u twitch -f
```

## Removing the bot

It's easy! All you have to do is delete the bot directory and the systemd file from `/etc/systemd/system/royalguard.service`

# Contributing Modules
The C# version of this bot features modular commands that can be swapped out as needed. To successfully have your module added, you need to follow the guidelines:

1. The module must be commented with a description on what each function does.
2. A module is NOT a wrapper! If you want to make a wrapper for something, use the Other class in modules.
3. You must be familiar with the CommandHandler syntax and link the module with the CommandHandler using a switch case. Nothing goes past it.
4. If you are using the database, modify the EF Core model accordingly and put a comment as to what you did and why you did this.
5. Use Dependency Injection as MUCH as possible. Reference the current modules for an example.

# Developers and Permissions

Currently, this bot is allowed for use outside of the developer's channel since I want people to understand how Twitch bots are coded and how to run them. I try to make the comments as detailed as possible, but if you don't understand something, please contact me via the Discord server! I'm always happy to talk!

Creator/Developer: Brian Dashore

Developer Discord: kingbri#6666

Join the support server here (get the king-updates role to access the channel): [https://discord.gg/pswt7by](https://discord.gg/pswt7by)
