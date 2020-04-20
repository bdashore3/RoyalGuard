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
            services.AddTransient<Bans>();
            services.AddTransient<StringRenderer>();
            services.AddTransient<Mutes>();
            services.AddTransient<Warns>();
            services.AddTransient<Help>();
            services.AddDbContext<RoyalGuardContext>(options => options.UseNpgsql(CredentialsHelper.DBConnection));
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
