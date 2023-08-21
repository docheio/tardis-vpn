/* ********************************************************************************************************** */
/*                                                                                                            */
/*                                                     :::::::::  ::::::::   ::::::::   :::    ::: :::::::::: */
/* client.rs                                          :+:    :+: :+:    :+: :+:    :+: :+:    :+: :+:         */
/*                                                   +:+    +:+ +:+    +:+ +:+        +:+    +:+ +:+          */
/* By: codespace <marvin@doche.io>                  +#+    +:+ +#+    +:+ +#+        +#++:++#++ +#++:++#      */
/*                                                 +#+    +#+ +#+    +#+ +#+        +#+    +#+ +#+            */
/* Created: 2023/08/21 04:56:09 by codespace      #+#    #+# #+#    #+# #+#    #+# #+#    #+# #+#             */
/* Updated: 2023/08/21 04:56:11 by codespace     #########  ########   ########   ###    ### ##########.io    */
/*                                                                                                            */
/* ********************************************************************************************************** */

use std::net::SocketAddr;
use std::process::Command;
use std::sync::Arc;
use std::time::Duration;
use std::{env, process, thread};

use std::net::UdpSocket;

use tun_tap::{Iface, Mode};

pub async fn client() {
    // Read Local & Remote IP from args
    let loc_address = "0.0.0.0:0".parse::<SocketAddr>().unwrap_or_else(|err| {
        eprintln!("Unable to bind udp socket: {}", err);
        process::exit(1);
    });
    let rem_address = env::args()
        .nth(2)
        .unwrap()
        .parse::<SocketAddr>()
        .unwrap_or_else(|err| {
            eprintln!("Unable to recognize listen ip: {}", err);
            process::exit(1);
        });

    // Create socket
    let socket = UdpSocket::bind(&loc_address).unwrap();

    // Create interface
    let name = &env::args().nth(3).expect("Unable to read Interface name");
    let iface = Iface::new(&name, Mode::Tap).unwrap_or_else(|err| {
        eprintln!("Failed to configure the interface name: {}", err);
        process::exit(1);
    });
    let iface = Arc::new(iface);

    // Configure the „local“ (kernel) endpoint.
    let ip = match env::args().nth(4) {
        Some(s) => s,
        None => {
            eprintln!("ERROR: {:?}", "Unable to read IP");
            process::exit(1);
        }
    };
    match Command::new("ip")
        .args(["addr", "add", "dev", iface.name(), &ip])
        .spawn()
        .unwrap()
        .wait()
    {
        Ok(s) => s,
        Err(e) => {
            eprintln!("ERROR: {:?}", e);
            process::exit(1);
        }
    };
    match Command::new("ip")
        .args(["link", "set", "up", "dev", iface.name()])
        .spawn()
        .unwrap()
        .wait()
    {
        Ok(s) => s,
        Err(e) => {
            eprintln!("ERROR: {:?}", e);
            process::exit(1);
        }
    };

    let iface = Arc::new(iface);
    let socket = Arc::new(socket);

    match socket.connect(&rem_address) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("ERROR: {:?}", e);
            process::exit(1);
        }
    };
    match socket.send(&vec![0; 1]) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("ERROR: {:?}", e);
            process::exit(1);
        }
    };

    let keeper = thread::spawn({
        let socket_keep = socket.clone();
        move || loop {
            let buf = vec![0; 0];
            thread::sleep(Duration::from_millis(1000));
            match socket_keep.send(&buf) {
                Ok(_) => {}
                Err(_) => break,
            };
        }
    });
    let writer = thread::spawn({
        let iface_writer = Arc::clone(&iface);
        let socket_recv = socket.clone();
        move || loop {
            let mut buf = vec![0; 1518];
            if keeper.is_finished() {
                break;
            }
            let len = match socket_recv.recv(&mut buf) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("ERROR: {:?}", e);
                    continue;
                }
            };
            match iface_writer.send(&buf[..len]) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("ERROR: {:?}", e);
                    continue;
                }
            };
        }
    });
    let reader = thread::spawn({
        let iface_reader = Arc::clone(&iface);
        let socket_send = socket.clone();
        move || loop {
            let mut buf = vec![0; 1518];
            if writer.is_finished() {
                break;
            }
            let len = match iface_reader.recv(&mut buf) {
                Ok(len) => len,
                Err(e) => {
                    eprintln!("ERROR: {:?}", e);
                    process::exit(1);
                }
            };
            if len > 0 {
                match socket_send.send(&buf[..len]) {
                    Ok(s) => s,
                    Err(e) => {
                        eprintln!("ERROR: {:?}", e);
                        process::exit(1);
                    }
                };
            }
        }
    });
    reader.join().unwrap();
}
