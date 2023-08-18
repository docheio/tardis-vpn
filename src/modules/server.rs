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
use std::sync::Arc;
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
    loop {
        let iface_writer = Arc::clone(&iface);
        let iface_reader = Arc::clone(&iface);
        let socket_send = socket.clone();
        let socket_recv = socket.clone();
        let mut buf = vec![0; 1];
        socket_recv
            .set_read_timeout(None)
            .expect("set_read_timeout call failed");
        let (_, addr) = socket.recv_from(&mut buf).unwrap();

        socket_recv
            .set_read_timeout(Some(Duration::from_secs(5)))
            .expect("set_read_timeout call failed");
        let writer = thread::spawn(move || {
            println!("w loaded");
            loop {
                let mut buf = vec![0; 1600];
                let len = match socket_recv.recv(&mut buf) {
                    Ok(len) => len,
                    Err(_) => {
                        break;
                    }
                };
                println!("recv: {:?}", len);
                if len > 0 {
                    iface_writer.send(&buf[..len]).unwrap();
                }
            }
            println!("w end");
        });
        let reader = thread::spawn(move || {
            println!("r loaded");
            iface_reader.set_non_blocking().unwrap();
            loop {
                let mut buf = vec![0; 1600];
                if writer.is_finished() {
                    break;
                }
                let len = match iface_reader.recv(&mut buf) {
                    Ok(len) => len,
                    Err(_) => {
                        continue;
                    }
                };
                println!("if recv");
                if len > 0 {
                    match socket_send.send_to(&buf[..len], &addr) {
                        Ok(_) => {}
                        Err(_) => {
                            break;
                        }
                    };
                    println!("send: {:?}", len);
                }
            }
            println!("r end");
        });
        reader.join().unwrap();
    }
}
