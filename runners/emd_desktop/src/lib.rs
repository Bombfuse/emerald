mod audio;
mod input;
mod rendering;

use audio::audio_engine::DesktopAudioEngine;
use emerald::core::game_engine::GameEngine;
use emerald::game_engine::GameEngineContext;
use emerald::rendering_engine::ScreenSize;
use emerald::{AssetEngine, Game, GameSettings, KeyCode, MouseButton, Vector2};
use input::input_engine::DesktopInputEngine;
use rendering::rendering_engine::DesktopRenderingEngine;
use winit::dpi::PhysicalSize;
use winit::event::{Event, VirtualKeyCode, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;

pub fn start(game: Box<dyn Game>) {
    let settings = GameSettings::default();
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

    let mut asset_engine = AssetEngine::new();
    let rendering_engine = Box::new(
        DesktopRenderingEngine::new(&window, settings.render_settings.clone(), &mut asset_engine)
            .unwrap(),
    );
    let mut game_engine = GameEngine::new(
        game,
        rendering_engine,
        Box::new(DesktopAudioEngine {}),
        Box::new(DesktopInputEngine::new()),
        asset_engine,
        &settings,
    )
    .unwrap();
    game_engine.initialize(&mut ctx).unwrap();

    event_loop.run(move |event, _, control_flow| match event {
        Event::MainEventsCleared => {
            // RedrawRequested will only trigger once, unless we manually
            // request it.
            window.request_redraw();
        }
        Event::RedrawRequested(window_id) => {
            if window_id == window.id() {
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
                    // Err(wgpu::SurfaceError::Lost) => {
                    //     game_engine.resize_window(game_engine.window_size());
                    // }
                    // // The system is out of memory, we should probably quit
                    // Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                    // All other errors (Outdated, Timeout) should be resolved by the next frame
                    Err(e) => eprintln!("{:?}", e),
                }
            }
        }
        Event::WindowEvent {
            ref event,
            window_id,
        } => {
            if window_id == window.id() {
                match event {
                    WindowEvent::Resized(physical_size) => {
                        game_engine
                            .rendering_engine
                            .handle_window_resize(ScreenSize {
                                width: physical_size.width,
                                height: physical_size.height,
                            });
                    }
                    WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                        game_engine
                            .rendering_engine
                            .handle_window_resize(ScreenSize {
                                width: new_inner_size.width,
                                height: new_inner_size.height,
                            });
                    }
                    WindowEvent::KeyboardInput { input, .. } => {
                        if let Some(virtual_keycode) = input.virtual_keycode {
                            let key_code = virtual_keycode_to_keycode(virtual_keycode);
                            game_engine.input_engine.handle_key_input(key_code);
                        }
                    }
                    WindowEvent::MouseInput { button, state, .. } => {
                        let button = match button {
                            winit::event::MouseButton::Left => MouseButton::Left,
                            winit::event::MouseButton::Right => MouseButton::Right,
                            winit::event::MouseButton::Middle => MouseButton::Middle,
                            winit::event::MouseButton::Other(id) => MouseButton::Other(*id),
                        };
                        let is_pressed = match state {
                            winit::event::ElementState::Pressed => true,
                            winit::event::ElementState::Released => false,
                        };
                        game_engine
                            .input_engine
                            .handle_mouse_input(button, is_pressed);
                    }
                    WindowEvent::CursorMoved { position, .. } => {
                        game_engine
                            .input_engine
                            .handle_cursor_move(Vector2::new(position.x as f32, position.y as f32));
                    }
                    WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                    _ => {}
                }
            }
        }

        _ => {}
    });
}

pub(crate) fn virtual_keycode_to_keycode(virtual_keycode: VirtualKeyCode) -> KeyCode {
    match virtual_keycode {
        VirtualKeyCode::A => KeyCode::A,
        VirtualKeyCode::B => KeyCode::B,
        VirtualKeyCode::C => KeyCode::C,
        VirtualKeyCode::D => KeyCode::D,
        VirtualKeyCode::E => KeyCode::E,
        VirtualKeyCode::F => KeyCode::F,
        VirtualKeyCode::G => KeyCode::G,
        VirtualKeyCode::H => KeyCode::H,
        VirtualKeyCode::I => KeyCode::I,
        VirtualKeyCode::J => KeyCode::J,
        VirtualKeyCode::K => KeyCode::K,
        VirtualKeyCode::L => KeyCode::L,
        VirtualKeyCode::M => KeyCode::M,
        VirtualKeyCode::N => KeyCode::N,
        VirtualKeyCode::O => KeyCode::O,
        VirtualKeyCode::P => KeyCode::P,
        VirtualKeyCode::Q => KeyCode::Q,
        VirtualKeyCode::R => KeyCode::R,
        VirtualKeyCode::S => KeyCode::S,
        VirtualKeyCode::T => KeyCode::T,
        VirtualKeyCode::U => KeyCode::U,
        VirtualKeyCode::V => KeyCode::V,
        VirtualKeyCode::W => KeyCode::W,
        VirtualKeyCode::X => KeyCode::X,
        VirtualKeyCode::Y => KeyCode::Y,
        VirtualKeyCode::Z => KeyCode::Z,
        VirtualKeyCode::Down => KeyCode::Down,
        VirtualKeyCode::Up => KeyCode::Up,
        VirtualKeyCode::Left => KeyCode::Left,
        VirtualKeyCode::Right => KeyCode::Right,
        VirtualKeyCode::Space => KeyCode::Space,
        VirtualKeyCode::Escape => KeyCode::Escape,
        VirtualKeyCode::Delete => KeyCode::Delete,
        VirtualKeyCode::Apostrophe => KeyCode::Apostrophe,
        VirtualKeyCode::Comma => KeyCode::Comma,
        VirtualKeyCode::Minus => KeyCode::Minus,
        VirtualKeyCode::Period => KeyCode::Period,
        VirtualKeyCode::Slash => KeyCode::Slash,
        VirtualKeyCode::Key0 => KeyCode::Key0,
        VirtualKeyCode::Key1 => KeyCode::Key1,
        VirtualKeyCode::Key2 => KeyCode::Key2,
        VirtualKeyCode::Key3 => KeyCode::Key3,
        VirtualKeyCode::Key4 => KeyCode::Key4,
        VirtualKeyCode::Key5 => KeyCode::Key5,
        VirtualKeyCode::Key6 => KeyCode::Key6,
        VirtualKeyCode::Key7 => KeyCode::Key7,
        VirtualKeyCode::Key8 => KeyCode::Key8,
        VirtualKeyCode::Key9 => KeyCode::Key9,
        VirtualKeyCode::Semicolon => KeyCode::Semicolon,
        VirtualKeyCode::Equals => KeyCode::Equal,
        VirtualKeyCode::LBracket => KeyCode::LeftBracket,
        VirtualKeyCode::Backslash => KeyCode::Backslash,
        VirtualKeyCode::RBracket => KeyCode::RightBracket,
        VirtualKeyCode::Grave => KeyCode::GraveAccent,
        VirtualKeyCode::NumpadEnter => KeyCode::KpEnter,
        VirtualKeyCode::Return => KeyCode::Enter,
        VirtualKeyCode::Tab => KeyCode::Tab,
        VirtualKeyCode::Back => KeyCode::Backspace,
        VirtualKeyCode::Insert => KeyCode::Insert,
        VirtualKeyCode::PageUp => KeyCode::PageUp,
        VirtualKeyCode::PageDown => KeyCode::PageDown,
        VirtualKeyCode::Home => KeyCode::Home,
        VirtualKeyCode::End => KeyCode::End,
        VirtualKeyCode::Capital => KeyCode::CapsLock,
        VirtualKeyCode::Scroll => KeyCode::ScrollLock,
        VirtualKeyCode::Numlock => KeyCode::NumLock,
        VirtualKeyCode::Pause => KeyCode::Pause,
        VirtualKeyCode::F1 => KeyCode::F1,
        VirtualKeyCode::F2 => KeyCode::F2,
        VirtualKeyCode::F3 => KeyCode::F3,
        VirtualKeyCode::F4 => KeyCode::F4,
        VirtualKeyCode::F5 => KeyCode::F5,
        VirtualKeyCode::F6 => KeyCode::F6,
        VirtualKeyCode::F7 => KeyCode::F7,
        VirtualKeyCode::F8 => KeyCode::F8,
        VirtualKeyCode::F9 => KeyCode::F9,
        VirtualKeyCode::F10 => KeyCode::F10,
        VirtualKeyCode::F11 => KeyCode::F11,
        VirtualKeyCode::F12 => KeyCode::F12,
        VirtualKeyCode::F13 => KeyCode::F13,
        VirtualKeyCode::F14 => KeyCode::F14,
        VirtualKeyCode::F15 => KeyCode::F15,
        VirtualKeyCode::F16 => KeyCode::F16,
        VirtualKeyCode::F17 => KeyCode::F17,
        VirtualKeyCode::F18 => KeyCode::F18,
        VirtualKeyCode::F19 => KeyCode::F19,
        VirtualKeyCode::F20 => KeyCode::F20,
        VirtualKeyCode::F21 => KeyCode::F21,
        VirtualKeyCode::F22 => KeyCode::F22,
        VirtualKeyCode::F23 => KeyCode::F23,
        VirtualKeyCode::F24 => KeyCode::F24,
        VirtualKeyCode::Numpad0 => KeyCode::Kp0,
        VirtualKeyCode::Numpad1 => KeyCode::Kp1,
        VirtualKeyCode::Numpad2 => KeyCode::Kp2,
        VirtualKeyCode::Numpad3 => KeyCode::Kp3,
        VirtualKeyCode::Numpad4 => KeyCode::Kp4,
        VirtualKeyCode::Numpad5 => KeyCode::Kp5,
        VirtualKeyCode::Numpad6 => KeyCode::Kp6,
        VirtualKeyCode::Numpad7 => KeyCode::Kp7,
        VirtualKeyCode::Numpad8 => KeyCode::Kp8,
        VirtualKeyCode::Numpad9 => KeyCode::Kp9,
        VirtualKeyCode::NumpadDecimal => KeyCode::KpDecimal,
        VirtualKeyCode::NumpadDivide => KeyCode::KpDivide,
        VirtualKeyCode::NumpadMultiply => KeyCode::KpMultiply,
        VirtualKeyCode::NumpadSubtract => KeyCode::KpSubtract,
        VirtualKeyCode::NumpadAdd => KeyCode::KpAdd,
        VirtualKeyCode::NumpadEquals => KeyCode::KpEqual,
        VirtualKeyCode::LShift => KeyCode::LeftShift,
        VirtualKeyCode::LControl => KeyCode::LeftControl,
        VirtualKeyCode::LAlt => KeyCode::LeftAlt,
        VirtualKeyCode::RShift => KeyCode::RightShift,
        VirtualKeyCode::RControl => KeyCode::RightControl,
        VirtualKeyCode::RAlt => KeyCode::RightAlt,
        _ => KeyCode::Unknown,
    }
}
