use gtk::prelude::*;
use gtk::{Application, ApplicationWindow, TextView};
use nix::pty::{forkpty, openpty, ptsname, Winsize};
use std::os::unix::io::AsRawFd;

fn opty() {
    // Attempt to open a new PTY
    let pty_result = openpty(None, None);

    match pty_result {
        Ok(pty) => {
            println!("PTY opened successfully.");
            println!("Master FD: {}", pty.master.as_raw_fd());
            println!("Slave FD: {}", pty.slave.as_raw_fd());

            // You can now read from and write to these file descriptors.
            // Typically, you might fork the process here, with the child
            // process becoming a session leader and attaching the slave
            // end of the PTY to its standard streams.
        }
        Err(e) => {
            eprintln!("Failed to open PTY: {}", e);
        }
    }
}


fn main() {
    // Initialize GTK application
    let application = Application::new(Some("com.realmrcactus.tailterm"), Default::default());

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

    // Run the application
    application.run();
}
