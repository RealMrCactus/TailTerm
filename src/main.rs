use qmetaobject::*;
use nix::fcntl::OFlag;
use nix::pty::{openpty, OpenptyResult, Winsize, PtyMaster};
use nix::sys::termios;
use nix::unistd::{fork, ForkResult, setsid, dup2, execvp, close};
use std::ffi::{CString, CStr};
use std::os::unix::io::{RawFd, AsRawFd};
use libc::{grantpt as other_grantpt, unlockpt as other_unlockpt};
use std::os::fd::IntoRawFd;
use std::thread;
use std::io::Read;


#[derive(QObject, Default)]
struct TerminalWindow {
    base: qt_base_class!(trait QObject),
}

impl TerminalWindow {
    fn show(&self) {
        let mut engine = QmlEngine::new(); // Declare `engine` as mutable
        engine.load_data(r#"
            import QtQuick 2.0
            import QtQuick.Controls 2.15
    
            ApplicationWindow {
                visible: true
                width: 640
                height: 480
                title: qsTr("Rust Terminal Emulator")
    
                TextArea {
                    anchors.fill: parent
                    font.family: "monospace"
                    // ... additional properties ...
                }
            }
        "#.into());
    
        engine.exec();
    }
    
}

fn spawn_shell(terminal_window: TerminalWindow) -> nix::Result<()> {
    let OpenptyResult { master, slave } = openpty(None, None)?;

    match unsafe { fork()? } {
        ForkResult::Parent { .. } => {
            // Spawn a new thread to handle the I/O
            thread::spawn(move || {
                let mut buffer = [0; 1024];
                loop {
                    match master.read(&mut buffer) {
                        Ok(n) => {
                            // Write the data to the terminal window
                            terminal_window.write_to_terminal(&buffer[..n]);
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

            // Attach the slave end of the PTY to the standard streams
            dup2(slave_fd, std::io::stdin().as_raw_fd())?;
            dup2(slave_fd, std::io::stdout().as_raw_fd())?;
            dup2(slave_fd, std::io::stderr().as_raw_fd())?;

            // Now close the original slave_fd
            close(slave_fd)?;

            // Prepare command and arguments
            let shell = CString::new("/bin/sh").unwrap();
            let args = [CStr::from_bytes_with_nul(b"/bin/sh\0").unwrap()];
            
            // Execute the shell
            execvp(&shell, &args)?;
        }
    }

    Ok(())
}

fn main() {
    let terminal_window = TerminalWindow::default();

    match spawn_shell(terminal_window) {
        Ok(_) => {
            // Note: In a real application, you'll need to find a way to manage 
            //       both the Qt event loop and the PTY I/O simultaneously.
            terminal_window.show();
        }
        Err(e) => {
            eprintln!("Error spawning shell: {}", e);
            std::process::exit(1);
        }
    }
}