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

use std::io::Result;
use std::net::SocketAddr;
use std::process::Command;
use std::sync::Arc;
use std::{env, process};

use tokio_core::net::{UdpCodec, UdpSocket};
use tokio_core::reactor::Core;

use tun_tap::{Iface, Mode};

struct VecCodec(SocketAddr);

impl UdpCodec for VecCodec {
    type In = Vec<u8>;
    type Out = Vec<u8>;
    fn decode(&mut self, _src: &SocketAddr, buf: &[u8]) -> Result<Self::In> {
        Ok(buf.to_owned())
    }
    fn encode(&mut self, msg: Self::Out, buf: &mut Vec<u8>) -> SocketAddr {
        buf.extend(&msg);
        self.0
    }
}

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
    let core = Core::new().unwrap();

    // Read Local & Remote IP from args
    let loc_address = env::args().nth(2).unwrap().parse().unwrap_or_else(|err| {
        eprintln!("Unable to recognize listen ip: {}", err);
        process::exit(1);
    });

    // Create socket
    let socket = UdpSocket::bind(&loc_address, &core.handle()).unwrap();
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
    let socket = Arc::new(socket);
    let socket_send = socket.clone();
    let socket_recv = socket.clone();

    let mut buf = vec![0; 1500];
    let (_, addr) = socket.recv_from(&mut buf).unwrap();

    let writer = tokio::spawn(async move {
        loop {
            let mut buf = vec![0; 1500];
            let (len, _) = socket_recv.recv_from(&mut buf).unwrap();
            println!("recv: {:?}", len);
            iface_writer.send(&buf[..len]).unwrap();
        }
    });
    let reader = tokio::spawn(async move {
        loop {
            let mut buf = vec![0; 1504];
            let len = iface_reader.recv(&mut buf).unwrap();
            if len > 0 {
                socket_send.send_to(&buf[..len], &addr).unwrap();
                println!("send: {:?}", len);
            }
        }
    });
    writer.await.unwrap();
    reader.await.unwrap();
}
