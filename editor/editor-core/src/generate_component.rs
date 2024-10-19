use std::{fs::File, io::Write};

const COMPONENT_TEMPLATE_PT_0: &'static str = r#"
import emerald::serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
#[serde(crate = "emerald::serde")]
"#;

const COMPONENT_TEMPLATE_PT_2: &'static str = r#"
}
"#;

pub fn generate_component_file(name: &str) {
    let mut file_content = COMPONENT_TEMPLATE_PT_0.to_string();

    let component_name = name;
    let component_declaration_line = format!("pub struct {}", component_name);

    file_content.push_str(&component_declaration_line);
    file_content.push_str(" {");
    file_content.push_str(&COMPONENT_TEMPLATE_PT_2);

    // TODO: create file with content
    let mut file = File::create(format!("{}_component.rs", name)).unwrap();
    file.write_all(file_content.as_bytes());

    // TODO: register newly created component in this modules init function
}
