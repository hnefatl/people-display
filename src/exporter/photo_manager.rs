use std::io::Read;

pub struct PhotoManager {
    photos_directory: std::path::PathBuf,
}
impl PhotoManager {
    pub fn new(photos_directory: std::path::PathBuf) -> Self {
        PhotoManager { photos_directory }
    }

    pub fn get_photo(&self, filename: &std::path::Path) -> std::io::Result<Vec<u8>> {
        let base_name = self.photos_directory.join(filename);
        let valid_extensions = ["png", "jpg", "jpeg"];
        let filenames = valid_extensions.map(|ext| (base_name.with_extension(ext)));
        for filename in &filenames {
            if let Ok(mut f) = std::fs::File::open(filename) {
                let mut buffer = vec![];
                f.read_to_end(&mut buffer)?;
                return Ok(buffer);
            }
        }
        Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Unable to find photo named any of {filenames:?}"),
        ))
    }
}
