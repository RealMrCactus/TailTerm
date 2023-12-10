use qt_core::{QString, Slot};
use qt_widgets::{QApplication, QLineEdit, QPushButton, QVBoxLayout, QWidget};
use std::process::Command;
use std::os::unix::process::CommandExt;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::process::Command;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let shell = "/bin/bash"; // This should be replaced with code that finds the user's shell

    let mut child = Command::new(shell)
        .spawn()
        .expect("failed to spawn shell");

    QApplication::init(|_| unsafe {
        let widget = QWidget::new_0a();
        let layout = QVBoxLayout::new_1a(&widget);
        let line_edit = QLineEdit::new();
        let button = QPushButton::from_q_string(&QString::from_std_str("Send"));

        layout.add_widget(&line_edit);
        layout.add_widget(&button);

        let slot = Slot::new(move || {
            let command = line_edit.text().to_std_string() + "\n";
            child.stdin.as_mut().unwrap().write_all(command.as_bytes()).await.unwrap();
        });

        button.released().connect(&slot);

        widget.show();
        QApplication::exec()
    })
}