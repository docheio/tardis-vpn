use std::io::Result;
use std::net::SocketAddr;
use std::process::Command;
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

pub async fn server() {
    let mut core = Core::new().unwrap();

    // Read Local & Remote IP from args
    let loc_address = env::args().nth(2).unwrap().parse().unwrap_or_else(|err| {
        eprintln!("Unable to recognize listen ip: {}", err);
        process::exit(1);
    });

    // Create socket
    let socket = UdpSocket::bind(&loc_address, &core.handle()).unwrap();
    println!("ok0");
    let mut buf = vec![0; 10];
    println!("ok1");
    let (_, rem_address) = socket.recv_from(&mut buf).unwrap();
    println!("ok2");
    // let (sender, receiver) = socket.framed(VecCodec(rem_address)).split();

    // Create interface
    let name = &env::args().nth(3).expect("Unable to read Interface name");
    let tap = Iface::new(&name, Mode::Tap).unwrap_or_else(|err| {
        eprintln!("Failed to configure the interface name: {}", err);
        process::exit(1);
    });

    // Configure the „local“ (kernel) endpoint.
    let ip = &env::args()
        .nth(4)
        .expect("Unable to recognize remote interface IP");
    cmd("ip", &["addr", "add", "dev", tap.name(), &ip]);
    cmd("ip", &["link", "set", "up", "dev", tap.name()]);

    // Handshake
    // let (sink, stream) = Async::new(tap, &core.handle()).unwrap().split();
    // let reader = stream.forward(sender);
    // let writer = receiver.forward(sink);
    // core.run(reader.join(writer)).unwrap();
}
