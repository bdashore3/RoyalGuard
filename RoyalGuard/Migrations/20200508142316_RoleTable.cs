using System;
using System.Collections.Generic;
using Microsoft.EntityFrameworkCore.Migrations;

namespace RoyalGuard.Migrations
{
    public partial class RoleTable : Migration
    {
        protected override void Up(MigrationBuilder migrationBuilder)
        {
            migrationBuilder.AddColumn<List<ulong>>(
                name: "RolesList",
                table: "NewMembers",
                nullable: true);

            migrationBuilder.CreateTable(
                name: "WelcomeRoles",
                columns: table => new
                {
                    Id = table.Column<Guid>(nullable: false),
                    GuildInfoGuildId = table.Column<decimal>(nullable: false),
                    RoleId = table.Column<decimal>(nullable: false)
                },
                constraints: table =>
                {
                    table.PrimaryKey("PK_WelcomeRoles", x => x.Id);
                    table.ForeignKey(
                        name: "FK_WelcomeRoles_GuildInfoStore_GuildInfoGuildId",
                        column: x => x.GuildInfoGuildId,
                        principalTable: "GuildInfoStore",
                        principalColumn: "GuildId",
                        onDelete: ReferentialAction.Cascade);
                });

            migrationBuilder.CreateIndex(
                name: "IX_WelcomeRoles_GuildInfoGuildId",
                table: "WelcomeRoles",
                column: "GuildInfoGuildId");
        }

        protected override void Down(MigrationBuilder migrationBuilder)
        {
            migrationBuilder.DropTable(
                name: "WelcomeRoles");

            migrationBuilder.DropColumn(
                name: "RolesList",
                table: "NewMembers");
        }
    }
}
