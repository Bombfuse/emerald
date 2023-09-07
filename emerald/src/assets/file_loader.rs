/// The thing that is responsible for loading data into memory
pub trait FileLoader {
    /// Load a file sync
    fn load_file(&mut self, filepath: &str) -> Vec<u8>;
}
