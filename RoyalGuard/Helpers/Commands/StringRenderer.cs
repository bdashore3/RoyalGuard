using System;
using System.Collections.Generic;
using System.Linq;
using System.Threading.Tasks;
using DSharpPlus.Entities;
using RoyalGuard.Handlers;

namespace RoyalGuard.Helpers.Commands
{
    public class StringRenderer
    {
        private readonly TrieHandler _trieHandler;
        public StringRenderer(TrieHandler trieHandler)
        {
            _trieHandler = trieHandler;
        }

        // Split the message into a list of words and remove the prefix
        public List<string> SplitMessage(DiscordMessage message)
        {
            string msg = message.Content.Substring(_trieHandler.GetPrefix(message.Channel.GuildId).Length);
            List<string> words = msg.Split(" ").ToList();
            return words;
        }

        // Gets only the word at index 0
        public string GetCommand(DiscordMessage message)
        {
            var words = SplitMessage(message);
            string command = words[0].ToLower();
            return command;
        }

        // Gets the word from a provided index
        public string GetWordFromIndex(DiscordMessage message, int index)
        {
            var words = SplitMessage(message);
            string word = words[index].ToLower();
            return word;
        }

        // Remove any extra words such as prefix and instruction if we want a joined string
        public string RemoveExtras(DiscordMessage message, int amount)
        {
            var words = SplitMessage(message);
            words.RemoveRange(0, amount);
            
            if (words.Count == 0)
                return null;

            string result = String.Join(" ", words.ToArray());
            return result;
        }

        // Gets the length of the list
        public int GetMessageCount(DiscordMessage message)
        {
            List<string> words = SplitMessage(message);
            return words.Count;
        }
    }
}
