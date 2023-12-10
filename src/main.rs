extern crate gtk;
extern crate vte;

use gtk::prelude::*;
use gtk::{Application, ApplicationWindow};
use vte::Terminal;
use gtk::InputPurpose::Terminal;

fn main() {
    // Initialize GTK application
    let application = Application::new(
        Some("com.example.gtk_vte_terminal"),
        Default::default(),
    ).expect("Initialization failed...");

    application.connect_activate(|app| {
        // Create a window
        let window = ApplicationWindow::new(app);
        window.set_title("Rust Terminal Emulator");
        window.set_default_size(800, 600);

        // Create a VTE Terminal
        let terminal = Terminal::new();

        // Optional: Set up callback for child-exited signal
        terminal.connect_child_exited(move |_| {
            std::process::exit(0);
        });

        // Spawn a new shell process
        terminal.spawn_async(
            &vte::PtyFlags::default(), // Default PTY flags
            None,                      // Working directory (None uses the current directory)
            &[],                       // Command to run (empty array runs the default shell)
            None,                      // Environment variables
            &[],                       // Spawn flags
            0,                         // Timeout
            None::<&gio::Cancellable>, // Cancellable object
            None,                      // Callback (None means no callback)
        );

        window.add(&terminal);
        window.show_all();
    });

    application.run(&[]);
}
