using System;
using System.Threading;
using System.Threading.Tasks;
using Microsoft.Extensions.Hosting;

namespace RoyalGuard.Services
{
    public class BotHostedService : IHostedService
    {
        private readonly DiscordBot _discordBot;
        private readonly string _credsPath;
        public BotHostedService(DiscordBot discordBot, string credsPath)
        {
            _discordBot = discordBot;
            _credsPath = credsPath;
        }

        public Task StartAsync(CancellationToken cancellationToken)
        {
            return _discordBot.Start(_credsPath);
        }

        public Task StopAsync(CancellationToken cancellationToken)
        {
            return _discordBot.Stop();
        }
    }
}
