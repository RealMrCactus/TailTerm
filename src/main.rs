use gtk::prelude::*;
use gtk::{Application, ApplicationWindow, TextView, TextBuffer, glib};
use glib::source;
use glib::ControlFlow::Continue;
use nix::pty::{forkpty, openpty, Winsize};
use std::os::unix::io::{AsRawFd, RawFd};
use std::{io::Read, thread};
use std::os::unix::io::FromRawFd;

fn setup_pty_output_to_textview(master_fd: RawFd, text_view: gtk::TextView) {
    thread::spawn(move || {
        let mut master_file = unsafe { std::fs::File::from_raw_fd(master_fd) };
        let mut buffer = [0; 1024];

        loop {
            match master_file.read(&mut buffer) {
                Ok(size) => {
                    if size > 0 {
                        let output = String::from_utf8_lossy(&buffer[..size]).to_string();

                        // Update the TextView in the main GTK thread
                        glib::idle_add_local(move || {
                            // Use of get_buffer method after ensuring the TextView's trait is in scope
                            let buffer = text_view.get_buffer().expect("Cannot get text buffer");
                            buffer.insert(&mut buffer.get_end_iter(), &output);
                            Continue(false)
                        });
                    }
                }
                Err(e) => {
                    eprintln!("Error reading from PTY: {}", e);
                    break;
                }
            }
        }
    });
}

fn main() {
    // Initialize GTK application
    let application = Application::new(Some("com.realmrcactus.tailterm"), Default::default());

    application.connect_activate(|app| {
        let window = ApplicationWindow::new(app);
        window.set_title("TailTerm");
        window.set_default_size(850, 450);

        let text_view = TextView::new();
        text_view.set_editable(false);
        text_view.set_wrap_mode(gtk::WrapMode::Word);

        window.add(&text_view);
        window.show_all();

        // Open PTY and setup output to textview
        if let Ok(pty) = openpty(None, None) {
            setup_pty_output_to_textview(pty.master.as_raw_fd(), text_view);
        }
    });

    application.run();
}
