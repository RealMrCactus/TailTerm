use gtk::prelude::*;
use gtk::{Application, ApplicationWindow, TextView};

fn main() {
    // Initialize GTK application
    let application = Application::new(Some("com.example.myapp"), Default::default())
        .expect("Initialization failed...");

    application.connect_activate(|app| {
        // Create a new window
        let window = ApplicationWindow::new(app);
        window.set_title("TailTerm");
        window.set_default_size(350, 70);

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
    application.run(&[]);
}
