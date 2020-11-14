use crate::common::*;

#[command]
#[checks(RoleMod)]
pub async fn breach(context: &Context, message: &Message, mut args: Args) -> CommandResult {
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
