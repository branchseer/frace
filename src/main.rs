use std::io;
use std::sync::Arc;

use bstr::BStr;
use libc::c_int;
use std::{fs, os::unix::net::UnixDatagram, process::Command, thread::spawn};
use tempfile::tempdir;

fn main() {
    let z = include_bytes!(concat!(env!("OUT_DIR"), "/frace_preload.dylib"));
    let dir = tempdir().expect("Failed create temporary directory for preload dylib");
    let preload_dylib_path = dir.path().join("frace_preload.dylib");
    fs::write(&preload_dylib_path, z).expect("Failed to create preload dylib");

    let unix_socket_path = dir.path().join("1.unix");

    let unix_datagram = Arc::new(UnixDatagram::bind(&unix_socket_path).unwrap());
    dbg!(&unix_socket_path);

    let handle = spawn({
        let unix_datagram = Arc::clone(&unix_datagram);
        move || -> io::Result<()> {
            let mut buf = [0; 16384];
            loop {
                let n = unix_datagram.recv(&mut buf).unwrap();
                if n == 0 {
                    break Ok(());
                }
                let flags = c_int::from_ne_bytes(buf[..size_of::<c_int>()].try_into().unwrap());
                let path = BStr::new(&buf[size_of::<c_int>()..n]);
                println!("{} - {}", path, flags);
            }
        }
    });
    let status = Command::new("/Users/chiwang/.local/share/mise/installs/node/22.14.0/bin/node")
        .arg("/Users/chiwang/code/frace/src/macos/index.mjs")
        .env("DYLD_INSERT_LIBRARIES", preload_dylib_path)
        .env("FRACE_IPC", &unix_socket_path)
        .status()
        .unwrap();

    dbg!(status.success());
    dir.close().unwrap();
    // unsafe { libc::close(unix_datagram.as_raw_fd()) };
    handle.join().unwrap().unwrap();
}
