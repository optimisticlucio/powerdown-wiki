use deadpool::managed::Object;
use deadpool_postgres::Manager;

#[derive(Debug, Clone)]
pub struct RitualCharacterData {
    pub name: String,
    pub power_name: String,
    pub power_description: String,
    pub is_main_character: bool,
    pub artwork_s3_key: String,
}

impl RitualCharacterData {
    /// Gets RitualCharacterData for all characters in the database.
    pub async fn get_all_characters(db_connection: &Object<Manager>) -> Vec<Self> {
        // TODO: Select only what's necessary to speed it up.
        let character_rows = db_connection
            .query(
                "SELECT * FROM ritual_info INNER JOIN character ON ritual_info.character_id == character.id WHERE character.post_state='public' AND character.retirement_reason IS NULL ORDER_BY character.short_name",
                &[],
            )
            .await
            .unwrap();

        let converted_rows: Result<Vec<Self>, _> =
            character_rows.iter().map(Self::try_from).collect();

        converted_rows
            .map_err(|err| eprintln!("Failed to read RitualCharacterData: {err:?}"))
            .unwrap()
    }

    pub fn get_art_url(&self) -> String {
        crate::utils::get_s3_public_object_url(&self.artwork_s3_key)
    }
}

impl TryFrom<&tokio_postgres::Row> for RitualCharacterData {
    type Error = tokio_postgres::Error;

    fn try_from(row: &tokio_postgres::Row) -> Result<Self, Self::Error> {
        let long_name: Option<String> = row.try_get("long_name")?;

        let name = long_name.unwrap_or(row.try_get("short_name")?);

        Ok(Self {
            name,
            power_name: row.try_get("power_name")?,
            power_description: row.try_get("power_description")?,
            is_main_character: row.try_get("is_main_character")?,
            artwork_s3_key: row.try_get("page_image")?,
        })
    }
}
