mod common;
use common::*;

mod commands;
mod store;
mod util;

const CONFIG_FILE_NAME: &str = "scp_config.json";
const STATUS_FILE_NAME: &str = "scp_status.json";

#[command]
#[checks(RoleMod)]
async fn breach(context: &Context, message: &Message, mut args: Args) -> CommandResult {
    // validate command
    info!("Breach command used by {}", message.author.name);
    if args.len() != 1 {
        message
            .reply(&context, "Use as: `!breach <mention>`")
            .await?;
        return Ok(());
    }

    // get targeted member
    let mention_str: String = args.single().expect("Should not be reached");
    let mut mentioned = match extract_mentioned_user(context, message, &mention_str).await {
        Ok(m) => m,
        Err(e) => {
            warn!("Could not find mentioned user in command: {}", e);
            message
                .reply(
                    context,
                    format!(r#"Could not find mentioned user "{}""#, mention_str),
                )
                .await?;
            return Ok(());
        }
    };

    // prep the stores
    let mut client_data = context.data.write().await;
    let (contained_role_id, remove_role_prefix) = {
        // this required syntax for navigating the locks is annoying, but what are ya gonna do
        let bot_config = client_data
            .get::<ConfigContainer>()
            .ok_or_else(|| anyhow!("Could not get config from client data"))?;
        (
            bot_config.role_to_add,
            bot_config.role_prefix_to_remove.clone(),
        )
    };
    let bot_status = client_data
        .get_mut::<StatusContainer>()
        .ok_or_else(|| anyhow!("Could not get status from client data"))?;

    // check for already being contained
    if bot_status
        .to_restore
        .iter()
        .find(|&cu| cu.user_id == *mentioned.user.id.as_u64())
        .is_some()
    {
        message
            .reply(
                context,
                "That user is already contained; use `!unbreach <mention>` to restore them",
            )
            .await?;
        return Ok(());
    };

    // strip matching roles from member
    let current_roles = mentioned.roles(context).await;
    let roles_to_restore = match current_roles {
        Some(current_roles) => {
            let roles_to_remove: Vec<_> = current_roles
                .iter()
                .filter(|&role| role.name.starts_with(&remove_role_prefix))
                .map(|role| role.id)
                .collect();
            mentioned.remove_roles(context, &roles_to_remove).await?;
            roles_to_remove
        }
        None => {
            debug!("{} does not have roles", mentioned.user.name);
            Vec::new()
        }
    };

    // add the contained role
    mentioned.add_role(context, contained_role_id).await?;

    // add to bot_status
    bot_status.to_restore.push(ContainedUser::new(
        mentioned.user.id.as_u64(),
        &mentioned.user.name,
        &roles_to_restore
            .iter()
            .map(|&role_id| *role_id.as_u64())
            .collect::<Vec<u64>>(),
    ));

    // reply
    message.reply(context, "Aye aye, sir!").await?;

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
