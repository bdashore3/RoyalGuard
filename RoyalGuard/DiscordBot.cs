using System;
using System.Threading.Tasks;
using DSharpPlus;
using DSharpPlus.Entities;
using RoyalGuard.Commands;
using RoyalGuard.Helpers.Security;

namespace RoyalGuard
{
    public class DiscordBot
    {
        public static DiscordShardedClient discord;
        private readonly CommandHandler _commandHandler; 

        public DiscordBot(CommandHandler commandHandler)
        {
            _commandHandler = commandHandler;
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
                    await _commandHandler.HandleCommand(e.Message);
            };

            await discord.StartAsync();

            Console.WriteLine("The bot is online and ready to work!");
            await Task.Delay(-1); 
        }
    }
}
