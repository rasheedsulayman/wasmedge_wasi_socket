/*
Given this definitions:
use wasmedge_wasi_socket::socket_wamr::{WasiErrno, WasiFd};

#[derive(Clone)]
#[repr(C)]
pub enum WasiSockType {
    SocketAny = -1,
    SocketDgram = 0,
    SocketStream,
}
pub type WasiIpPort = u16;
#[derive(Clone, PartialEq, PartialOrd)]
#[repr(C)]
pub enum WasiAddrType {
    IPv4 = 0,
    IPv6,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct WasiAddrIp4 {
    pub n0: u8,
    pub n1: u8,
    pub n2: u8,
    pub n3: u8,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct WasiAddrIp4Port {
    pub addr: WasiAddrIp4,
    pub port: WasiIpPort, // host byte order
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct WasiAddrIp6 {
    pub n0: u16,
    pub n1: u16,
    pub n2: u16,
    pub n3: u16,
    pub h0: u16,
    pub h1: u16,
    pub h2: u16,
    pub h3: u16,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct WasiAddrIp6Port {
    pub addr: WasiAddrIp6,
    pub port: WasiIpPort, // host byte order
}
#[repr(C)]
pub union WasiAddrIpUnion {
    pub ip4: WasiAddrIp4,
    pub ip6: WasiAddrIp6,
}
#[repr(C)]
pub struct WasiAddrIp {
    pub kind: WasiAddrType,
    pub addr: WasiAddrIpUnion,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union WasiAddrUnion {
    pub ip4: WasiAddrIp4Port,
    pub ip6: WasiAddrIp6Port,
}

#[derive(Clone)]
#[repr(C)]
pub struct WasiAddr {
    pub kind: WasiAddrType,
    pub addr: WasiAddrUnion,
}

impl WasiAddr {
    pub fn default() -> Self {
        Self {
            kind: WasiAddrType::IPv4,
            addr: WasiAddrUnion {
                ip4: WasiAddrIp4Port {
                    addr: WasiAddrIp4 {
                        n0: 0,
                        n1: 0,
                        n2: 0,
                        n3: 0,
                    },
                    port: 0,
                },
            },
        }
    }
}

#[repr(C)]
pub enum WasiAddressFamily {
    Inet4 = 0,
    Inet6,
    InetUnspec,
}

#[derive(Clone)]
#[repr(C)]
pub struct WasiAddrInfo {
    pub addr: WasiAddr,
    pub type_: WasiSockType,
}

impl WasiAddrInfo {
    pub fn default() -> Self {
        Self {
            addr: WasiAddr::default(),
            type_: WasiSockType::SocketAny,
        }
    }
}

#[repr(C)]
pub struct WasiAddrInfoHints {
    pub type_: WasiSockType,
    pub family: WasiAddressFamily,
    pub hints_enabled: u8,
}

fn sock_bind(fd: i32, addr: i32) -> i32;

pub fn wamr_sock_bind(fd: WasiFd, addr: *const WasiAddr) -> WasiErrno {
    unsafe { sock_bind(fd as i32, addr as i32) as WasiErrno }
}
pub fn sock_open(fd: i32, af: i32, socktype: i32, sockfd: i32) -> i32;

pub fn wamr_sock_open(
    fd: WasiFd,
    af: WasiAddressFamily,
    socktype: WasiSockType,
    sockfd: *mut WasiFd,
) -> WasiErrno {
    let res = unsafe { sock_open(fd as i32, af as i32, socktype as i32, sockfd as i32) };
    res as WasiErrno
}

*/