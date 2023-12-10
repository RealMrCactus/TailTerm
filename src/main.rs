extern crate gtk;
extern crate vte;

use gtk::prelude::*;
use gtk::{Application, ApplicationWindow};
use vte::Terminal;

fn main() {
    let application = Application::new(
        Some("com.example.gtk_vte_terminal"),
        Default::default(),
    ).expect("Failed to initialize GTK application");

    application.connect_activate(|app| {
        let window = ApplicationWindow::new(app);
        window.set_title("Simple Terminal");
        window.set_default_size(800, 600);

        let terminal = Terminal::new();
        terminal.spawn_async(&vte::PtyFlags::default(), None, &[], None, &[], 0, None::<&gio::Cancellable>, None);

        window.add(&terminal);
        window.show_all();
    });

    application.run(&[]);
}
