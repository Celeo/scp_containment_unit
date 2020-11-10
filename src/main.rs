use anyhow::anyhow;
use log::{debug, error, info, warn};
use serenity::{
    framework::standard::{
        help_commands,
        macros::{check, command, group, help},
        Args, CheckResult, CommandGroup, CommandOptions, CommandResult, HelpOptions,
        StandardFramework,
    },
    model::{channel::Message, gateway::Ready, id::UserId},
    prelude::*,
    utils::parse_mention,
};
use std::{collections::HashSet, env, path::Path, process};

mod store;
use store::{Config, Status};
mod util;

const CONFIG_FILE_NAME: &str = "scp_config.json";
const STATUS_FILE_NAME: &str = "scp_status.json";

#[command]
#[checks(RoleMod)]
async fn breach(context: &Context, message: &Message, mut args: Args) -> CommandResult {
    info!("Breach command used by {}", message.author.name);
    if args.len() != 1 {
        message
            .reply(&context, "Use as: `!breach <mention>`")
            .await?;
        return Ok(());
    }

    let mention_str: String = args.single().expect("Should not be reached");
    let mention_id = match parse_mention(&mention_str) {
        Some(m) => m,
        None => {
            message
                .reply(&context, "Could not resolve that mention to anything")
                .await?;
            return Ok(());
        }
    };
    let mentioned_member = match context
        .http
        .get_member(
            *message
                .guild_id
                .ok_or_else(|| anyhow!("Could not get guild ID from message"))?
                .as_u64(),
            mention_id,
        )
        .await
    {
        Ok(m) => m,
        Err(e) => {
            warn!(
                "Could not turn mentioned user id into member instance: {}",
                e
            );
            message
                .reply(&context, "Could not resolve that mention to a user")
                .await?;
            return Ok(());
        }
    };

    // TODO here's where the processing can happen

    // let client_data = context.data.read().await;
    // let config = client_data
    //     .get::<ConfigContainer>()
    //     .ok_or_else(|| anyhow!("Could not get config from client data"))?;
    // info!("Got config reference, bot token is {}", config.bot_token);

    Ok(())
}

#[command]
#[checks(RoleMod)]
async fn unbreach(context: &Context, message: &Message, args: Args) -> CommandResult {
    info!("Unbreach command used by {}", message.author.name);
    message.reply(&context, "Command not implemented").await?;
    // TODO
    Ok(())
}

#[command]
#[checks(RoleMod)]
async fn sitrep(context: &Context, message: &Message) -> CommandResult {
    info!("Sitrep command used by {}", message.author.name);
    message.reply(&context, "Command not implemented").await?;
    // TODO
    Ok(())
}

#[check]
#[name = "RoleMod"]
#[check_in_help(true)]
#[display_in_help(true)]
async fn role_mod_check(
    context: &Context,
    message: &Message,
    _: &mut Args,
    _: &CommandOptions,
) -> CheckResult {
    debug!("Checking permission for command");
    if let Ok(member) = message.member(&context).await {
        if let Ok(permissions) = member.permissions(&context).await {
            return permissions.manage_roles().into();
        }
    }
    debug!(
        "User '{}' does not have the required permission",
        message.author.name
    );
    false.into()
}

#[group]
#[commands(breach, unbreach, sitrep)]
struct General;

struct ConfigContainer;

impl TypeMapKey for ConfigContainer {
    type Value = Config;
}

struct StatusContainer;

impl TypeMapKey for StatusContainer {
    type Value = Status;
}

struct Handler;

#[serenity::async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: Context, _: Ready) {
        info!("Bot connected");
    }
}

fn load_flatfiles() -> (Config, Status) {
    debug!("Loading config from {}", CONFIG_FILE_NAME);
    let config: Config = match store::load(Path::new(CONFIG_FILE_NAME)) {
        Ok(c) => c,
        Err(e) => {
            error!("Could not load config: {}", e);
            process::exit(1);
        }
    };
    debug!("loading status from {}", STATUS_FILE_NAME);
    let status: Status = {
        let path = Path::new(STATUS_FILE_NAME);
        if path.exists() {
            match store::load(path) {
                Ok(s) => s,
                Err(e) => {
                    error!("Status file exists but could not be read: {}", e);
                    warn!("Defaulting to empty status");
                    Status::default()
                }
            }
        } else {
            warn!("Status file does not exist, defaulting to empty status");
            Status::default()
        }
    };
    (config, status)
}

#[help]
#[command_not_found_text = "Could not find: `{}`."]
#[max_levenshtein_distance(3)]
#[indention_prefix = "+"]
#[lacking_permissions = "Hide"]
#[lacking_role = "Nothing"]
#[wrong_channel = "Strike"]
async fn my_help(
    context: &Context,
    msg: &Message,
    args: Args,
    help_options: &'static HelpOptions,
    groups: &[&'static CommandGroup],
    owners: HashSet<UserId>,
) -> CommandResult {
    let _ = help_commands::with_embeds(context, msg, args, help_options, groups, owners).await;
    Ok(())
}

#[tokio::main]
async fn main() {
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "scp_containment_unit")
    }
    pretty_env_logger::init();

    let (config, status) = load_flatfiles();

    let mut client = Client::builder(&config.bot_token)
        .event_handler(Handler)
        .framework(
            StandardFramework::new()
                .configure(|c| c.prefix("!"))
                .help(&MY_HELP)
                .group(&GENERAL_GROUP),
        )
        .await
        .expect("Could not create client");
    {
        debug!("Moving config and status into client datastore");
        let mut client_data = client.data.write().await;
        client_data.insert::<ConfigContainer>(config);
        client_data.insert::<StatusContainer>(status);
    }
    debug!("Bot set up");

    debug!("Bot starting");
    if let Err(e) = client.start().await {
        error!("Error starting client: {}", e);
        process::exit(1);
    }
}
