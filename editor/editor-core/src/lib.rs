use generate_component::generate_component_file;
use generate_event::generate_event_file;
use generate_event_handler::generate_event_handler_file;
use generate_system::generate_system_file;

pub const CURRENT_VERSION: &'static str = "0.0.1";

pub mod generate_component;
pub mod generate_event;
pub mod generate_event_handler;
pub mod generate_module;
pub mod generate_system;
