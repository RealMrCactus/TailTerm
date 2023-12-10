use gtk::prelude::*;
use gtk::{Application, ApplicationWindow, TextView, TextBuffer, glib};
use glib::source;
use nix::pty::openpty;
use std::os::unix::io::{AsRawFd, FromRawFd, RawFd};
use std::{io::Read, thread, sync::mpsc};
use std::io::Write;
use std::sync::{Arc, Mutex};

fn setup_pty_output_to_textview(master_fd: RawFd, text_view: TextView, tx: mpsc::Sender<String>) {
    println!("setup_pty_output_to_textview: Setting up PTY output thread...");

    thread::spawn(move || {
        println!("Thread started. Master FD: {:?}", master_fd);

        // SAFETY: We're assuming here that we're the only ones who have access to this FD.
        let mut master_file = unsafe { std::fs::File::from_raw_fd(master_fd) };
        println!("Thread: File descriptor is now wrapped in std::fs::File");

        let mut buffer = [0; 1024];
        loop {
            match master_file.read(&mut buffer) {
                Ok(size) => {
                    if size == 0 {
                        println!("Thread: Read 0 bytes from PTY (EOF)");
                        break; // EOF reached
                    }
                    println!("Thread: Read {} bytes from PTY", size);
                    let output = String::from_utf8_lossy(&buffer[..size]).to_string();
                    if tx.send(output).is_err() {
                        println!("Thread: Failed to send output to main thread");
                        break;
                    }
                }
                Err(e) => {
                    println!("Thread: Error reading from PTY: {:?}", e);
                    break;
                }
            }
        }

        println!("Thread: Exiting PTY output thread.");
    });

    println!("setup_pty_output_to_textview: PTY output thread setup complete.");
}



fn main() {
    let application = Application::new(Some("com.example.tailterm"), Default::default());

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
            let master_fd = pty.master.into_raw_fd();
            let master_file = unsafe { std::fs::File::from_raw_fd(master_fd) };

            // Create an Arc and Mutex to share the master file between threads
            let master_file = Arc::new(Mutex::new(master_file));

            // Clone the Arc to share with the thread
            let thread_master_file = Arc::clone(&master_file);
            thread::spawn(move || {
                let mut local_master_file = thread_master_file.lock().unwrap();
                let mut buffer = [0; 1024];
                loop {
                    match local_master_file.read(&mut buffer) {
                        Ok(size) if size > 0 => {
                            let output = String::from_utf8_lossy(&buffer[..size]).to_string();
                            println!("Read from PTY: {}", output);
                        }
                        Ok(_) => {}
                        Err(e) => {
                            eprintln!("Error reading from PTY: {}", e);
                            break;
                        }
                    }
                }
            });

            // Example of writing to the PTY master end in the main thread
            if let Ok(mut master_file) = master_file.lock() {
                writeln!(master_file, "Hello from the main thread!").unwrap();
            }
        } else {
            eprintln!("Failed to open PTY");
        }
    });

    application.run();
}

