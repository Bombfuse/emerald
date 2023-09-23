use emerald::file_loader::FileLoader;

pub struct DesktopFileLoader {}
impl DesktopFileLoader {
    pub fn new() -> Self {
        Self {}
    }
}
impl FileLoader for DesktopFileLoader {
    fn load_file(&mut self, _filepath: &str) -> Vec<u8> {
        todo!()
    }
}
