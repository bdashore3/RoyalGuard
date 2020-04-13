using System;
using System.Collections.Generic;
using System.Linq;
using System.Threading.Tasks;

namespace RoyalGuard.Helpers.Commands
{
    public class StringRenderer
    {
        public List<string> SplitMessage(DSharpPlus.Entities.DiscordMessage message)
        {
            string msg = message.Content.Substring(1);
            List<string> words = msg.Split(" ").ToList();
            return words;
        }

        public string GetCommand(DSharpPlus.Entities.DiscordMessage message)
        {
            var words = SplitMessage(message);
            string command = words[0].ToLower();
            return command;
        }

        public string RemoveExtras(DSharpPlus.Entities.DiscordMessage message, int amount)
        {
            var words = SplitMessage(message);
            words.RemoveRange(0, amount);
            string result = String.Join(" ", words.ToArray());
            return result;
        }
    }
}
