use std::io::Read;

pub struct PhotoManager {
    photos_directory: std::path::PathBuf,
}
impl PhotoManager {
    pub fn new(photos_directory: std::path::PathBuf) -> Self {
        return PhotoManager { photos_directory };
    }

    pub fn get_photo(&self, filename: &std::path::Path) -> std::io::Result<Vec<u8>> {
        let mut f = std::fs::File::open(self.photos_directory.join(filename))?;
        let mut buffer = vec![];
        f.read_to_end(&mut buffer)?;
        return Ok(buffer);
    }
}
