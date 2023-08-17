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

async fn udp_to_iface(socket: &UdpSocket, iface: &Iface) {
    let mut buf = vec![0; 1500];
    let len = socket.recv(&mut buf).await.unwrap();
    println!("{:?}", buf.clone());
    let _ = iface.send(&mut buf[..len]);
}

async fn udp_to_iface_loop(socket: &UdpSocket, iface: &Iface) {
    loop {
        udp_to_iface(&socket, &iface).await;
    }
}

async fn iface_to_udp(socket: &UdpSocket, iface: &Iface) {
    let mut buf = vec![0; 1504];
    let len = iface.recv(&mut buf).unwrap();
    println!("{:?}", buf.clone());
    if len > 4 {
        let _ = socket.send(&mut buf[4..len]);
    }
}

async fn iface_to_udp_loop(socket: &UdpSocket, iface: &Iface) {
    loop {
        iface_to_udp(&socket, &iface).await;
    }
}

async fn udp_connecter(socket: &UdpSocket, iface: &Iface, rem_address: &String) {
    socket.connect(&rem_address).await.unwrap();
    let _ = iface_to_udp_loop(&socket, &iface);
    let _ = udp_to_iface_loop(&socket, &iface);
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
    udp_connecter(&socket, &iface, &rem_address).await;
    loop {}
}
