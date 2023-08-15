use std::{process::Command, env};
use tun_tap::{Iface, Mode};

fn cmd(cmd: &str, args: &[&str]) {
    let command = Command::new(cmd)
        .args(args)
        .spawn()
        .unwrap()
        .wait()
        .unwrap();
    assert!(command.success(), "Failed to execute {}", cmd);
}

fn main() {
    let argv: Vec<String> = env::args().collect();
    assert!(argv.len() > 1, "Must set ip");

    // Create TAP interface
    let iface = Iface::new("tardis%d", Mode::Tap).unwrap();
    eprintln!("Iface: {:?}", iface);

    // Configure vpn interface
    cmd("ip", &["addr", "add", "dev", iface.name(), "172.16.42.2/24"]);
    cmd("ip", &["link", "set", "up", "dev", iface.name()]);


    let mut buffer = vec![0; 1504];
    loop {
        let size = iface.recv(&mut buffer).unwrap();
        assert!(size >= 4);
        println!("packet: {:?}", &buffer[4..size]);
    }
}
