#![deny(future_incompatible, nonstandard_style)]

pub mod assets;
pub mod audio;
pub mod colors;
pub mod core;
pub mod input;
pub mod logging;
pub mod profiling;
pub mod rendering;
pub mod types;
pub mod world;

use crate::core::game_engine::GameEngine;

pub use crate::assets::*;
pub use crate::colors::*;
pub use crate::core::*;
pub use crate::input::*;
pub use crate::rendering::*;
pub use crate::types::*;
pub use crate::world::physics::*;
pub use crate::world::*;
pub use audio::*;
pub use logging::*;
use winit::dpi::PhysicalSize;
use winit::event::ElementState;
use winit::event::Event;
use winit::event::KeyboardInput;
use winit::event::VirtualKeyCode;
use winit::event::WindowEvent;
use winit::event_loop::ControlFlow;
use winit::event_loop::EventLoop;
use winit::window::WindowBuilder;

pub use serde;
pub use toml;

pub use rapier2d::{
    crossbeam,
    dynamics::{RigidBody, RigidBodyBuilder, RigidBodyHandle},
    geometry::{Collider, ColliderBuilder, ColliderHandle, InteractionGroups, Ray},
    na as nalgebra,
    na::Vector2,
    parry,
};
//

#[cfg(feature = "gamepads")]
pub use gamepad;
#[cfg(feature = "gamepads")]
pub use gamepad::{Button, Joystick};

pub fn start(game: Box<dyn Game>, settings: GameSettings) {
    pollster::block_on(run(game, settings)).unwrap();
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
async fn run(game: Box<dyn Game>, settings: GameSettings) -> Result<(), EmeraldError> {
    env_logger::init();

    #[cfg(target_arch = "wasm32")]
    {
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));
        console_log::init_with_level(log::Level::Warn).expect("Couldn't initialize logger");
    }

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_inner_size(PhysicalSize::new(
            settings.render_settings.resolution.0,
            settings.render_settings.resolution.1,
        ))
        .build(&event_loop)
        .unwrap();
    let mut game_engine = GameEngine::new(game, &window, &settings).await?;
    game_engine.initialize()?;
    #[cfg(target_arch = "wasm32")]
    {
        // Winit prevents sizing with CSS, so we have to set
        // the size manually when on web.
        use winit::dpi::PhysicalSize;
        window.set_inner_size(PhysicalSize::new(450, 400));

        use winit::platform::web::WindowExtWebSys;
        web_sys::window()
            .and_then(|win| win.document())
            .and_then(|doc| {
                let dst = doc.get_element_by_id("wasm-example")?;
                let canvas = web_sys::Element::from(window.canvas());
                dst.append_child(&canvas).ok()?;
                Some(())
            })
            .expect("Couldn't append canvas to document body.");
    }

    event_loop.run(move |event, _, control_flow| match event {
        Event::MainEventsCleared => {
            // RedrawRequested will only trigger once, unless we manually
            // request it.
            window.request_redraw();
        }
        Event::RedrawRequested(window_id) if window_id == window.id() => {
            if let Err(_) = game_engine.update() {
                *control_flow = ControlFlow::Exit;
            } else {
                match game_engine.render() {
                    Ok(_) => {}
                    // Reconfigure the surface if lost
                    Err(wgpu::SurfaceError::Lost) => {
                        game_engine.resize_window(game_engine.window_size());
                    }
                    // The system is out of memory, we should probably quit
                    Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                    // All other errors (Outdated, Timeout) should be resolved by the next frame
                    Err(e) => eprintln!("{:?}", e),
                }
            }
        }
        Event::WindowEvent {
            ref event,
            window_id,
        } if window_id == window.id() => {
            if !game_engine.input(event) {
                match event {
                    WindowEvent::Resized(physical_size) => {
                        game_engine.resize_window(*physical_size);
                    }
                    WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                        game_engine.resize_window(**new_inner_size);
                    }
                    WindowEvent::KeyboardInput { input, .. } => {
                        if let Some(virtual_keycode) = input.virtual_keycode {
                            game_engine.handle_virtual_keycode(virtual_keycode, input.state);
                        }
                    }
                    WindowEvent::CloseRequested
                    | WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                state: ElementState::Pressed,
                                virtual_keycode: Some(VirtualKeyCode::Escape),
                                ..
                            },
                        ..
                    } => *control_flow = ControlFlow::Exit,
                    _ => {}
                }
            }
        }

        _ => {}
    });

    Ok(())
}
