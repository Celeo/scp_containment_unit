import json
from typing import Dict

import discord
from discord.ext import commands
from discord.ext.commands import Context
from loguru import logger


__version__ = "0.1.0"

CONFIG_FILE_NAME = "scpcu_config.json"
RUNTIME_INFO_FILE_NAME = "scpcu_data.json"

description = """SCP Containment Unit, here to contain your SCPs!"""
intents = discord.Intents.default()
intents.members = True
bot = commands.Bot(command_prefix="!", description=description, intents=intents)


@bot.event
async def on_ready():
    logger.debug("Bot::on_ready")


def command_perms_check(context: Context) -> bool:
    """Command check gate that prevents commands from being issued by non-admins."""
    return context.author.guild_permissions.administrator


@bot.command(brief="Put an SCP into containment")
@commands.check(command_perms_check)
async def breach(context: Context, *args: str) -> None:
    logger.debug(
        f"Bot::command::breach by {context.author.name} in {context.channel.name}"
    )
    await context.send("Command not implemented!")


@bot.command(brief="Let someone out of containment")
@commands.check(command_perms_check)
async def unbreach(context: Context, *args: str) -> None:
    logger.debug(
        f"Bot::comand::unbreach by {context.author.name} in {context.channel.name}"
    )
    await context.send("Command not implemented!")


@bot.command(brief="Get a situation report of the containment facilities")
@commands.check(command_perms_check)
async def sitrep(context: Context, *args: str) -> None:
    logger.debug(
        f"Bot::comand::sitrep by {context.author.name} in {context.channel.name}"
    )
    await context.send("Command not implemented!")


def load_config() -> Dict[str, str]:
    with open(CONFIG_FILE_NAME) as f:
        return json.load(f)


if __name__ == "__main__":
    logger.debug("Setting up")
    config = load_config()
    bot.run(config["token"])
    logger.warning("Bot terminated")
