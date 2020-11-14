use crate::common::*;

pub async fn extract_mentioned_user(
    context: &Context,
    message: &Message,
    mention_str: &str,
) -> Result<Member> {
    let mention_id = parse_mention(&mention_str)
        .ok_or_else(|| anyhow!("Could not determine any mention in string"))?;
    let mentioned_member = context
        .http
        .get_member(
            *message
                .guild_id
                .ok_or_else(|| anyhow!("Could not get guild ID from message"))?
                .as_u64(),
            mention_id,
        )
        .await?;
    Ok(mentioned_member)
}
