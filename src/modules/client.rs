use std::process::Command;
use std::{env, process};

use tokio::net::UdpSocket;

use tun_tap::{Iface, Mode};

fn cmd(cmd: &str, args: &[&str]) {
    let ecode = Command::new(cmd)
        .args(args)
        .spawn()
        .unwrap()
        .wait()
        .unwrap();
    assert!(ecode.success(), "Failed to execte {}", cmd);
}

pub async fn client() {
    // Read Local & Remote IP from args
    let loc_address = env::args().nth(2).expect("Unable to recognize listen IP");
    let rem_address = env::args().nth(3).expect("Unable to recognize listen IP");

    // Create interface
    let name = &env::args()
        .nth(4)
        .expect("Unable to configure the interface name");
    let iface = Iface::new(&name, Mode::Tap).unwrap_or_else(|err| {
        eprintln!("Failed to configure the interface: {}", err);
        process::exit(1);
    });

    // Configure the „local“ (kernel) endpoint.
    let ip = &env::args()
        .nth(5)
        .expect("Unable to recognize remote interface IP");
    cmd("ip", &["addr", "add", "dev", iface.name(), &ip]);
    cmd("ip", &["link", "set", "up", "dev", iface.name()]);

    // Create socket
    let socket = UdpSocket::bind(&loc_address).await.unwrap_or_else(|err| {
        eprintln!("Failed to open socket: {}", err);
        process::exit(1);
    });

    // Handshake
    socket.connect(&rem_address).await.unwrap();
    loop {
        let mut buf = vec![0; 1504];
        let len = socket.recv(&mut buf).await.unwrap();
        println!("{:?} bytes received from {:?}", len, rem_address);
        let len = socket.send(&buf[..len]).await.unwrap();
        println!("{:?} bytes sent", len);
    }
}
