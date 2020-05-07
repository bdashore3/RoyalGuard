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

        public Task StartAsync(CancellationToken stoppingToken)
        {
            _timer = new Timer(DoWork, null, 2000, 345600000);
            return Task.CompletedTask;
        }

        private async void DoWork(object state)
        {
            await _guildInfoHelper.CheckForRemoval();
        }

        public Task StopAsync(CancellationToken cancellationToken)
        {
            _timer?.Change(Timeout.Infinite, 0);

            return Task.CompletedTask;
        }

        public void Dispose()
        {
            _timer?.Dispose();
        }
    }
}
