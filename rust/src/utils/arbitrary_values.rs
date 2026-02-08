use deadpool::managed::Object;
use deadpool_postgres::Manager;

/// Gets the invite link for the discord server. Returns None if err, or if the link is set to null in the DB.
pub async fn get_discord_link(db_connection: &Object<Manager>) -> Option<String> {
    const GET_DISCORD_LINK_QUERY: &str =
        "SELECT item_value FROM arbitrary_value WHERE item_key='discord_invite_url';";

    // Assume `discord_invite_url` key exists
    let resulted_row = db_connection
        .query_one(GET_DISCORD_LINK_QUERY, &[])
        .await
        .map_err(|err| {
            eprintln!(
                "[GET DISCORD LINK] Failed reading `discord_invite_url`! Returning None. Err: {err:?}",
            )
        })
        .ok()?;

    let discord_invite_url: String = resulted_row.get(0);

    if discord_invite_url.is_empty() {
        None
    } else {
        Some(discord_invite_url)
    }
}

/// Sets the invite link for the discord server. Set to None if the discord goes on lockdown.
pub async fn set_discord_link(
    db_connection: &Object<Manager>,
    new_link: &str,
) -> Result<u64, postgres::Error> {
    const SET_DISCORD_LINK_QUERY: &str =
        "UPDATE arbitrary_value SET item_value=$1 WHERE item_key='discord_invite_url';";

    // TODO: Sanitize this a little more, this is fairly benign.
    let sanitized_link = new_link.trim();

    db_connection
        .execute(SET_DISCORD_LINK_QUERY, &[&sanitized_link])
        .await
}
