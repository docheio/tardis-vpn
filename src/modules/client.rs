/* ********************************************************************************************************** */
/*                                                                                                            */
/*                                                     :::::::::  ::::::::   ::::::::   :::    ::: :::::::::: */
/* client.rs                                          :+:    :+: :+:    :+: :+:    :+: :+:    :+: :+:         */
/*                                                   +:+    +:+ +:+    +:+ +:+        +:+    +:+ +:+          */
/* By: se-yukun <yukun@doche.io>                    +#+    +:+ +#+    +:+ +#+        +#++:++#++ +#++:++#      */
/*                                                 +#+    +#+ +#+    +#+ +#+        +#+    +#+ +#+            */
/* Created: 2023/08/18 02:58:41 by se-yukun       #+#    #+# #+#    #+# #+#    #+# #+#    #+# #+#             */
/* Updated: 2023/08/18 02:58:44 by se-yukun      #########  ########   ########  ###    ### ##########.io.    */
/*                                                                                                            */
/* ********************************************************************************************************** */

use std::net::SocketAddr;
use std::process::Command;
use std::sync::Arc;
use std::time::Duration;
use std::{env, process, thread};
use tokio;

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
    let socket = UdpSocket::bind(&loc_address).await.unwrap();
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
    let iface_writer = Arc::clone(&iface);
    let iface_reader = Arc::clone(&iface);
    let socket_keep = socket.clone();
    let socket_send = socket.clone();
    let socket_recv = socket.clone();

    socket.connect(&rem_address).await.unwrap();
    let buf = vec![0; 1];
    socket.send(&buf).await.unwrap();

    let keeper = tokio::task::spawn(async move {
        println!("k loaded");
        loop {
            let buf = vec![0; 0];
            match socket_keep.send(&buf).await {
                Ok(_) => {}
                Err(_) => break,
            };
            println!("send: keep");
            tokio::time::sleep(Duration::from_millis(1000)).await;
        }
        println!("k end")
    });
    let writer = tokio::task::spawn(async move {
        println!("w loaded");
        iface_writer.set_non_blocking().unwrap();
        loop {
            let mut buf = vec![0; 1518];
            if keeper.is_finished() {
                break;
            }
            let len = socket_recv.recv(&mut buf).await.unwrap();
            if len > 0 {
                iface_writer.send(&buf[..len]).unwrap();
                println!("recv: {:?}", len);
            } else if len == 0 {
                println!("keep")
            } else {
                println!("receive invalid byte")
            }
            println!("recv: {:?}", len);
        }
        println!("w end");
    });
    let reader = tokio::task::spawn(async move {
        println!("r loaded");
        loop {
            let mut buf = vec![0; 1518];
            let len = match iface_reader.recv(&mut buf) {
                Ok(len) => len,
                Err(_) => continue,
            };
            if len > 0 {
                socket_send.send(&buf[..len]).await.unwrap();
                println!("send: {:?}", len);
            }
        }
    });
    loop {
        if writer.is_finished() {
            println!("closed");
            reader.abort();
        }
        thread::sleep(Duration::from_secs(1));
    }
}
