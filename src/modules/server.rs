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
use std::{env, process};

use tokio_core::net::UdpSocket;
use tokio_core::reactor::Core;

use super::peer::ft_peer;

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
    let core = Core::new().unwrap();
    let socket = UdpSocket::bind(&loc_address, &core.handle()).unwrap_or_else(|err| {
        eprintln!("Unable to bind udp socket: {}", err);
        process::exit(1);
    });

    // Read interface name
    let name = &env::args().nth(3).expect("Unable to read Interface name");

    // Read the „local“ (kernel) endpoint ip.
    let ip = &env::args()
        .nth(4)
        .expect("Unable to recognize remote interface IP");

    // Handshake
    let mut buf = [0; 1500];
    let (_, addr) = socket.recv_from(&mut buf).unwrap();
    ft_peer(socket, &addr, &name, &ip).await;
}
