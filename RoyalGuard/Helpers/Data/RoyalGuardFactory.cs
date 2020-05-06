using System;
using Microsoft.EntityFrameworkCore;
using Microsoft.EntityFrameworkCore.Design;
using RoyalGuard.Helpers.Data;
using RoyalGuard.Helpers.Security;

namespace RoyalGuard
{
    public class DbMigrationFactory : IDesignTimeDbContextFactory<RoyalGuardContext>
    {
        public RoyalGuardContext CreateDbContext(string[] args)
        {
            var optionsBuilder = new DbContextOptionsBuilder<RoyalGuardContext>();
            optionsBuilder.UseNpgsql(CredentialsHelper.GetConnectionString("info.json"));

            return new RoyalGuardContext(optionsBuilder.Options);
        }
    }
}
