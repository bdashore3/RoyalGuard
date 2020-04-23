using System;
using System.Threading.Tasks;
using DSharpPlus;
using RoyalGuard.Commands;
using RoyalGuard.Handlers;
using RoyalGuard.Helpers.Commands;
using RoyalGuard.Helpers.Security;
using RoyalGuard.Modules;

namespace RoyalGuard
{
    public class DiscordBot
    {
        // Make the client accessible in all classes
        public static DiscordClient discord;

        // Variables and constructor for DI
        private readonly CommandHandler _commandHandler;
        private readonly NewMemberHandler _newMemberHandler;
        private readonly TrieHandler _trieHandler;
        private readonly PrefixHelper _prefixHelper;
        private readonly Mutes _mutes;

        public DiscordBot(CommandHandler commandHandler, NewMemberHandler newMemberHandler, PrefixHelper prefixHelper, TrieHandler trieHandler, Mutes mutes)
        {
            _commandHandler = commandHandler;
            _newMemberHandler = newMemberHandler;
            _prefixHelper = prefixHelper;
            _trieHandler = trieHandler;
            _mutes = mutes;
        }


        /*
         * Flow:
         * 1. Initialize the Discord Client
         * 2. Wipe the token once set
         * 3. Load all existing muteTimers and the server prefixes
         * 4. Register events and connect
         */
        public async Task Start() 
        {
            discord = new DiscordClient(new DiscordConfiguration
            {
                Token = CredentialsHelper.BotToken,
                TokenType = TokenType.Bot
            });

            CredentialsHelper.WipeToken();
            await _mutes.LoadMuteTimers();
            await _prefixHelper.LoadPrefix();

            // Use a try/catch to log any errors
            discord.MessageCreated += async e =>
            {
                try 
                {
                    if (e.Message.Content.StartsWith(_trieHandler.GetPrefix(e.Channel.GuildId)))
                    {
                        await _commandHandler.HandleCommand(e.Message);                    
                    }
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

            // Authenticate and sign into Discord
            await discord.ConnectAsync();

            Console.WriteLine("The bot is online and ready to work!");
            await Task.Delay(-1); 
        }
    }
}
