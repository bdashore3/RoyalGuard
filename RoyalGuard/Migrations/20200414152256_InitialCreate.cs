using System;
using Microsoft.EntityFrameworkCore.Migrations;

namespace RoyalGuard.Migrations
{
    public partial class InitialCreate : Migration
    {
        protected override void Up(MigrationBuilder migrationBuilder)
        {
            migrationBuilder.CreateTable(
                name: "Mutes",
                columns: table => new
                {
                    Id = table.Column<Guid>(nullable: false),
                    DiscordId = table.Column<decimal>(nullable: false),
                    MuteTime = table.Column<long>(nullable: false)
                },
                constraints: table =>
                {
                    table.PrimaryKey("PK_Mutes", x => x.Id);
                });

            migrationBuilder.CreateTable(
                name: "Warns",
                columns: table => new
                {
                    Id = table.Column<Guid>(nullable: false),
                    DiscordId = table.Column<decimal>(nullable: false),
                    WarnNumber = table.Column<int>(nullable: false)
                },
                constraints: table =>
                {
                    table.PrimaryKey("PK_Warns", x => x.Id);
                });
        }

        protected override void Down(MigrationBuilder migrationBuilder)
        {
            migrationBuilder.DropTable(
                name: "Mutes");

            migrationBuilder.DropTable(
                name: "Warns");
        }
    }
}
