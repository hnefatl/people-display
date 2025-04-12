use std::{io::Read, path::PathBuf};

use crate::homeassistant_types::EntityId;

const VALID_EXTENSIONS: [&str; 3] = ["png", "jpg", "jpeg"];

pub struct PhotoManager {
    photos_directory: std::path::PathBuf,
}
impl PhotoManager {
    pub fn new(photos_directory: std::path::PathBuf) -> Self {
        PhotoManager { photos_directory }
    }

    fn potential_paths(&self, entity_id: &impl EntityId) -> Vec<PathBuf> {
        // Replace `.` with `_` so that setting a `.png`/`.jpg` extension is easier.
        let filename = entity_id.to_string().replace('.', "_");
        let base_name = self.photos_directory.join(filename);
        VALID_EXTENSIONS
            .map(|ext| base_name.with_extension(ext))
            .to_vec()
    }

    // Vulnerable to race conditions.
    pub fn find_first_existing_path(&self, entity_id: &impl EntityId) -> Option<PathBuf> {
        self.potential_paths(entity_id)
            .iter()
            .find(|p| p.exists())
            .cloned()
    }

    pub fn get_photo(&self, entity_id: &impl EntityId) -> std::io::Result<Vec<u8>> {
        let Some(Ok(mut f)) = self
            .potential_paths(entity_id)
            .iter()
            .map(std::fs::File::open)
            .find(|f| f.is_ok())
        else {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Unable to find photo named any of {VALID_EXTENSIONS:?}"),
            ));
        };
        let mut buffer = vec![];
        f.read_to_end(&mut buffer)?;
        return Ok(buffer);
    }
}
