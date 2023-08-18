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
    // let iface_writer = Arc::clone(&iface);
    // let iface_reader = Arc::clone(&iface);
    let socket = Arc::new(socket);
    // let socket_send = socket.clone();
    // let socket_recv = socket.clone();

    // let mut buf = vec![0; 1500];
    // let (_, addr) = socket.recv_from(&mut buf).await.unwrap();
    loop {
        let mut buf = vec![0; 1500];
        let (len, addr) = socket.recv_from(&mut buf).await.unwrap();
        println!("recv size: {:?}", addr);
        socket.send_to(&buf[0..len], &addr).await.unwrap();
        println!("send");
        socket.send_to(&buf[0..len], &addr).await.unwrap();
        println!("send");
    }

    // let writer = tokio::spawn(async move {
    //     println!("loaded");
    //     loop {
    //         let mut buf = vec![0; 1500];
    //         let len = socket_recv.recv(&mut buf).await.unwrap();
    //         iface_writer.send(&buf[..len]).unwrap();
    //     }
    // });
    // let reader = tokio::spawn(async move {
    //     loop {
    //         let mut buf = vec![0; 1504];
    //         let len = iface_reader.recv(&mut buf).unwrap();
    //         if len > 0 {
    //             socket_send.send_to(&buf[..len], &addr).await.unwrap();
    //             println!("send: {:?}", len);
    //         }
    //     }
    // });
    // writer.await.unwrap();
    // reader.await.unwrap();
}
