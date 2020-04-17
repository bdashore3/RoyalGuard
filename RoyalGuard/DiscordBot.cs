using System;
using System.Threading.Tasks;
using DSharpPlus;
using DSharpPlus.Entities;
using RoyalGuard.Commands;
using RoyalGuard.Handlers;
using RoyalGuard.Helpers.Security;

namespace RoyalGuard
{
    public class DiscordBot
    {
        public static DiscordShardedClient discord;
        private readonly CommandHandler _commandHandler;
        private readonly NewMemberHandler _newMemberHandler;

        public DiscordBot(CommandHandler commandHandler, NewMemberHandler newMemberHandler)
        {
            _commandHandler = commandHandler;
            _newMemberHandler = newMemberHandler;
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
                if (e.Message.Content.StartsWith(CredentialsHelper.Prefix))
                {
                    try 
                    {
                        await _commandHandler.HandleCommand(e.Message);
                    }                       
                    catch (Exception ex) 
                    {
                        Console.WriteLine(ex);
                    }
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
