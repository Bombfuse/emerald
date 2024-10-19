use std::{fs::File, io::Write};

const SYSTEM_TEMPLATE_PT_0: &'static str = r#"
import emerald::{World, Emerald};

"#;

const SYSTEM_TEMPLATE_PT_2: &'static str = r#"
}
"#;

pub fn generate_system_file(name: &str) {
    // TODO: register new system to current module

    let mut file_content = SYSTEM_TEMPLATE_PT_0.to_string();

    let component_name = name;
    let component_declaration_line = format!(
        "pub fn {}_system(emd: &mut Emerald, world: &mut World)",
        component_name
    );
    file_content.push_str(&component_declaration_line);
    file_content.push_str(" {");
    file_content.push_str(&SYSTEM_TEMPLATE_PT_2);

    // TODO: create file with content
    let mut file = File::create(format!("{}_system.rs", name)).unwrap();
    file.write_all(file_content.as_bytes());
}
