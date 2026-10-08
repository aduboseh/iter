use std::io::{self, Read, Write};

// Only this fixture's stdout is closed; its process deliberately stays alive.
#[cfg(unix)]
fn close_stdout() {
    use std::os::fd::{AsRawFd, FromRawFd};
    drop(unsafe { std::fs::File::from_raw_fd(io::stdout().as_raw_fd()) });
}

#[cfg(windows)]
fn close_stdout() {
    use std::os::windows::io::{AsRawHandle, FromRawHandle};
    drop(unsafe { std::fs::File::from_raw_handle(io::stdout().as_raw_handle()) });
}

fn main() {
    let mode = std::env::args().nth(1).unwrap();
    if mode.ends_with("blocked-write") {
        io::stdin().read_exact(&mut [0]).unwrap();
    } else {
        let mut line = String::new();
        io::stdin().read_line(&mut line).unwrap();
    }
    if mode.ends_with("two-requests") {
        io::stdin().read_line(&mut String::new()).unwrap();
    }
    if mode.ends_with("silent") {
        loop {
            std::thread::park();
        }
    }
    if mode.ends_with("response") {
        println!(r#"{{"jsonrpc":"2.0","id":1,"result":{{"ok":true}}}}"#);
        io::stdout().flush().unwrap();
    }
    eprintln!("transport fixture diagnostic");
    if mode.ends_with("read-error") {
        io::stdout().write_all(b"\xff\n").unwrap();
        io::stdout().flush().unwrap();
    }
    close_stdout();
    if !mode.ends_with("exit") {
        loop {
            std::thread::park();
        }
    }
}
