use nix::pty::forkpty;
use nix::unistd::ForkResult;
use std::os::unix::io::RawFd;
use std::process::Command;

fn read_from_fd(fd: RawFd) -> Option<Vec<u8>> {
    unimplemented!()
}

unsafe fn spawn_pty_with_shell(default_shell: String) -> RawFd {
    match forkpty(None, None) {
        Ok(fork_pty_res) => {
            let stdout_fd = fork_pty_res.master; // primary
            if let ForkResult::Child = fork_pty_res.fork_result {
                // I'm the secondary part of the pty
                Command::new(&default_shell)
                    .spawn()
                    .expect("failed to spawn");
                std::thread::sleep(std::time::Duration::from_millis(2000));
                std::process::exit(0);
            }
            stdout_fd
        }
        Err(e) => {
            panic!("failed to fork {:?}", e);
        }
    }
}


fn main() {
    let default_shell = std::env::var("SHELL")
        .expect("could not find default shell from $SHELL");
    let stdout_fd = spawn_pty_with_shell(default_shell);
    let mut read_buffer = vec![];
    loop {
        match read_from_fd(stdout_fd) {
            Some(mut read_bytes) => {
                read_buffer.append(&mut read_bytes);
            }
            None => {
                println!("{:?}", String::from_utf8(read_buffer).unwrap());
                std::process::exit(0);
            }
        }
    }
}