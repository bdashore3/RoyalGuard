using System;
using Microsoft.EntityFrameworkCore.Migrations;

namespace RoyalGuard.Migrations
{
    public partial class AddDeleteTable : Migration
    {
        protected override void Up(MigrationBuilder migrationBuilder)
        {
            migrationBuilder.DropColumn(
                name: "DeleteTime",
                table: "GuildInfoStore");

            migrationBuilder.CreateTable(
                name: "DeleteTimeStore",
                columns: table => new
                {
                    Id = table.Column<Guid>(nullable: false),
                    GuildInfoGuildId = table.Column<decimal>(nullable: false),
                    DeleteTime = table.Column<long>(nullable: false)
                },
                constraints: table =>
                {
                    table.PrimaryKey("PK_DeleteTimeStore", x => x.Id);
                    table.ForeignKey(
                        name: "FK_DeleteTimeStore_GuildInfoStore_GuildInfoGuildId",
                        column: x => x.GuildInfoGuildId,
                        principalTable: "GuildInfoStore",
                        principalColumn: "GuildId",
                        onDelete: ReferentialAction.Cascade);
                });

            migrationBuilder.CreateIndex(
                name: "IX_DeleteTimeStore_GuildInfoGuildId",
                table: "DeleteTimeStore",
                column: "GuildInfoGuildId");
        }

        protected override void Down(MigrationBuilder migrationBuilder)
        {
            migrationBuilder.DropTable(
                name: "DeleteTimeStore");

            migrationBuilder.AddColumn<long>(
                name: "DeleteTime",
                table: "GuildInfoStore",
                type: "bigint",
                nullable: false,
                defaultValue: 0L);
        }
    }
}
