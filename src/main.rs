// vim: ts=4 :

use std::net::{IpAddr, SocketAddr, UdpSocket};
use std::thread;
use std::process::Command;
use std::str::FromStr;

use notify_rust::Notification;
use serde::Deserialize;
use structopt::StructOpt;

#[derive(StructOpt)]
struct Cli {
    #[structopt(default_value = "1337", long, short="p", help = "Set port to bind")]
    port: u16,

    #[structopt(default_value = "0.0.0.0", long, help = "Set address to bind")]
    address: String,

    #[structopt(long, help = "Show notification server capabilities and exit")]
    capabilities: bool,
}

#[derive(Deserialize)]
struct NotificationRequest {
    summary: String,
    body: Option<String>,
    appname: Option<String>,
    icon: Option<String>,
    link: Option<String>,
}


fn main() -> std::io::Result<()> { 
    let args = Cli::from_args();

    if args.capabilities {
        let capabilities = notify_rust::get_capabilities().unwrap();
        println!("Notification capabilities: {:#?}", capabilities);
        return Ok(());
    }

    let ip = IpAddr::from_str(&args.address).expect("Unable to parse IP");

    let addr = SocketAddr::new(ip, args.port);

    eprintln!("Listening on {}", addr);
    
    let socket = UdpSocket::bind(addr).expect("Couldn't bind to address");
    
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
