/* ********************************************************************************************************** */
/*                                                                                                            */
/*                                                     :::::::::  ::::::::   ::::::::   :::    ::: :::::::::: */
/* server.rs                                          :+:    :+: :+:    :+: :+:    :+: :+:    :+: :+:         */
/*                                                   +:+    +:+ +:+    +:+ +:+        +:+    +:+ +:+          */
/* By: se-yukun <yukun@doche.io>                    +#+    +:+ +#+    +:+ +#+        +#++:++#++ +#++:++#      */
/*                                                 +#+    +#+ +#+    +#+ +#+        +#+    +#+ +#+            */
/* Created: 2023/08/18 02:58:57 by se-yukun       #+#    #+# #+#    #+# #+#    #+# #+#    #+# #+#             */
/* Updated: 2023/08/18 02:58:58 by se-yukun      #########  ########   ########  ###    ### ##########.io.    */
/*                                                                                                            */
/* ********************************************************************************************************** */

use std::net::SocketAddr;
use std::process::Command;
use std::sync::Arc;
use std::env;

use anyhow::Ok;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
// use tokio::net::UdpSocket;
use udp_stream::UdpListener;

use tun_tap::{Iface, Mode};

fn cmd(cmd: &str, args: &[&str]) -> anyhow::Result<()> {
    let ecode = Command::new(cmd).args(args).spawn()?.wait()?;
    assert!(ecode.success(), "Failed to execte {}", cmd);
    Ok(())
}

pub async fn server() -> anyhow::Result<()> {
    // Read Local & Remote IP from args
    let loc_address = env::args().nth(2).unwrap().parse::<SocketAddr>()?;

    // Create socket
    // let socket = UdpSocket::bind(&loc_address).await.unwrap_or_else(|err| {
    //     eprintln!("Unable to bind udp socket: {}", err);
    //     process::exit(1);
    // });
    let listener = UdpListener::bind(loc_address).await?;

    // Create interface
    let name = &env::args().nth(3).unwrap();
    let iface = Iface::new(&name, Mode::Tap)?;

    // Configure the „local“ (kernel) endpoint.
    let ip = &env::args()
        .nth(4)
        .expect("Unable to recognize remote interface IP");
    cmd("ip", &["addr", "add", "dev", iface.name(), &ip])?;
    cmd("ip", &["link", "set", "up", "dev", iface.name()])?;

    // Handshake

    let (mut stream, _) = listener.accept().await?;
    let iface = Arc::new(iface);
    let iface_recv = iface.clone();
    let iface_send = iface.clone();

    tokio::spawn(async move {
        let mut buf = vec![0; 1504];
        loop {
            let len = iface_recv.recv(&mut buf).unwrap();
            stream.write(&buf[..len]).await.unwrap();
            let len = stream.read(&mut buf).await.unwrap();
            iface_send.send(&mut buf[..len]).unwrap();
        }
    })
    .await?;
    Ok(())

    // let mut buf = [0; 1504];
    // loop {d
    //     let (len, addr) = socket.recv_from(&mut buf).await?;
    //     println!("recv: {:?}", len);
    //     iface.send(&buf[..len])?;
    //     let len = iface.recv(&mut buf)?;
    //     if len > 0 {
    //         socket.send_to(&buf[..len], addr).await?;
    //         println!("send: {:?}", len);
    //     }
    // }
}
