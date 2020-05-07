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

        public async Task StartAsync(CancellationToken cancellationToken)
        {
            var task = _discordBot.Start(_credsPath);
            await Task.WhenAny(task, Task.Delay(Timeout.Infinite, cancellationToken));
        }

        public async Task StopAsync(CancellationToken cancellationToken)
        {
            await _discordBot.Stop();
        }
    }
}
