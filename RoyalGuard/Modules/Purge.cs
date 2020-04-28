using System;
using System.Threading.Tasks;
using DSharpPlus.Entities;
using RoyalGuard.Helpers.Commands;

namespace RoyalGuard.Modules
{
    public class Purge
    {
        private readonly StringRenderer _stringRenderer;

        public Purge(StringRenderer stringRenderer)
        {
            _stringRenderer = stringRenderer;
        }
        public async Task PurgeMessages(DiscordMessage message)
        {
            DiscordChannel channel = message.Channel;
            string purgeAmountString = _stringRenderer.GetWordFromIndex(message, 1);
            bool useInt = CheckPurgeInt(purgeAmountString);

            if (_stringRenderer.GetMessageCount(message) <= 1)
                await PurgeHelp(message);

            if (useInt)
            {
                int amount = int.Parse(purgeAmountString);

                if (amount > 100)
                {
                    await message.RespondAsync("You can only remove up to 100 messages at a time!");
                    return;
                }
            
                foreach(var i in await channel.GetMessagesBeforeAsync(message.Id, amount))
                    await channel.DeleteMessageAsync(i);
                
                await message.DeleteAsync();
            }
            else
            {
                ulong startId = UInt64.Parse(_stringRenderer.GetWordFromIndex(message, 1));
            
                var messages = await channel.GetMessagesAfterAsync(startId);

                if (messages.Count > 100)
                {
                    await message.RespondAsync("You can only remove up to 100 messages at a time!");
                    return;
                }

                foreach(var i in messages)
                    await channel.DeleteMessageAsync(i); 
                
                await channel.DeleteMessageAsync(await channel.GetMessageAsync(startId));
            }

            await channel.SendMessageAsync("Purge Complete.");
        }

        public bool CheckPurgeInt(string purgeAmountString)
        {
            try
            {
                int.Parse(purgeAmountString);
                return true;
            }
            catch (System.OverflowException)
            {
                return false;
            }
        }

        public async static Task PurgeHelp(DiscordMessage message)
        {
            DiscordEmbedBuilder eb = new DiscordEmbedBuilder();
            eb.WithTitle("Purge Help");
            eb.WithDescription("Description: Commands for bulk removes of messages in a server");
            eb.AddField("Commands", "purge <amount to remove>: Removes a specified amount of messages before the command. \n\n" +
                                    "purge <ID of message to remove>: Removes all messages between the ID and the command.");

            await message.RespondAsync("", false, eb.Build());
        }
    }
}
