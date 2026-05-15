use std::path::PathBuf;

pub struct FileDialog;

impl FileDialog {
    pub fn open_directory() -> Option<PathBuf> {
        rfd::FileDialog::new()
            .set_title("Select Git Repository")
            .pick_folder()
    }

    pub fn open_file() -> Option<PathBuf> {
        rfd::FileDialog::new().set_title("Select File").pick_file()
    }

    pub fn save_file() -> Option<PathBuf> {
        rfd::FileDialog::new().set_title("Save File").save_file()
    }
}
