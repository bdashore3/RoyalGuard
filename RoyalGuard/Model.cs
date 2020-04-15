using System;
using Microsoft.EntityFrameworkCore;

namespace RoyalGuard.Helpers.Data
{
    public class RoyalGuardContext : DbContext
    {
        public RoyalGuardContext(DbContextOptions<RoyalGuardContext> options) : base(options)
        {
        }
        public DbSet<Mute> Mutes { get; set; }
        public DbSet<Warn> Warns { get; set; }
    }
    public class Mute
    {
        public Guid Id { get; set; }
        public ulong DiscordId { get; set; }
        public long MuteTime { get; set; }
    }
    public class Warn
    {
        public Guid Id { get; set; }
        public ulong DiscordId { get; set; }
        public int WarnNumber { get; set; }
    }
}
