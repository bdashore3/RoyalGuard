using System;
using Microsoft.EntityFrameworkCore.Migrations;

namespace RoyalGuard.Migrations
{
    public partial class CascadeDeletes1 : Migration
    {
        protected override void Up(MigrationBuilder migrationBuilder)
        {
            migrationBuilder.DropPrimaryKey(
                name: "PK_GuildInfoStore",
                table: "GuildInfoStore");

            migrationBuilder.DropColumn(
                name: "Id",
                table: "GuildInfoStore");

            migrationBuilder.AddColumn<decimal>(
                name: "GuildInfoGuildId",
                table: "Warns",
                nullable: true);

            migrationBuilder.AddColumn<decimal>(
                name: "GuildInfoGuildId",
                table: "NewMembers",
                nullable: true);

            migrationBuilder.AddColumn<long>(
                name: "DeleteTime",
                table: "GuildInfoStore",
                nullable: false,
                defaultValue: 0L);

            migrationBuilder.AddPrimaryKey(
                name: "PK_GuildInfoStore",
                table: "GuildInfoStore",
                column: "GuildId");

            migrationBuilder.CreateIndex(
                name: "IX_Warns_GuildInfoGuildId",
                table: "Warns",
                column: "GuildInfoGuildId");

            migrationBuilder.CreateIndex(
                name: "IX_NewMembers_GuildInfoGuildId",
                table: "NewMembers",
                column: "GuildInfoGuildId");

            migrationBuilder.AddForeignKey(
                name: "FK_NewMembers_GuildInfoStore_GuildInfoGuildId",
                table: "NewMembers",
                column: "GuildInfoGuildId",
                principalTable: "GuildInfoStore",
                principalColumn: "GuildId",
                onDelete: ReferentialAction.Cascade);

            migrationBuilder.AddForeignKey(
                name: "FK_Warns_GuildInfoStore_GuildInfoGuildId",
                table: "Warns",
                column: "GuildInfoGuildId",
                principalTable: "GuildInfoStore",
                principalColumn: "GuildId",
                onDelete: ReferentialAction.Cascade);
        }

        protected override void Down(MigrationBuilder migrationBuilder)
        {
            migrationBuilder.DropForeignKey(
                name: "FK_NewMembers_GuildInfoStore_GuildInfoGuildId",
                table: "NewMembers");

            migrationBuilder.DropForeignKey(
                name: "FK_Warns_GuildInfoStore_GuildInfoGuildId",
                table: "Warns");

            migrationBuilder.DropIndex(
                name: "IX_Warns_GuildInfoGuildId",
                table: "Warns");

            migrationBuilder.DropIndex(
                name: "IX_NewMembers_GuildInfoGuildId",
                table: "NewMembers");

            migrationBuilder.DropPrimaryKey(
                name: "PK_GuildInfoStore",
                table: "GuildInfoStore");

            migrationBuilder.DropColumn(
                name: "GuildInfoGuildId",
                table: "Warns");

            migrationBuilder.DropColumn(
                name: "GuildInfoGuildId",
                table: "NewMembers");

            migrationBuilder.DropColumn(
                name: "DeleteTime",
                table: "GuildInfoStore");

            migrationBuilder.AddColumn<Guid>(
                name: "Id",
                table: "GuildInfoStore",
                type: "uuid",
                nullable: false,
                defaultValue: new Guid("00000000-0000-0000-0000-000000000000"));

            migrationBuilder.AddPrimaryKey(
                name: "PK_GuildInfoStore",
                table: "GuildInfoStore",
                column: "Id");
        }
    }
}
