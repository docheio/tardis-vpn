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
use std::{env, process};

use tokio_core::net::UdpSocket;
use tokio_core::reactor::Core;

use super::peer::ft_peer;

pub async fn client() {
    // Read Remote IP from args
    let rem_address = env::args()
        .nth(2)
        .unwrap()
        .parse::<SocketAddr>()
        .unwrap_or_else(|err| {
            eprintln!("Unable to recognize remote ip: {}", err);
            process::exit(1);
        });

    // Allocate an available ip:port
    let loc_address = "0.0.0.0:0".parse::<SocketAddr>().unwrap_or_else(|err| {
        eprintln!("Unable to bind udp socket: {}", err);
        process::exit(1);
    });

    // Create socket
    let core = Core::new().unwrap();
    let socket = UdpSocket::bind(&loc_address, &core.handle()).unwrap();

    // iface
    let name = &env::args().nth(3).expect("Unable to read Interface name");

    // Read the „local“ (kernel) endpoint ip.
    let ip = &env::args()
        .nth(4)
        .expect("Unable to recognize remote interface IP");

    // Handshake
    let buf = [0; 1500];
    socket.connect(&rem_address).unwrap_or_else(|err| {
        eprintln!("Unable to connect to server: {}", err);
        process::exit(1);
    });
    socket.send(&buf).unwrap();
    ft_peer(socket, &rem_address, &name, &ip).await;
}
