using System;
using System.Threading.Tasks;
using Microsoft.EntityFrameworkCore;
using Microsoft.Extensions.DependencyInjection;
using RoyalGuard.Commands;
using RoyalGuard.Helpers.Commands;
using RoyalGuard.Helpers.Data;
using RoyalGuard.Helpers.Security;
using RoyalGuard.Modules;
using RoyalGuard.Handlers;

namespace RoyalGuard
{
    class Program
    {
        private static IServiceCollection ConfigureServices()
        {
            IServiceCollection services = new ServiceCollection();
            services.AddScoped<DiscordBot>();
            services.AddScoped<CommandHandler>();
            services.AddScoped<PermissionsHandler>();
            services.AddScoped<NewMemberHandler>();
            services.AddScoped<PrefixHelper>();
            services.AddScoped<TrieHandler>();
            services.AddTransient<Bans>();
            services.AddTransient<StringRenderer>();
            services.AddTransient<Mutes>();
            services.AddTransient<Warns>();
            services.AddTransient<Help>();
            services.AddTransient<TimeConversion>();
            services.AddDbContext<RoyalGuardContext>(options => options.UseNpgsql(CredentialsHelper.DBConnection));
            return services;
        }

        static async Task Main(string[] args)
        {
            // Read credentials from the JSON file
            CredentialsHelper.ReadCreds(args[0]);

            // Configure all classes as services
            var services = ConfigureServices();

            // Build the interface for these services
            var serviceProvider = services.BuildServiceProvider();

            // Startup the bot!
            await serviceProvider.GetService<DiscordBot>().Start();
        }
    }
}
