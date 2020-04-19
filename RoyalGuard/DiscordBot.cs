using System;
using System.Threading.Tasks;
using DSharpPlus;
using RoyalGuard.Commands;
using RoyalGuard.Handlers;
using RoyalGuard.Helpers.Commands;
using RoyalGuard.Helpers.Security;

namespace RoyalGuard
{
    public class DiscordBot
    {
        public static DiscordShardedClient discord;
        private readonly CommandHandler _commandHandler;
        private readonly NewMemberHandler _newMemberHandler;
        private readonly PrefixHelper _prefixHelper;

        public DiscordBot(CommandHandler commandHandler, NewMemberHandler newMemberHandler, PrefixHelper prefixHelper)
        {
            _commandHandler = commandHandler;
            _newMemberHandler = newMemberHandler;
            _prefixHelper = prefixHelper;
        }

        public async Task Start() 
        {
            discord = new DiscordShardedClient(new DiscordConfiguration
            {
                Token = CredentialsHelper.BotToken,
                TokenType = TokenType.Bot
            });

            CredentialsHelper.WipeToken();

            discord.MessageCreated += async e =>
            {
                try 
                {
                    if (e.Message.Content.StartsWith(await _prefixHelper.FetchPrefix(e.Message.Channel.GuildId)))
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

            await discord.StartAsync();

            Console.WriteLine("The bot is online and ready to work!");
            await Task.Delay(-1); 
        }
    }
}
