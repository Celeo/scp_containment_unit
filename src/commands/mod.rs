use crate::common::*;

pub mod breach;
pub use breach::*;

#[check]
#[name = "RoleMod"]
#[check_in_help(true)]
#[display_in_help(true)]
pub async fn role_mod_check(
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
