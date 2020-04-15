using System;
using System.Collections.Generic;
using System.Linq;
using System.Threading.Tasks;
using DSharpPlus.Entities;
using RoyalGuard.Helpers.Commands;
using RoyalGuard.Modules;

namespace RoyalGuard.Commands
{
    public class CommandHandler
    {
        private readonly StringRenderer _stringRenderer;
        private readonly Bans _bans;
        private readonly Mutes _mutes;
        private readonly Warns _warns;
        public CommandHandler(StringRenderer stringRenderer, Bans bans, Mutes mutes, Warns warns)
        {
            _stringRenderer = stringRenderer;
            _bans = bans;
            _mutes = mutes;
            _warns = warns;
        }

        public async Task HandleCommand(DSharpPlus.Entities.DiscordMessage message)
        {
            try {
                switch (_stringRenderer.GetCommand(message))
                {
                    case "ping":
                        await Other.Ping(message);
                        break;
                    case "ban":
                        await _bans.BanUser(message);
                        break;
                    case "unban":
                        await _bans.UnbanUser(message);
                        break;
                    case "warn":
                        await _warns.WarnUser(message);
                        break;
                    case "unwarn":
                        await _warns.UnwarnUser(message);
                        break;
                    case "getwarns":
                        await _warns.GetWarns(message);
                        break;
                }
            }
            catch (Exception ex)
            {
                Console.WriteLine(ex);
            }
        }
    }
}
