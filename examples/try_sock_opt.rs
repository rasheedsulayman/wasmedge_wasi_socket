use std::io;
use std::net::Ipv4Addr;
use wasmedge_wasi_socket::{socket::{AddressFamily, Socket, SocketType}, socket_wamr, ToSocketAddrs};
use wasmedge_wasi_socket::socket_wamr::{wamr_sock_bind, WasiAddr, WasiAddrIp4, WasiAddrIp4Port, WasiAddrType, WasiAddrUnion, WasiErrno, WasiFd, WasiIpPort};

fn main() {
    let sock_fd = open_sock(AddressFamily::Inet4, SocketType::Datagram).unwrap();
    bind_sock(sock_fd as u32);


    // let device = s.device().unwrap();
    // assert!(device.is_none());
    // s.bind_device(Some(b"lo")).unwrap();
    // let device = s.device().unwrap();
    // assert!(device.is_some());
    // assert_eq!(device.unwrap(), b"lo");
    // let addr = "8.8.8.8:53".to_socket_addrs().unwrap().next().unwrap();
    //
    // let recv_timeout = s.get_recv_timeout().unwrap();
    // println!("recv_timeout {:?}", recv_timeout);
    // let send_timeout = s.get_send_timeout().unwrap();
    // println!("send_timeout {:?}", send_timeout);
    //
    // let snd_timeout = std::time::Duration::from_secs(1);
    // let rcv_timeout = std::time::Duration::from_secs(1);
    //
    // s.set_recv_timeout(Some(snd_timeout)).unwrap();
    // s.set_send_timeout(Some(rcv_timeout)).unwrap();
    //
    // let recv_timeout = s.get_recv_timeout().unwrap();
    // println!("recv_timeout {:?}", recv_timeout);
    // assert_eq!(recv_timeout, Some(rcv_timeout));
    // let send_timeout = s.get_send_timeout().unwrap();
    // println!("send_timeout {:?}", send_timeout);
    // assert_eq!(send_timeout, Some(snd_timeout));
    //
    // println!("start connect {addr}");
    // assert!(s.connect(&addr).is_err());
}

pub fn open_sock(addr_family: AddressFamily, sock_kind: SocketType) -> io::Result<i32> {
    let mut socket_fd: socket_wamr::WasiFd = 0;
    let poolfd = u32::MAX;

    let wamr_address_family = match addr_family {
        AddressFamily::Inet4 => socket_wamr::WasiAddressFamily::Inet4,
        AddressFamily::Inet6 => socket_wamr::WasiAddressFamily::Inet6,
        _ => socket_wamr::WasiAddressFamily::InetUnspec,
    };
    let wamr_sock_type = match sock_kind {
        SocketType::Any => socket_wamr::WasiSockType::SocketAny,
        SocketType::Datagram => socket_wamr::WasiSockType::SocketDgram,
        SocketType::Stream => socket_wamr::WasiSockType::SocketStream,
    };
    let errno = socket_wamr::wamr_sock_open(
        poolfd,
        wamr_address_family,
        wamr_sock_type,
        &mut socket_fd,
    );
    if errno == 0 {
        println!("\n\n Successfully opened socket \n\n");
        Ok(socket_fd as i32)
    } else {
        Err(io::Error::from_raw_os_error(errno as i32))
    }
}


pub fn bind_sock(fd: u32) {
    let socket_fd = fd as WasiFd;

    // Bind to IPv4 address 127.0.0.1 and port 8080
    let ipv4_addr = Ipv4Addr::new(127, 0, 0, 1);
    let ipv4_port: WasiIpPort = 5674;

    let wasi_addr = WasiAddr {
        kind: WasiAddrType::IPv4,
        addr: WasiAddrUnion {
            ip4: WasiAddrIp4Port {
                addr: WasiAddrIp4 {
                    n0: ipv4_addr.octets()[0],
                    n1: ipv4_addr.octets()[1],
                    n2: ipv4_addr.octets()[2],
                    n3: ipv4_addr.octets()[3],
                },
                port: ipv4_port.to_be(),
            },
        },
    };

    let bind_result = wamr_sock_bind(socket_fd, &wasi_addr);
    if bind_result != 0 {
        // Handle bind error
        println!("Failed to bind socket: {:?}", bind_result);
    } else {
        println!("\n\n Socket bound successfully\n\n");
    }
}