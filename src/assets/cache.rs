use std::collections::HashMap;

#[derive(Debug)]
pub(crate) struct Cache {
    pub(crate) data: HashMap<String, Vec<u8>>,
}
impl Cache {
    pub fn new() -> Self {
        Cache {
            data: HashMap::new(),
        }
    }
}
