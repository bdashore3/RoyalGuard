using System;
using System.Threading.Tasks;
using DSharpPlus;
using DSharpPlus.Entities;
using DSharpPlus.EventArgs;
using RoyalGuard.Commands;
using RoyalGuard.Handlers;
using RoyalGuard.Helpers.Commands;
using RoyalGuard.Helpers.Data;
using RoyalGuard.Helpers.Security;
using RoyalGuard.Modules;

namespace RoyalGuard
{
    public class DiscordBot
    {
        // Make the client accessible in all classes
        public static DiscordShardedClient discord;

        // Variables and constructor for DI
        private readonly CommandHandler _commandHandler;
        private readonly NewMemberHandler _newMemberHandler;
        private readonly TrieHandler _trieHandler;
        private readonly PrefixHelper _prefixHelper;
        private readonly Mutes _mutes;
        private readonly PermissionsHandler _permissionsHandler;
        private readonly GuildInfoHelper _guildInfoHelper;

        public DiscordBot(
            CommandHandler commandHandler, 
            NewMemberHandler newMemberHandler, 
            PrefixHelper prefixHelper, 
            TrieHandler trieHandler, 
            Mutes mutes, 
            PermissionsHandler permissionsHandler,
            GuildInfoHelper guildInfoHelper)
        {
            _commandHandler = commandHandler;
            _newMemberHandler = newMemberHandler;
            _prefixHelper = prefixHelper;
            _trieHandler = trieHandler;
            _mutes = mutes;
            _permissionsHandler = permissionsHandler;
            _guildInfoHelper = guildInfoHelper;
        }


        /*
         * Flow:
         * 1. Initialize the Discord Client
         * 2. Wipe the token once set
         * 3. Load all existing muteTimers and the server prefixes
         * 4. Register events and connect
         */
        public async Task Start(string credsPath) 
        {
            CredentialsHelper securityHelper = new CredentialsHelper();

            discord = new DiscordShardedClient(new DiscordConfiguration
            {
                Token = securityHelper.ReadCreds(credsPath).BotToken,
                TokenType = TokenType.Bot
            });

            securityHelper.SetStatics(credsPath);

            await _mutes.LoadMuteTimers();
            await _prefixHelper.LoadPrefix();

            // Use a try/catch to log any errors
            discord.MessageCreated += async e =>
            {
                try 
                {
                    if (e.Author.IsBot)
                        return;

                    int prefix_length = _trieHandler.GetPrefix(e.Channel.GuildId).Length;

                    if (e.Message.Content.Length < prefix_length)
                        return;

                    if (e.Message.Content.Substring(0, prefix_length)
                            .Equals(_trieHandler.GetPrefix(e.Channel.GuildId)))
                    {
                        await _commandHandler.HandleCommand(e.Message);
                        return;
                    }

                    if (e.MentionedUsers.Count < 1)
                        return;

                    else if (e.MentionedUsers[0].Id.Equals(CredentialsHelper.BotId) && 
                            _permissionsHandler.CheckPermission(e.Message, Permissions.ManageMessages))
                    {
                        await _commandHandler.HandleEmergency(e.Message);
                        return;
                    }
                }

                catch (DSharpPlus.Exceptions.UnauthorizedException)
                {
                    await e.Message.RespondAsync("I don't have the proper role position to execute this command! \n" +
                                                "Please put my role below all admin/mod roles and above all user roles!");
                }

                catch (Exception ex)
                {
                    Console.WriteLine(ex);
                }
            };

            discord.GuildMemberAdded += async e =>
            {
                try
                {
                    await _newMemberHandler.OnNewMemberEvent(e.Guild, e.Member, "welcome");
                }
                catch (Exception ex)
                {
                    Console.WriteLine(ex);
                }
            };

            discord.GuildMemberRemoved += async e =>
            {
                try
                {
                    await _newMemberHandler.OnNewMemberEvent(e.Guild, e.Member, "leave");
                }
                catch (Exception ex)
                {
                    Console.WriteLine(ex);
                }
            };

            discord.GuildCreated += async e =>
            {
                try
                {
                    await _guildInfoHelper.NewGuildAdded(e.Guild.Id);
                }
                catch (Exception ex)
                {
                    Console.WriteLine(ex);
                }
            };

            discord.GuildDeleted += async e =>
            {
                try
                {
                    await _guildInfoHelper.FlagForRemoval(e.Guild.Id);
                }
                catch (Exception ex)
                {
                    Console.WriteLine(ex);
                }
            };

            discord.Ready += onDiscordReady;

            // Authenticate and sign into Discord
            await discord.StartAsync();

            Console.WriteLine("The bot is online and ready to work!");
            await Task.Delay(-1); 
        }

        private async Task onDiscordReady(ReadyEventArgs e)
        {
            await discord.UpdateStatusAsync(new DiscordActivity("the castle", ActivityType.Watching), UserStatus.Online, null);
        }

        // Stop call for the bot HostedService
        public async Task Stop()
        {
            Console.WriteLine("Disconnecting!");
            foreach (var i in discord.ShardClients)
            {
                await i.Value.DisconnectAsync();
            }
        }
    }
}
