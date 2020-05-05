using System;
using System.Threading.Tasks;
using Microsoft.EntityFrameworkCore;
using Microsoft.Extensions.DependencyInjection;
using Microsoft.Extensions.Hosting;
using RoyalGuard.Commands;
using RoyalGuard.Helpers.Commands;
using RoyalGuard.Helpers.Data;
using RoyalGuard.Helpers.Security;
using RoyalGuard.Modules;
using RoyalGuard.Handlers;
using RoyalGuard.Services;

namespace RoyalGuard
{
    class Program
    {
        public static IHostBuilder CreateHostBuilder(string[] args) =>
            Host.CreateDefaultBuilder(args)
                .ConfigureServices(services =>
                {
                    services.AddDbContext<RoyalGuardContext>(options => options.UseNpgsql(CredentialsHelper.GetConnectionString()));
                    services.AddHostedService<GuildDeleteService>();
                    services.AddSingleton<DiscordBot>();
                    services.AddSingleton<CommandHandler>();
                    services.AddScoped<PermissionsHandler>();
                    services.AddScoped<NewMemberHandler>();
                    services.AddScoped<PrefixHelper>();
                    services.AddScoped<TrieHandler>();
                    services.AddTransient<Bans>();
                    services.AddTransient<StringRenderer>();
                    services.AddTransient<Mutes>();
                    services.AddTransient<Warns>();
                    services.AddTransient<Help>();
                    services.AddTransient<Purge>();
                    services.AddTransient<TimeConversion>();
                    services.AddTransient<GuildInfoHelper>();
                    services.AddHostedService<BotHostedService>();
                });

        static async Task Main(string[] args)
        {
            // Read credentials from the JSON file
            CredentialsHelper.ReadCreds(args[0]);

            // Startup the bot!
            using var host = CreateHostBuilder(args).Build();
            
            await host.RunAsync();
        }
    }
}
