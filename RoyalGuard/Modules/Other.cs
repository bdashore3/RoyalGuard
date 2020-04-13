using System;
using System.Threading.Tasks;
using DSharpPlus.Entities;

namespace RoyalGuard.Modules
{
    public class Other
    {
        public static async Task Ping(DiscordMessage message)
        {
            await message.RespondAsync("Pong!");
        }
    }
}
