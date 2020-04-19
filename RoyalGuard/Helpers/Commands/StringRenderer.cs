using System;
using System.Collections.Generic;
using System.Linq;
using System.Threading.Tasks;
using DSharpPlus.Entities;

namespace RoyalGuard.Helpers.Commands
{
    public class StringRenderer
    {
        public List<string> SplitMessage(DiscordMessage message)
        {
            string msg = message.Content.Substring(1);
            List<string> words = msg.Split(" ").ToList();
            return words;
        }

        public string GetCommand(DiscordMessage message)
        {
            var words = SplitMessage(message);
            string command = words[0].ToLower();
            return command;
        }

        public string GetWordFromIndex(DiscordMessage message, int index)
        {
            var words = SplitMessage(message);
            string word = words[index].ToLower();
            return word;
        }

        public string RemoveExtras(DiscordMessage message, int amount)
        {
            var words = SplitMessage(message);
            words.RemoveRange(0, amount);
            
            if (words.Count == 0)
                return null;

            string result = String.Join(" ", words.ToArray());
            Console.WriteLine(result);
            return result;
        }

        public int GetMessageCount(DiscordMessage message)
        {
            List<string> words = SplitMessage(message);
            return words.Count;
        }
    }
}
