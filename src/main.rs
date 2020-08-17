use anyhow::{anyhow, Result};
use log::{debug, error, info};
use serenity::{
    framework::standard::{
        macros::{check, command, group},
        Args, CheckResult, CommandOptions, CommandResult, StandardFramework,
    },
    model::{channel::Message, guild::Member, id::GuildId},
    prelude::*,
};
use std::{env, path::Path, process};

/// Return user ids from the 'CONTAINMENT_USER_IDS' environment variable.
///
/// The environment variable is treated as a comma-delimited list of `u64`s.
fn get_containment_user_ids() -> Result<Vec<u64>> {
    Ok(env::var("CONTAINMENT_USER_IDS")?
        .split(',')
        .filter_map(|s| s.parse::<u64>().ok())
        .collect())
}

/// Return the value of the 'CONTAINMENT_ROLE' environment variables as a `u64`.
fn get_containment_role() -> Result<u64> {
    Ok(env::var("CONTAINMENT_ROLE")?.parse()?)
}

/// Returns a vector of `Member`s that match the user ids from the environment variable.
fn get_members_for_containment(context: &mut Context, guild_id: &GuildId) -> Result<Vec<Member>> {
    debug!("Getting member references for containment, matching env var");
    let to_contain_ids = get_containment_user_ids()?;
    let mut members = vec![];
    for member in guild_id.members_iter(&context) {
        let member = match member {
            Ok(m) => m,
            Err(e) => {
                error!("Could not get member: {}", e);
                continue;
            }
        };
        let user_id = member.user_id();
        if to_contain_ids.contains(user_id.as_u64()) {
            debug!(
                r#"User with member "{}" matches containment user ids list"#,
                member.display_name()
            );
            members.push(member);
        }
    }
    debug!(
        "Returning members matching containment: {}",
        members
            .iter()
            .map(|m| format!(r#""{}""#, m.display_name()))
            .collect::<Vec<_>>()
            .join(", ")
    );
    Ok(members)
}

/// Turns a collection of `Member`s into a String.
///
/// The content of that String depends on how many structs are
/// in the collection. The resulting String should be ready for
/// reading as normal English.
fn members_to_string(members: &[Member]) -> String {
    match members.len() {
        1 => format!(r#""{}""#, members[0].display_name()),
        2 => format!(
            r#""{}" and "{}""#,
            members[0].display_name(),
            members[1].display_name()
        ),
        _ => {
            let comma_seperated = members
                .iter()
                .map(|m| format!(r#""{}""#, m.display_name()))
                .take(members.len() - 1)
                .collect::<Vec<_>>()
                .join(", ");
            format!(
                "{}, and {}",
                comma_seperated,
                members[members.len() - 1].display_name()
            )
        }
    }
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
    message.channel_id.send_message(&context, |m| {
        m.content("Putting subjects into quarantine!");
        m
    })?;

    let guild = message
        .guild_id
        .ok_or_else(|| anyhow!("Could not get guild from message model"))?;
    let containment_role = get_containment_role()?;
    let members = get_members_for_containment(context, &guild)?;
    for mut member in members {
        debug!("Adding contained role to {}", member.display_name());
        if let Err(e) = member.add_role(&context, containment_role) {
            error!(
                "Could not add containment role {} to {}: {}",
                containment_role,
                member.display_name(),
                e
            );
        }
    }
    Ok(())
}

#[command]
#[checks(Admin)]
fn sitrep(context: &mut Context, message: &Message) -> CommandResult {
    info!("Sitrep command used by {}", message.author.name);
    let mut is_contained = false;
    let guild = message
        .guild_id
        .ok_or_else(|| anyhow!("Could not get guild from message model"))?;
    let containment_role = get_containment_role()?;
    let members = get_members_for_containment(context, &guild)?;
    for member in &members {
        let roles = member
            .roles
            .iter()
            .map(|r| {
                format!(
                    "{} ({})",
                    r.to_role_cached(&context).unwrap().name,
                    r.as_u64()
                )
            })
            .collect::<Vec<_>>();
        debug!("Roles for {}: {}", member.display_name(), roles.join(", "));

        if member
            .roles
            .iter()
            .map(|r| r.as_u64())
            .any(|r| r == &containment_role)
        {
            debug!(
                "Setting 'is_contained' to true because of {}",
                member.display_name()
            );
            is_contained = true;
            break;
        }
    }

    if is_contained {
        message.channel_id.send_message(&context, |m| {
            m.content(format!(
                "Subjects {} are contained, sir! o7",
                members_to_string(&members)
            ));
            m
        })?;
    } else {
        message.channel_id.send_message(&context, |m| {
            m.content("Standing at the ready! o7");
            m
        })?;
    }
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
    let members = get_members_for_containment(context, &guild)?;
    for mut member in members {
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

#[group]
#[commands(breach, unbreach, sitrep)]
struct General;

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
