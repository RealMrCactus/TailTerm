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

        // Open PTY and setup output to textview
        if let Ok(pty) = openpty(None, None) {
            setup_pty_output_to_textview(pty.master.as_raw_fd(), text_view.clone(), tx_clone);
        } else {
            eprintln!("Failed to open PTY");
        }

        let rx_clone = Arc::clone(&rx); // Clone the Arc<Mutex<Receiver<_>>>

        // Note: You need to pass a clone of the TextView's TextBuffer into the closure
        let text_buffer = text_view.buffer().unwrap(); // Get the buffer and unwrap it (ensure this is safe!)

        source::idle_add_local(move || {
            // Lock the Mutex and try to receive
            if let Ok(output) = rx_clone.lock().expect("Failed to lock rx").try_recv() {
                text_buffer.insert(&mut text_buffer.end_iter(), &output);
            }
            glib::Continue(true) // Return Continue directly if that's what the API expects
        });
    });

    application.run();
}
