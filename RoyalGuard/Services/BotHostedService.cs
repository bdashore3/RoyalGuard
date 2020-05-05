using System;
using System.Threading;
using System.Threading.Tasks;
using Microsoft.Extensions.Hosting;

namespace RoyalGuard.Services
{
    public class BotHostedService : IHostedService
    {
        private readonly DiscordBot _discordBot;
        public BotHostedService(DiscordBot discordBot)
        {
            _discordBot = discordBot;
        }

        public Task StartAsync(CancellationToken cancellationToken)
        {
            return _discordBot.Start();
        }

        public Task StopAsync(CancellationToken cancellationToken)
        {
            return _discordBot.Stop();
        }
    }
}
