use inotify::{
    Inotify,
    WatchMask,
};
use std::{
    net::{TcpListener, TcpStream},
    thread::spawn,
};

use log::*;
use tungstenite::{accept, handshake::HandshakeRole, Error, HandshakeError, Message, Result};


fn must_not_block<Role: HandshakeRole>(err: HandshakeError<Role>) -> Error {
    match err {
        HandshakeError::Interrupted(_) => panic!("Bug: blocking socket would block"),
        HandshakeError::Failure(f) => f,
    }
}


fn handle_client(stream: TcpStream) -> Result<()> {
    let mut inotify = Inotify::init().expect("INOTIFY FAILED");
    let mut socket = accept(stream).map_err(must_not_block)?;

    println!("{:?}", socket);
    if let Some(home_path_buf) = std::env::home_dir() {
        let mut buffer = [0;1024];
        loop {
            inotify 
                .watches()
                .add(
                    format!("{}/WSLOCK", home_path_buf.display()),
                    WatchMask::MODIFY 
                )
                .expect("WATHCMASK FAILED");


            let events = inotify.read_events_blocking(&mut buffer).expect("reading events error");
        
            let fc = std::fs::read_to_string(format!("{}/WSLOCK", home_path_buf.display()))
                .expect("failed to read from WSLOCK");
        
            let lastline = fc
                .split("\n")
                .filter(|line| !line.is_empty())
                .last()
                .expect("No lines in file");
        
            let sid = lastline.split_whitespace().next().unwrap_or("").to_string();
            socket.send(lastline.into())?;
        }
    }
    Ok(())
}


fn main() {
    env_logger::init();

    let server = TcpListener::bind("127.0.0.1:9002").unwrap();

    for stream in server.incoming() {
        spawn(move || match stream {
            Ok(stream) => {
                if let Err(err) = handle_client(stream) {
                    match err {
                        Error::ConnectionClosed | Error::Protocol(_) | Error::Utf8(_) => (),
                        e => error!("test: {}", e),
                    }
                }
            }
            Err(e) => error!("Error accepting stream: {}", e),
        });
    }
}
