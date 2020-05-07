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

        // calls the start method with the provided credentials path
        public async Task StartAsync(CancellationToken cancellationToken)
        {
            var task = _discordBot.Start(_credsPath);
            await Task.WhenAny(task, Task.Delay(Timeout.Infinite, cancellationToken));
        }

        // calls the stop method
        public async Task StopAsync(CancellationToken cancellationToken)
        {
            await _discordBot.Stop();
        }
    }
}
