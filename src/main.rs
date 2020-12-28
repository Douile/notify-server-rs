// vim: ts=4 :

use std::net::{SocketAddr, UdpSocket};
use std::thread;
use std::process::Command;

use notify_rust::{Hint, Notification};
use serde::Deserialize;

#[derive(Deserialize)]
struct NotificationRequest {
    summary: String,
    body: Option<String>,
    appname: Option<String>,
    icon: Option<String>,
    link: Option<String>,
}


fn main() -> std::io::Result<()> {
    let capabilities = notify_rust::get_capabilities().unwrap();
    println!("Notification capabilities: {:#?}", capabilities);

    let addrs = [
        SocketAddr::from(([0,0,0,0], 1337)),
    ];
    
    let socket = UdpSocket::bind(&addrs[..]).expect("Couldn't bind to address");
    
    loop {
        let mut buf = [0;1024];
        let (number_of_bytes, src_addr) = socket.recv_from(&mut buf)
            .expect("No data");
        println!("Connection from {}", src_addr);
        thread::spawn(move || {
            let filled_buf = &mut buf[..number_of_bytes];
            let string = String::from_utf8_lossy(filled_buf);
            parse_notification(&string); 
        });
        
    }
}

fn parse_notification(string: &str) -> serde_json::Result<()> {
    let req: NotificationRequest = serde_json::from_str(&string)?;
    let mut noto = Notification::new();
    noto.summary(&req.summary);
    
    on_some(req.body, |b| {noto.body(&b);});
    on_some(req.appname, |n| {noto.appname(&n);});
    on_some(req.icon, |i| {noto.icon(&i);});
    if req.link.is_some() {
        noto.action("show", "Open link");
    }

    notify(&mut noto, req.link);

    Ok(())
}

fn notify(notification: &mut Notification, link: Option<String>) {
    match notification.show() {
        Ok(handle) => {
            let id = handle.id();
            println!("[{}] created", id);
            handle.wait_for_action(|action| {
                println!("[{}] action \"{}\"", id, action);
                match action {
                    "show" => {
                        on_some(link, |link| {
                            Command::new("firefox")
                                .arg(link)
                                .spawn()
                                .expect("Failed to open firefox");
                        });
                        // handle.close();
                    },
                    _ => {},
                };
            });
            /*handle.on_close(|| {
                println!("[{}] closed", id);
            });*/
        },
        Err(e) => {eprintln!("{}", e)},
    };
}

fn on_some<U>(o: Option<U>, f: impl FnOnce(U)) {
    match o {
        Some(v) => f(v),
        None => {},
    };
}

/*
    Notification::new()
        .summary("Hello")
        .body("Hello world")
        .show()?;
}*/