use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;

use rdev::{listen, Event, EventType, Key};

/// Events emitted by the input listener.
#[derive(Debug, Clone)]
pub enum InputEvent {
    LeftClick { x: f64, y: f64 },
    RightClick { x: f64, y: f64 },
    KeyEnter,
    KeyPress(char),
}

/// Start a global input listener on a background thread.
/// Returns a receiver for input events.
pub fn start_listener() -> mpsc::Receiver<InputEvent> {
    let (tx, rx) = mpsc::channel();
    // Track mouse position since ButtonPress doesn't include coordinates
    let mouse_pos = Arc::new(Mutex::new((0.0f64, 0.0f64)));

    let mouse_pos_clone = mouse_pos.clone();
    thread::spawn(move || {
        listen(move |event: Event| {
            let input_event = match event.event_type {
                EventType::MouseMove { x, y } => {
                    if let Ok(mut pos) = mouse_pos_clone.lock() {
                        *pos = (x, y);
                    }
                    None
                }
                EventType::ButtonPress(rdev::Button::Left) => {
                    let (x, y) = *mouse_pos_clone.lock().unwrap_or_else(|e| e.into_inner());
                    Some(InputEvent::LeftClick { x, y })
                }
                EventType::ButtonPress(rdev::Button::Right) => {
                    let (x, y) = *mouse_pos_clone.lock().unwrap_or_else(|e| e.into_inner());
                    Some(InputEvent::RightClick { x, y })
                }
                EventType::KeyPress(Key::Return) => Some(InputEvent::KeyEnter),
                EventType::KeyPress(key) => key_to_char(key).map(InputEvent::KeyPress),
                _ => None,
            };

            if let Some(evt) = input_event {
                let _ = tx.send(evt);
            }
        })
        .expect("Failed to start input listener");
    });

    rx
}

fn key_to_char(key: Key) -> Option<char> {
    match key {
        Key::KeyA => Some('a'),
        Key::KeyB => Some('b'),
        Key::KeyC => Some('c'),
        Key::KeyD => Some('d'),
        Key::KeyE => Some('e'),
        Key::KeyF => Some('f'),
        Key::KeyG => Some('g'),
        Key::KeyH => Some('h'),
        Key::KeyI => Some('i'),
        Key::KeyJ => Some('j'),
        Key::KeyK => Some('k'),
        Key::KeyL => Some('l'),
        Key::KeyM => Some('m'),
        Key::KeyN => Some('n'),
        Key::KeyO => Some('o'),
        Key::KeyP => Some('p'),
        Key::KeyQ => Some('q'),
        Key::KeyR => Some('r'),
        Key::KeyS => Some('s'),
        Key::KeyT => Some('t'),
        Key::KeyU => Some('u'),
        Key::KeyV => Some('v'),
        Key::KeyW => Some('w'),
        Key::KeyX => Some('x'),
        Key::KeyY => Some('y'),
        Key::KeyZ => Some('z'),
        Key::Num0 => Some('0'),
        Key::Num1 => Some('1'),
        Key::Num2 => Some('2'),
        Key::Num3 => Some('3'),
        Key::Num4 => Some('4'),
        Key::Num5 => Some('5'),
        Key::Num6 => Some('6'),
        Key::Num7 => Some('7'),
        Key::Num8 => Some('8'),
        Key::Num9 => Some('9'),
        Key::Space => Some(' '),
        Key::Minus => Some('-'),
        Key::Equal => Some('='),
        Key::LeftBracket => Some('['),
        Key::RightBracket => Some(']'),
        Key::SemiColon => Some(';'),
        Key::Quote => Some('\''),
        Key::Comma => Some(','),
        Key::Dot => Some('.'),
        Key::Slash => Some('/'),
        Key::BackSlash => Some('\\'),
        _ => None,
    }
}
