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
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::{env, process, thread};

use std::net::UdpSocket;

use tun_tap::{Iface, Mode};

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
    let socket = match UdpSocket::bind(&loc_address) {
        Ok(socket) => socket,
        Err(e) => {
            eprintln!("ERROR: {:?}", e);
            process::exit(1);
        }
    };
    let socket = Arc::new(socket);

    // Create interface
    let name = &env::args().nth(3).expect("Unable to read Interface name");
    let iface = match Iface::new(&name, Mode::Tap) {
        Ok(iface) => iface,
        Err(e) => {
            eprintln!("ERROR: {:?}", e);
            process::exit(1);
        }
    };
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
    let iface_reader = iface.clone();
    let socket_send = socket.clone();
    let s_addr: Arc<Mutex<Option<SocketAddr>>> = Arc::new(Mutex::new(None));
    let r_addr = s_addr.clone();
    let w_addr = s_addr.clone();

    let reader = thread::spawn({
        move || {
            println!("r loaded");
            loop {
                let mut buf = vec![0; 1518];
                let r_addr = r_addr.lock().unwrap();
                let len = match iface_reader.recv(&mut buf) {
                    Ok(len) => len,
                    Err(e) => {
                        eprintln!("ERROR: {:?}", e);
                        process::exit(1);
                    }
                };
                println!("if recv");
                match *r_addr {
                    None => {
                        println!("ignored");
                    }
                    Some(addr) => {
                        if len > 0 {
                            println!("obeyd");
                            match socket_send.send_to(&buf[..len], addr) {
                                Ok(x) => x,
                                Err(_) => 0,
                            };
                            println!("send: {:?}", len);
                        }
                    }
                }
            }
        }
    });
    loop {
        let iface_writer = iface.clone();
        let socket_recv = socket.clone();
        let mut buf = vec![0; 1];
        socket_recv.set_read_timeout(None).unwrap();
        let (_, addr) = socket.recv_from(&mut buf).unwrap();
        {
            let mut w_addr = w_addr.lock().unwrap();
            *w_addr = Some(addr);
        };
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
                if 0 < len && len <= 1518 {
                    match iface_writer.send(&buf[..len]) {
                        Ok(s) => s,
                        Err(e) => {
                            eprintln!("ERROR: {:?}", e);
                            process::exit(1);
                        }
                    };
                    println!("recv: {:?}", len);
                } else if len == 0 {
                    continue;
                } else {
                    eprintln!("WARN: Received invalid byte");
                }
            }
            println!("w end");
        });
        writer.join().unwrap();
        {
            let mut w_addr = w_addr.lock().unwrap();
            *w_addr = None;
        };
        if reader.is_finished() {
            break;
        }
    }
}
