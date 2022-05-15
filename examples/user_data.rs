use emerald::*;

pub fn main() {
    emerald::start(Box::new(UserDataExample {}), GameSettings::default())
}

pub struct UserDataExample {}
impl Game for UserDataExample {
    fn initialize(&mut self, mut emd: Emerald) {
        let my_json = r#"
            {
                "some_data": "this is some save file data for a game"
            }
        "#;

        // This will write the contents of `my_json` to a new file in the root user data directory.
        // This will overwrite any contents that previously existed at that location.
        emd.writer()
            .write_to_user_file(my_json.as_bytes(), "user_data_example.sav")
            .unwrap();
    }

    fn update(&mut self, _emd: Emerald) {}
}
