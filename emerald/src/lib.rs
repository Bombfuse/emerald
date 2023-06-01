#![deny(future_incompatible, nonstandard_style)]

pub mod assets;
pub mod audio;
pub mod colors;
pub mod core;
pub mod input;
pub mod logging;
pub mod profiling;
pub mod rendering;
pub mod resources;
pub mod types;
pub mod world;

use crate::core::game_engine::{GameEngine, GameEngineContext};

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
use winit::event::Event;
use winit::event::WindowEvent;
use winit::event_loop::ControlFlow;
use winit::event_loop::EventLoop;
use winit::window::WindowBuilder;

pub use serde;
pub use serde_json;
pub use toml;

pub use wgpu::PresentMode;
pub use winit::window::CursorIcon;

#[cfg(feature = "gamepads")]
pub use gamepad;
#[cfg(feature = "gamepads")]
pub use gamepad::{Button, Joystick};
pub use rapier2d::{
    crossbeam,
    dynamics::{RigidBody, RigidBodyBuilder, RigidBodyHandle},
    geometry::{Collider, ColliderBuilder, ColliderHandle, InteractionGroups, Ray},
    na as nalgebra,
    na::Vector2,
    parry,
};

pub fn start(game: Box<dyn Game>, settings: GameSettings) {
    pollster::block_on(start_async(game, settings)).unwrap();
}

pub async fn start_async(game: Box<dyn Game>, settings: GameSettings) -> Result<(), EmeraldError> {
    run(game, settings).await
}

async fn run(game: Box<dyn Game>, settings: GameSettings) -> Result<(), EmeraldError> {
    env_logger::init();

    #[cfg(target_arch = "wasm32")]
    {
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));
        console_log::init_with_level(log::Level::Warn).expect("Couldn't initialize logger");
    }

    let fullscreen_option = if settings.render_settings.fullscreen {
        Some(winit::window::Fullscreen::Borderless(None))
    } else {
        None
    };
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_inner_size(PhysicalSize::new(
            settings.render_settings.resolution.0,
            settings.render_settings.resolution.1,
        ))
        .with_fullscreen(fullscreen_option)
        .build(&event_loop)
        .unwrap();
    let mut game_engine = GameEngine::new(game, &window, &settings).await?;

    #[cfg(target_arch = "wasm32")]
    {
        // Winit prevents sizing with CSS, so we have to set
        // the size manually when on web.
        use winit::dpi::PhysicalSize;
        window.set_inner_size(PhysicalSize::new(
            settings.render_settings.resolution.0,
            settings.render_settings.resolution.1,
        ));

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

    let mut ctx = GameEngineContext {
        window: Some(window),
        user_requesting_quit: false,
    };

    game_engine.initialize(&mut ctx)?;

    event_loop.run(move |event, _, control_flow| match event {
        Event::MainEventsCleared => {
            // RedrawRequested will only trigger once, unless we manually
            // request it.
            if let Some(window) = &mut ctx.window {
                window.request_redraw();
            }
        }
        Event::RedrawRequested(window_id) => {
            if let Some(w_id) = ctx.get_window_id() {
                if window_id == w_id {
                    {
                        let result = game_engine.update(&mut ctx);
                        if result.is_err() || ctx.user_requesting_quit {
                            *control_flow = ControlFlow::Exit;
                            return;
                        }
                    }
                    match game_engine.render(&mut ctx) {
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
        }
        Event::WindowEvent {
            ref event,
            window_id,
        } => {
            if let Some(w_id) = ctx.get_window_id() {
                if window_id == w_id {
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
                                    game_engine
                                        .handle_virtual_keycode(virtual_keycode, input.state);
                                }
                            }
                            WindowEvent::MouseInput { button, state, .. } => {
                                game_engine.handle_mouse_input(button, state);
                            }
                            WindowEvent::CursorMoved { position, .. } => {
                                game_engine.handle_cursor_move(position);
                            }
                            WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                            _ => {}
                        }
                    }
                }
            }
        }

        _ => {}
    });
}
