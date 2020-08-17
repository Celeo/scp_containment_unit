use anyhow::{anyhow, Result};
use log::{debug, error, info};
use serenity::{
    framework::standard::{
        macros::{check, command, group},
        Args, CheckResult, CommandOptions, CommandResult, StandardFramework,
    },
    model::channel::Message,
    prelude::*,
};
use std::{env, path::Path, process};

#[group]
#[commands(breach, unbreach, sitrep)]
struct General;

fn get_containment_user_ids() -> Result<Vec<u64>> {
    Ok(env::var("CONTAINMENT_USER_IDS")?
        .split(',')
        .filter_map(|s| s.parse::<u64>().ok())
        .collect())
}

fn get_containment_role() -> Result<u64> {
    Ok(env::var("CONTAINMENT_ROLE")?.parse()?)
}

#[command]
#[checks(Admin)]
fn breach(context: &mut Context, message: &Message) -> CommandResult {
    info!("Breach command used by {}", message.author.name);
    message.reply(&context, "Aye, aye sir!")?;
    message.channel_id.send_message(&context, |m| {
        m.content("Containment breach detected!");
        m
    })?;

    let guild = message
        .guild_id
        .ok_or_else(|| anyhow!("Could not get guild from message model"))?;

    let to_contain_ids = get_containment_user_ids()?;
    let containment_role = get_containment_role()?;

    for member in guild.members_iter(&context) {
        let mut member = match member {
            Ok(m) => m,
            Err(e) => {
                error!("Could not get member: {}", e);
                continue;
            }
        };
        let user_id = member.user_id();
        if to_contain_ids.contains(user_id.as_u64()) {
            if let Err(e) = member.add_role(&context, containment_role) {
                error!(
                    "Could not add containment role {} to {}: {}",
                    containment_role,
                    member.display_name(),
                    e
                );
            }
        }
    }
    Ok(())
}

#[command]
#[checks(Admin)]
fn sitrep(context: &mut Context, message: &Message) -> CommandResult {
    message.channel_id.send_message(&context, |m| {
        m.content("Standing at the ready! o7");
        m
    })?;
    Ok(())
}

#[command]
#[checks(Admin)]
fn unbreach(context: &mut Context, message: &Message) -> CommandResult {
    info!("Unbreach command used by {}", message.author.name);
    message.reply(&context, "Aye, aye sir!")?;

    let guild = message
        .guild_id
        .ok_or_else(|| anyhow!("Could not get guild from message model"))?;
    let containment_role = get_containment_role()?;
    for member in guild.members_iter(&context) {
        let mut member = match member {
            Ok(m) => m,
            Err(e) => {
                error!("Could not get member: {}", e);
                continue;
            }
        };
        if let Err(e) = member.remove_role(&context, containment_role) {
            error!(
                "Error removing containment role from {}: {}",
                member.display_name(),
                e
            );
        }
    }

    Ok(())
}

#[check]
#[name = "Admin"]
#[check_in_help(true)]
#[display_in_help(true)]
fn admin_check(ctx: &mut Context, msg: &Message, _: &mut Args, _: &CommandOptions) -> CheckResult {
    if let Some(member) = msg.member(&ctx.cache) {
        if let Ok(permissions) = member.permissions(&ctx.cache) {
            return permissions.administrator().into();
        }
    }
    false.into()
}

struct Handler;

impl EventHandler for Handler {}

fn main() -> Result<()> {
    // environment setup
    if Path::new(".env").exists() {
        kankyo::init()?;
    }
    pretty_env_logger::init();
    let token = match env::var("DISCORD_TOKEN") {
        Ok(t) => t,
        Err(_) => {
            error!("Environment variable 'DISCORD_TOKEN' is not set");
            process::exit(1);
        }
    };
    debug!("Token loaded from environment variable");

    // bot setup
    let mut client = Client::new(&token, Handler)?;
    client.with_framework(
        StandardFramework::new()
            .configure(|c| c.prefix("!"))
            .group(&GENERAL_GROUP),
    );
    debug!("Bot set up");

    // run bot
    if let Err(e) = client.start() {
        error!("Error starting client: {}", e);
    }
    Ok(())
}
