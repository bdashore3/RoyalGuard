using System;
using System.Threading;
using System.Threading.Tasks;
using Microsoft.Extensions.Hosting;
using RoyalGuard.Helpers.Data;

namespace RoyalGuard.Services
{
    public class GuildDeleteService : IHostedService, IDisposable
    {
        private readonly GuildInfoHelper _guildInfoHelper;
        public GuildDeleteService(GuildInfoHelper guildInfoHelper)
        {
            _guildInfoHelper = guildInfoHelper;
        }
        private Timer _timer;

        // Starts a timer which fires every 4 days
        public Task StartAsync(CancellationToken stoppingToken)
        {
            _timer = new Timer(DoWork, null, 2000, 345600000);
            return Task.CompletedTask;
        }

        // Check which guilds need removal from the Database
        private async void DoWork(object state)
        {
            Console.WriteLine("Starting guild cleanup!");
            await _guildInfoHelper.CheckForRemoval();
        }

        // On stop, turn off the timer
        public Task StopAsync(CancellationToken cancellationToken)
        {
            _timer?.Change(Timeout.Infinite, 0);

            return Task.CompletedTask;
        }

        // Get rid of the timer when finished
        public void Dispose()
        {
            _timer?.Dispose();
        }
    }
}
