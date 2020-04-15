using System;
using Microsoft.EntityFrameworkCore;
using Microsoft.EntityFrameworkCore.Design;
using RoyalGuard.Helpers.Security;

namespace RoyalGuard.Helpers.Data
{
    public class RoyalGuardFactory : IDesignTimeDbContextFactory<RoyalGuardContext>
    {
        public RoyalGuardContext CreateDbContext(string[] args)
        {
            CredentialsHelper.ReadCreds("info.json");
            var optionsBuilder = new DbContextOptionsBuilder<RoyalGuardContext>();
            optionsBuilder.UseNpgsql(CredentialsHelper.DBConnection);

            return new RoyalGuardContext(optionsBuilder.Options);
        }
    }
}