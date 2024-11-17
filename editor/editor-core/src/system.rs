use std::{fs::File, io::Write};

use crate::module::EmeraldModule;

const SYSTEM_TEMPLATE_PT_0: &'static str = r#"
import emerald::{World, Emerald};

"#;

const SYSTEM_TEMPLATE_PT_2: &'static str = r#"
}
"#;

pub fn generate_system(request: GenerateSystemRequest) {}

struct GenerateSystemRequest {
    system_name: String,
}
fn generate_system_file_content(request: GenerateSystemRequest) -> String {
    let component_declaration_line = format!(
        "pub fn {}_system(emd: &mut Emerald, world: &mut World)",
        request.system_name
    );

    let mut file_content = SYSTEM_TEMPLATE_PT_0.to_string();
    file_content.push_str(&component_declaration_line);
    file_content.push_str(" {");
    file_content.push_str(&SYSTEM_TEMPLATE_PT_2);

    file_content
}

#[cfg(test)]
mod tests {
    use crate::system::{generate_system_file_content, GenerateSystemRequest};

    #[test]
    fn file_content() {
        let expected_file_content = "\nimport emerald::{World, Emerald};\n\npub fn test_system(emd: &mut Emerald, world: &mut World) {\n}\n";
        assert_eq!(
            &generate_system_file_content(GenerateSystemRequest {
                system_name: "test".to_string(),
            },),
            &expected_file_content
        );
    }
}
