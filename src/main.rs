/* ********************************************************************************************************** */
/*                                                                                                            */
/*                                                     :::::::::  ::::::::   ::::::::   :::    ::: :::::::::: */
/* main.rs                                            :+:    :+: :+:    :+: :+:    :+: :+:    :+: :+:         */
/*                                                   +:+    +:+ +:+    +:+ +:+        +:+    +:+ +:+          */
/* By: se-yukun <yukun@doche.io>                    +#+    +:+ +#+    +:+ +#+        +#++:++#++ +#++:++#      */
/*                                                 +#+    +#+ +#+    +#+ +#+        +#+    +#+ +#+            */
/* Created: 2023/08/16 21:18:22 by se-yukun       #+#    #+# #+#    #+# #+#    #+# #+#    #+# #+#             */
/* Updated: 2023/08/16 21:18:27 by se-yukun      #########  ########   ########  ###    ### ##########.io.    */
/*                                                                                                            */
/* ********************************************************************************************************** */

use std::io::Result;
use std::net::{IpAddr, SocketAddr};
use std::process::Command;
use std::str::FromStr;
use std::{env, process};

use futures::{Future, Stream};
use tokio_core::net::{UdpCodec, UdpSocket};
use tokio_core::reactor::Core;

use tun_tap::asynclib::Async;
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

async fn server() {
    // Read Local & Remote IP from args
    let loc_address_str = env::args().nth(2).expect("Unable to recognize listen IP");
    let loc_address = loc_address_str
        .parse::<SocketAddr>()
        .expect("Unable to recognize listen IP");

    // Create socket
    let mut core = Core::new().unwrap();
    let socket = UdpSocket::bind(&loc_address, &core.handle());
}

fn client() {}

#[tokio::main]
async fn main() {
    let mode = env::args().nth(2).expect("Unable to select mode");
    if mode == "server" {
        server().await;
    } else if mode == "client" {
        client();
    } else {
        eprintln!("Unable to select mode");
    }
}
