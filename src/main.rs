use gtk::prelude::*;
use gtk::{Application, ApplicationWindow, TextView, TextBuffer, glib};
use glib::source;
use nix::pty::openpty;
use std::os::unix::io::{AsRawFd, FromRawFd, RawFd};
use std::{io::Read, thread, sync::mpsc, };
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

        println!("Attempting to open PTY...");
        let pty_result = openpty(None, None);
        if let Ok(pty) = pty_result {
            println!("PTY opened successfully. Master FD: {:?}", pty.master);
            println!("Slave device path: /dev/pts/{}", pty.slave.as_raw_fd());

            // Here you can write something to the slave to test
            let mut slave_file = unsafe { std::fs::File::from_raw_fd(pty.slave.as_raw_fd()) };
            writeln!(slave_file, "Hello from the slave end!").unwrap();

            // Setup the PTY output to be displayed in the TextView
            setup_pty_output_to_textview(pty.master.as_raw_fd(), text_view.clone(), tx_clone);

            let rx_clone = Arc::clone(&rx); // Clone the Arc<Mutex<Receiver<_>>>
            let text_buffer = text_view.buffer().expect("Failed to get text buffer");

            source::idle_add_local(move || {
                if let Ok(output) = rx_clone.lock().expect("Failed to lock rx").try_recv() {
                    text_buffer.insert(&mut text_buffer.end_iter(), &output);
                }
                true.into()
            });
        } else {
            eprintln!("Failed to open PTY: {:?}", pty_result.err());
        }
    });

    application.run();
}

