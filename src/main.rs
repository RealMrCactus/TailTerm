extern crate gtk;

use gtk::prelude::*;
use gtk::{Application, ApplicationWindow, TextView};
use glib::source;
use glib::ControlFlow::Continue;
use nix::pty::openpty;
use std::os::unix::io::{AsRawFd, FromRawFd, RawFd};
use std::{thread, sync::mpsc};
use std::io::{Write, Read};
use std::sync::{Arc, Mutex};
use std::os::fd::IntoRawFd;

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
    let application = Application::new(Some("com.realmrcactus.TailTerm"), Default::default())
    .unwrap_or_else(|| panic!("Initialization failed..."));

        application.connect_activate(|app| {
            let window = ApplicationWindow::new(app);
            window.set_title("PTY Terminal");
            window.set_default_size(800, 600);
    
            let text_view = TextView::new();
            text_view.set_editable(false);
            window.add(&text_view);
        
            
            window.show_all();

        // Open PTY and handle any errors that might occur
        let pty = match openpty(None, None) {
            Ok(pty) => pty,
            Err(e) => {
                eprintln!("Failed to open PTY: {:?}", e);
                return;
            }
        };

        // Extract master and slave file descriptors
        let master_fd = pty.master.as_raw_fd();
        let slave_fd = pty.slave.as_raw_fd();

        println!("Master FD: {}", master_fd);
        println!("Slave FD: {}", slave_fd);

        // Wrap the master file descriptor in a safe File handle
        let mut master_file = unsafe { std::fs::File::from_raw_fd(master_fd) };


        // Write to the master end of the PTY
        let write_result = writeln!(master_file, "Hello PTY");
        match write_result {
            Ok(_) => println!("Successfully wrote to PTY master"),
            Err(e) => eprintln!("Failed to write to PTY master: {:?}", e),
        }

        // Read from the master end of the PTY
        let mut buffer = [0; 1024];
        loop {
            match master_file.read(&mut buffer) {
                Ok(0) => {
                    println!("Reached EOF on PTY master");
                    break;
                }
                Ok(size) => {
                    let output = String::from_utf8_lossy(&buffer[..size]);
                    println!("Read {} bytes from PTY master: {}", size, output);
                }
                Err(e) => {
                    eprintln!("Error reading from PTY master: {:?}", e);
                    break;
                }
            }
        }

        // Wrap the slave file descriptor in a safe File handle
        // Note: This is just an example. Normally, you should not open slave_fd here since it is used by the child process.
        let mut slave_file = unsafe { std::fs::File::from_raw_fd(slave_fd) };

        // Write something to the slave end to simulate input (for testing purposes)
        let write_result = writeln!(slave_file, "Input to slave");
        match write_result {
            Ok(_) => println!("Successfully wrote to PTY slave"),
            Err(e) => eprintln!("Failed to write to PTY slave: {:?}", e),
        }

        let context = glib::MainContext::default();
        context.acquire().unwrap();
        gtk::idle_add(move || {
            if let Ok(output) = rx.try_recv() {
                text_view.get_buffer().unwrap().insert_at_cursor(&output);
            }
            Continue(true)
        });
    });
    application.run();
}