use deadpool_postgres::Object;

use crate::characters::structs::PageCharacter;

#[derive(Debug, Clone)]
pub struct RitualCharacterData {
    pub page_character: PageCharacter,
    pub power_name: String,
    pub power_description: String,
    pub ritual_specific_artwork_s3_key: Option<String>,
}

impl RitualCharacterData {
    /// Gets RitualCharacterData for all characters in the database.
    pub async fn get_all_ritualists(db_connection: &Object) -> Result<Vec<Self>, postgres::Error> {
        // TODO: Select only what's necessary to speed it up.
        let character_rows = db_connection
            .query(
                "SELECT * FROM ritual_info INNER JOIN character ON ritual_info.character_id=character.id WHERE character.post_state='public' AND character.retirement_reason IS NULL ORDER BY character.short_name",
                &[],
            )
            .await
            .unwrap();

        character_rows.iter().map(Self::try_from).collect()
    }

    pub fn get_art_url(&self) -> String {
        let artwork_s3_key = self
            .ritual_specific_artwork_s3_key
            .as_ref()
            .unwrap_or(&self.page_character.page_img_key);
        crate::utils::get_s3_public_object_url(artwork_s3_key)
    }

    pub fn get_name(&self) -> &str {
        self.page_character
            .long_name
            .as_ref()
            .unwrap_or(&self.page_character.base_character.name)
    }

    pub fn get_thumbnail(&self) -> String {
        crate::utils::get_s3_public_object_url(&self.page_character.base_character.thumbnail_key)
    }
}

impl TryFrom<&tokio_postgres::Row> for RitualCharacterData {
    type Error = tokio_postgres::Error;

    fn try_from(row: &tokio_postgres::Row) -> Result<Self, Self::Error> {
        Ok(Self {
            page_character: PageCharacter::try_from(row)?,
            power_name: row.try_get("power_name")?,
            power_description: row.try_get("power_description")?,
            ritual_specific_artwork_s3_key: None, // TODO: Hook up
        })
    }
}
