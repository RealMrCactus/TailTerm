use gtk::prelude::*;
use gtk::{Application, ApplicationWindow, TextView, TextBuffer, glib};
use glib::source;
use nix::pty::openpty;
use std::os::unix::io::{AsRawFd, FromRawFd, RawFd};
use std::{io::Read, thread, sync::mpsc};
use std::sync::{Arc, Mutex};

fn setup_pty_output_to_textview(master_fd: RawFd, text_view: TextView, tx: mpsc::Sender<String>) {
    thread::spawn(move || {
        // SAFETY: We're assuming here that we're the only ones who have access to this FD.
        let mut master_file = unsafe { std::fs::File::from_raw_fd(master_fd) };
        println!("Setup PTY: File descriptor is {:?}", master_file);

        let mut buffer = [0; 1024];
        loop {
            match master_file.read(&mut buffer) {
                Ok(size) => {
                    println!("Read {} bytes from PTY", size);
                    if size > 0 {
                        let output = String::from_utf8_lossy(&buffer[..size]).to_string();
                        if tx.send(output).is_err() {
                            println!("Failed to send output to main thread");
                            break;
                        }
                    }
                }
                Err(e) => {
                    // This will print the error whenever the read operation fails
                    println!("Error reading from PTY: {:?}", e);
                    break;
                }
            }
        }

        // Drop the master_file explicitly
        drop(master_file);
        println!("PTY master file descriptor closed.");
    });
}


fn main() {
    // Initialize GTK application
    let application = Application::new(Some("com.example.tailterm"), Default::default());
    let (tx, rx) = mpsc::channel();
    let rx = Arc::new(Mutex::new(rx)); // Wrap the receiver in an Arc<Mutex<_>>

    application.connect_activate(move |app| {
        let window = ApplicationWindow::new(app);
        window.set_title("TailTerm");
        window.set_default_size(850, 450);

        let text_view = TextView::new();
        text_view.set_editable(false);
        text_view.set_wrap_mode(gtk::WrapMode::Word);

        window.add(&text_view);
        window.show_all();
        
        let tx_clone = tx.clone(); // Clone the sender

        let pty_result = openpty(None, None);
        if let Ok(pty) = pty_result {
            println!("PTY opened successfully. Master FD: {:?}", pty.master);
            
            // This clone is necessary to keep the FD open in the reading thread
            let master_fd_clone = pty.master.try_clone().expect("Failed to clone PTY master FD");
            setup_pty_output_to_textview(master_fd_clone.as_raw_fd(), text_view.clone(), tx_clone);

            // You might need to handle the slave end here as well
            // For example, you might need to set up the slave end to act as a terminal
            // This would typically involve setting terminal attributes and possibly
            // spawning a shell or another process that uses the slave end as its terminal

        } else {
            eprintln!("Failed to open PTY: {:?}", pty_result.err());
        }

        let rx_clone = Arc::clone(&rx); // Clone the Arc<Mutex<Receiver<_>>>

        // Note: You need to pass a clone of the TextView's TextBuffer into the closure
        let text_buffer = text_view.buffer().unwrap(); // Get the buffer and unwrap it (ensure this is safe!)

        source::idle_add_local(move || {
            // Lock the Mutex and try to receive
            if let Ok(output) = rx_clone.lock().expect("Failed to lock rx").try_recv() {
                text_buffer.insert(&mut text_buffer.end_iter(), &output);
            }
            true.into() // Return Continue directly if that's what the API expects
        });
    });

    application.run();
}
