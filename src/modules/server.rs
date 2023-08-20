/* ********************************************************************************************************** */
/*                                                                                                            */
/*                                                     :::::::::  ::::::::   ::::::::   :::    ::: :::::::::: */
/* peer.rs                                            :+:    :+: :+:    :+: :+:    :+: :+:    :+: :+:         */
/*                                                   +:+    +:+ +:+    +:+ +:+        +:+    +:+ +:+          */
/* By: se-yukun <yukun@doche.io>                    +#+    +:+ +#+    +:+ +#+        +#++:++#++ +#++:++#      */
/*                                                 +#+    +#+ +#+    +#+ +#+        +#+    +#+ +#+            */
/* Created: 2023/08/18 02:58:51 by se-yukun       #+#    #+# #+#    #+# #+#    #+# #+#    #+# #+#             */
/* Updated: 2023/08/18 02:58:54 by se-yukun      #########  ########   ########  ###    ### ##########.io.    */
/*                                                                                                            */
/* ********************************************************************************************************** */

use std::net::SocketAddr;
use std::process::Command;
use std::sync::{mpsc, Arc};
use std::time::Duration;
use std::{env, process, thread};

use std::net::UdpSocket;

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

pub async fn server() {
    // Read Local & Remote IP from args
    let loc_address = env::args()
        .nth(2)
        .unwrap()
        .parse::<SocketAddr>()
        .unwrap_or_else(|err| {
            eprintln!("Unable to recognize listen ip: {}", err);
            process::exit(1);
        });

    // Create socket
    let socket = UdpSocket::bind(&loc_address).unwrap();
    let socket = Arc::new(socket);

    // Create interface
    let name = &env::args().nth(3).expect("Unable to read Interface name");
    let iface = Iface::new(&name, Mode::Tap).unwrap_or_else(|err| {
        eprintln!("Failed to configure the interface name: {}", err);
        process::exit(1);
    });
    let iface = Arc::new(iface);

    // Configure the „local“ (kernel) endpoint.
    let ip = &env::args()
        .nth(4)
        .expect("Unable to recognize remote interface IP");
    cmd("ip", &["addr", "add", "dev", iface.name(), &ip]);
    cmd("ip", &["link", "set", "up", "dev", iface.name()]);

    let iface = Arc::new(iface);
    let iface_reader = iface.clone();
    let socket_send = socket.clone();

    let (tx, rx) = mpsc::channel::<SocketAddr>();

    let reader = thread::spawn(move || {
        println!("r loaded");
        loop {
            let mut buf = vec![0; 1518];
            let len = match iface_reader.recv(&mut buf) {
                Ok(len) => len,
                Err(_) => continue,
            };
            match rx.try_recv() {
                Ok(addr) => {
                    println!("if recv");
                    if len > 0 {
                        match socket_send.send_to(&buf[..len], addr) {
                            Ok(x) => x,
                            Err(_) => 0,
                        };
                        println!("send: {:?}", len);
                    }
                }
                Err(mpsc::TryRecvError::Empty) => {}
                Err(mpsc::TryRecvError::Disconnected) => break,
            }
        }
        println!("r end");
    });
    loop {
        let iface_writer = iface.clone();
        let socket_recv = socket.clone();
        let mut buf = vec![0; 1];
        socket_recv.set_read_timeout(None).unwrap();
        let (_, addr) = socket.recv_from(&mut buf).unwrap();
        tx.send(addr).unwrap();
        let writer = thread::spawn(move || {
            println!("w loaded");
            socket_recv
                .set_read_timeout(Some(Duration::from_millis(1500)))
                .unwrap();
            loop {
                let mut buf = vec![0; 1518];
                let len = match socket_recv.recv(&mut buf) {
                    Ok(len) => len,
                    Err(_) => break,
                };
                if len > 0 {
                    iface_writer.send(&buf[..len]).unwrap();
                    println!("recv: {:?}", len);
                } else if len == 0 {
                    println!("keep")
                } else {
                    println!("receive invalid byte")
                }
            }
            println!("w end");
        });
        writer.join().unwrap();
        if reader.is_finished() {
            break;
        }
    }
}
