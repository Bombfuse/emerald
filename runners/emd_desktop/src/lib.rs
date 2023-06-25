mod audio;
mod input;
mod rendering;

use audio::audio_engine::DesktopAudioEngine;
use emerald::core::game_engine::GameEngine;
use emerald::game_engine::GameEngineContext;
use emerald::{Game, GameSettings};
use input::input_engine::DesktopInputEngine;
use rendering::rendering_engine::DesktopRenderingEngine;
use winit::dpi::PhysicalSize;
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;

pub fn start(game: Box<dyn Game>) {
    let settings = GameSettings::default();
    let mut game_engine = GameEngine::new(
        game,
        Box::new(DesktopRenderingEngine::new()),
        Box::new(DesktopAudioEngine {}),
        Box::new(DesktopInputEngine::new()),
        &settings,
    )
    .unwrap();

    let mut ctx = GameEngineContext {
        user_requesting_quit: false,
    };

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

    event_loop.run(move |event, _, control_flow| match event {
        Event::MainEventsCleared => {
            // RedrawRequested will only trigger once, unless we manually
            // request it.
            // if let Some(window) = &mut ctx.window {
            //     window.request_redraw();
            // }
        }
        Event::RedrawRequested(window_id) => {
            // if let Some(w_id) = ctx.get_window_id() {
            //     if window_id == w_id {
            //         {
            //             let result = game_engine.update(&mut ctx);
            //             if result.is_err() || ctx.user_requesting_quit {
            //                 *control_flow = ControlFlow::Exit;
            //                 return;
            //             }
            //         }
            //         match game_engine.render(&mut ctx) {
            //             Ok(_) => {}
            //             // Reconfigure the surface if lost
            //             // Err(wgpu::SurfaceError::Lost) => {
            //             //     game_engine.resize_window(game_engine.window_size());
            //             // }
            //             // // The system is out of memory, we should probably quit
            //             // Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
            //             // All other errors (Outdated, Timeout) should be resolved by the next frame
            //             Err(e) => eprintln!("{:?}", e),
            //         }
            //     }
            // }
        }
        Event::WindowEvent {
            ref event,
            window_id,
        } => {
            // if let Some(w_id) = ctx.get_window_id() {
            //     if window_id == w_id {
            //         if !game_engine.input(event) {
            //             match event {
            //                 WindowEvent::Resized(physical_size) => {}
            //                 WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {}
            //                 WindowEvent::KeyboardInput { input, .. } => {
            //                     if let Some(virtual_keycode) = input.virtual_keycode {}
            //                 }
            //                 WindowEvent::MouseInput { button, state, .. } => {}
            //                 WindowEvent::CursorMoved { position, .. } => {}
            //                 WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
            //                 _ => {}
            //             }
            //         }
            //     }
            // }
        }

        _ => {}
    });
}
