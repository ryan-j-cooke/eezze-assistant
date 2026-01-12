use fltk::{
    app,
    button::Button,
    enums::{Color, Event, FrameType},
    prelude::*,
    window::Window,
};
use serde::Deserialize;
use std::cell::RefCell;
use std::io::{BufRead, BufReader};
use std::net::TcpListener;
use std::rc::Rc;
use std::sync::mpsc;
use std::thread;

#[derive(Debug, Deserialize)]
struct SocketMessage {
    #[serde(default)]
    event: Option<serde_json::Value>,
    #[serde(default)]
    data: serde_json::Value,
}

fn start_socket_server(tx: mpsc::Sender<SocketMessage>) {
    thread::spawn(move || {
        let listener = TcpListener::bind("0.0.0.0:3015")
            .expect("Failed to bind socket server on port 3015");

        println!("Socket server listening on 0.0.0.0:3015");

        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    let tx_client = tx.clone();
                    thread::spawn(move || {
                        let reader = BufReader::new(stream);
                        for line in reader.lines() {
                            match line {
                                Ok(raw) => {
                                    let trimmed = raw.trim();
                                    if trimmed.is_empty() {
                                        continue;
                                    }

                                    match serde_json::from_str::<SocketMessage>(trimmed) {
                                        Ok(msg) => {
                                            println!("[socket] received: {:?}", msg);
                                            let _ = tx_client.send(msg);
                                        }
                                        Err(err) => {
                                            eprintln!(
                                                "[socket] invalid message, expected {{\"event\": ..., \"data\": ...}}: {}",
                                                err
                                            );
                                        }
                                    }
                                }
                                Err(err) => {
                                    eprintln!("[socket] read error: {}", err);
                                    break;
                                }
                            }
                        }
                    });
                }
                Err(err) => {
                    eprintln!("[socket] incoming connection error: {}", err);
                }
            }
        }
    });
}

fn main() {
    let app = app::App::default();

    // Position the window as a small floating widget on the right-center
    let (sw, sh) = app::screen_size();
    let widget_w = 100;
    let widget_h = 100;
    let margin = 40;
    let x = sw as i32 - widget_w - margin;
    let y = sh as i32 / 2 - widget_h / 2;

    // Frameless, small window acting as the round widget container
    let mut wind = Window::new(x, y, widget_w, widget_h, "");
    wind.set_color(Color::from_rgb(15, 23, 42));
    wind.set_frame(FrameType::FlatBox);
    wind.make_resizable(false);
    wind.top_window();
    wind.set_override(); // remove decorations

    // Circular-looking button centered in the window (acts as the "bubble")
    let mut btn = Button::new(10, 10, widget_w - 20, widget_h - 20, "⚡");
    btn.set_color(Color::from_rgb(79, 70, 229));
    btn.set_selection_color(Color::from_rgb(129, 140, 248));
    btn.set_frame(FrameType::RoundUpBox);
    btn.set_label_size(24);
    btn.clear_visible_focus();

    btn.set_callback(|_| {
        println!("Widget clicked");
    });

    // Shared drag state: previous mouse position, to move window by deltas
    let drag_state: Rc<RefCell<Option<(i32, i32)>>> = Rc::new(RefCell::new(None));

    // Make the whole widget (including the button) draggable
    let mut wind_for_btn = wind.clone();
    let drag_state_btn = drag_state.clone();
    btn.handle(move |_, ev| match ev {
        Event::Push => {
            let (mx, my) = app::event_coords();
            *drag_state_btn.borrow_mut() = Some((mx, my));
            true
        }
        Event::Drag => {
            let (mx, my) = app::event_coords();
            let (dx, dy) = {
                let mut state = drag_state_btn.borrow_mut();
                if let Some((px, py)) = *state {
                    let dx = mx - px;
                    let dy = my - py;
                    *state = Some((mx, my));
                    (dx, dy)
                } else {
                    *state = Some((mx, my));
                    (0, 0)
                }
            };

            if dx != 0 || dy != 0 {
                wind_for_btn.set_pos(wind_for_btn.x() + dx, wind_for_btn.y() + dy);
            }
            true
        }
        Event::Released => {
            *drag_state_btn.borrow_mut() = None;
            true
        }
        _ => false,
    });

    wind.end();
    wind.show();

    // Make the window draggable (like a floating chat-head)
    let drag_state_win = drag_state.clone();
    wind.handle(move |w, ev| match ev {
        Event::Push => {
            let (mx, my) = app::event_coords();
            *drag_state_win.borrow_mut() = Some((mx, my));
            true
        }
        Event::Drag => {
            let (mx, my) = app::event_coords();
            let (dx, dy) = {
                let mut state = drag_state_win.borrow_mut();
                if let Some((px, py)) = *state {
                    let dx = mx - px;
                    let dy = my - py;
                    *state = Some((mx, my));
                    (dx, dy)
                } else {
                    *state = Some((mx, my));
                    (0, 0)
                }
            };

            if dx != 0 || dy != 0 {
                w.set_pos(w.x() + dx, w.y() + dy);
            }
            true
        }
        Event::Released => {
            *drag_state_win.borrow_mut() = None;
            true
        }
        _ => false,
    });

    // Start socket server and connect it to the UI via a channel
    let (tx, rx) = mpsc::channel::<SocketMessage>();
    start_socket_server(tx);

    // Clone widget handles for use inside the idle callback
    let mut btn_clone = btn.clone();

    // On each incoming message, visually "bounce" / flash the widget
    app::add_idle(move || {
        while let Ok(_msg) = rx.try_recv() {
            // Simple feedback animation: temporarily change color & label size
            btn_clone.set_color(Color::from_rgb(129, 140, 248));
            btn_clone.set_label_size(28);
            btn_clone.redraw();

            let mut inner_btn = btn_clone.clone();
            app::add_timeout(0.20, move || {
                inner_btn.set_color(Color::from_rgb(79, 70, 229));
                inner_btn.set_label_size(24);
                inner_btn.redraw();
            });
        }
    });

    app.run().unwrap();
}
