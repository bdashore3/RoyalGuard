using System;
using System.Threading.Tasks;
using Microsoft.Extensions.DependencyInjection;
using RoyalGuard.Commands;
using RoyalGuard.Helpers.Commands;
using RoyalGuard.Helpers.Security;
using RoyalGuard.Modules;

namespace RoyalGuard
{
    class Program
    {
        private static IServiceCollection ConfigureServices()
        {
            IServiceCollection services = new ServiceCollection();
            services.AddScoped<DiscordBot>();
            services.AddScoped<CommandHandler>();
            services.AddTransient<Bans>();
            services.AddTransient<StringRenderer>();
            return services;
        }

        static async Task Main(string[] args)
        {
            CredentialsHelper.ReadCreds(args[0]);

            var services = ConfigureServices();

            var serviceProvider = services.BuildServiceProvider();

            await serviceProvider.GetService<DiscordBot>().Start();
        }
    }
}
