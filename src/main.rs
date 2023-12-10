use qmetaobject::*;
use nix::fcntl::OFlag;
use nix::pty::{openpty, OpenptyResult, Winsize, PtyMaster};
use nix::sys::termios;
use nix::unistd::{fork, ForkResult, setsid, dup2, execvp, close};
use std::ffi::{CString, CStr};
use std::os::unix::io::{RawFd, AsRawFd};
use libc::{grantpt as other_grantpt, unlockpt as other_unlockpt};
use std::os::fd::IntoRawFd;
use std::os::fd::RawFd;
use std::os::fd::FromRawFd;
use std::io::Read;
use std::io::Write;
use std::thread;
use std::fs::File;
use std::sync::{Arc, Mutex};

#[derive(QObject, Default)]
struct TerminalWindow {
    base: qt_base_class!(trait QObject),
    terminal_text: qt_property!(QString; NOTIFY terminal_text_changed),
    terminal_text_changed: qt_signal!(),
}

impl TerminalWindow {
    fn show(&self) {
        let mut engine = QmlEngine::new();
        let qml_data = QByteArray::from(r#"
            import QtQuick 2.0
            import QtQuick.Controls 2.15
    
            ApplicationWindow {
                visible: true
                width: 640
                height: 480
                title: qsTr("TailTerm")
            }
        "#.as_bytes());
        engine.load_data(qml_data);
    }

    fn write_to_terminal(&mut self, data: &[u8]) {
        let qstring = QString::from_utf8(data.to_vec());
        self.terminal_text.append(&qstring);
        self.terminal_text_changed();
    }
}

fn spawn_shell(terminal_window: Arc<Mutex<TerminalWindow>>) -> nix::Result<()> {
    let OpenptyResult { master, slave } = openpty(None, None)?;
    let master_file = unsafe { File::from_raw_fd(master.into_raw_fd()) };

    match unsafe { fork()? } {
        ForkResult::Parent { .. } => {
            thread::spawn(move || {
                let mut buffer = [0; 1024];
                loop {
                    match master_file.read(&mut buffer) {
                        Ok(n) => {
                            if let Ok(mut tw) = terminal_window.lock() {
                                tw.write_to_terminal(&buffer[..n]);
                            }
                        }
                        Err(e) => {
                            eprintln!("Error reading from master PTY: {}", e);
                            break;
                        }
                    }
                }
            });
        }
        ForkResult::Child => {
            setsid()?;
            let slave_fd = slave.into_raw_fd();
            dup2(slave_fd, std::io::stdin().as_raw_fd())?;
            dup2(slave_fd, std::io::stdout().as_raw_fd())?;
            dup2(slave_fd, std::io::stderr().as_raw_fd())?;
            close(slave_fd)?;
            let shell = CString::new("/bin/sh").unwrap();
            let args = [CStr::from_bytes_with_nul(b"/bin/sh\0").unwrap()];
            execvp(&shell, &args)?;
        }
    }

    Ok(())
}

fn main() {
    let terminal_window = Arc::new(Mutex::new(TerminalWindow::default()));

    match spawn_shell(terminal_window.clone()) {
        Ok(_) => {
            // Note: In a real application, you'll need to manage both the Qt event loop and the PTY I/O.
            // Qt's event loop can be started by calling `exec` on `QCoreApplication` or `QApplication`.
            terminal_window.lock().unwrap().show();
        }
        Err(e) => {
            eprintln!("Error spawning shell: {}", e);
            std::process::exit(1);
        }
    }
}
