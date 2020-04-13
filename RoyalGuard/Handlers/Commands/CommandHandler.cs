using System;
using System.Collections.Generic;
using System.Linq;
using System.Threading.Tasks;
using RoyalGuard.Helpers.Commands;
using RoyalGuard.Modules;

namespace RoyalGuard.Commands
{
    public class CommandHandler
    {
        private readonly StringRenderer _stringRenderer;
        private readonly Bans _bans;
        public CommandHandler(StringRenderer stringRenderer, Bans bans)
        {
            _stringRenderer = stringRenderer;
            _bans = bans;
        }
        public async Task HandleCommand(DSharpPlus.Entities.DiscordMessage message)
        {
            switch (_stringRenderer.GetCommand(message))
            {
                case "ping":
                    await Other.Ping(message);
                    break;
                case "ban":
                    await _bans.BanUser(message);
                    break;
            }
        }
    }
}
