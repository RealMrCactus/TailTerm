use gtk::prelude::*;
use gtk::{Application, ApplicationWindow, TextView};
use nix::pty::{openpty, Winsize};
use std::os::unix::io::AsRawFd;
use std::thread;

fn opty() {
    // Attempt to open a new PTY
    let pty_result = openpty(None, None);

    match pty_result {
        Ok(pty) => {
            println!("PTY opened successfully.");
            println!("Master FD: {}", pty.master.as_raw_fd());
            println!("Slave FD: {}", pty.slave.as_raw_fd());

            // The PTY handling would go here.
            // You might read from the master FD and update the TextView in your GTK app,
            // or take input from the TextView and write it to the master FD.
        }
        Err(e) => {
            eprintln!("Failed to open PTY: {}", e);
        }
    }
}

fn main() {
    // Initialize GTK application
    if let Ok(application) = Application::new(Some("com.realmrcactus.tailterm"), Default::default()) {
        application.connect_activate(|app| {
            // Create a new window
            let window = ApplicationWindow::new(app);
            window.set_title("TailTerm");
            window.set_default_size(850, 450);

            // Create a text view (a multi-line text box)
            let text_view = TextView::new();
            text_view.set_editable(true); // Make the text view editable
            text_view.set_wrap_mode(gtk::WrapMode::Word); // Set wrap mode

            // Add the text view to the window
            window.add(&text_view);

            // Show all window components
            window.show_all();
        });

        // Start the PTY handling in a separate thread
        thread::spawn(move || {
            opty();
        });

        // Run the application
        application.run();
    } else {
        eprintln!("Failed to initialize GTK application");
    }
}
