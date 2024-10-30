// struct addrinfo {
//     int ai_flags;             /* Input flags.  */
//     int ai_family;            /* Protocol family for socket.  */
//     int ai_socktype;          /* Socket type.  */
//     int ai_protocol;          /* Protocol for socket.  */
//     socklen_t ai_addrlen;     /* Length of socket address.  */
//     struct sockaddr *ai_addr; /* Socket address for socket.  */
//     char *ai_canonname;       /* Canonical name for service location.  */
//     struct addrinfo *ai_next; /* Pointer to next in list.  */
// };
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

// Socket options
pub const SO_REUSEADDR: u8 = 2;
pub const SO_BROADCAST: u8 = 6;
pub const SO_SNDBUF: u8 = 7;
pub const SO_RCVBUF: u8 = 8;
pub const SO_KEEPALIVE: u8 = 9;
pub const SO_LINGER: u8 = 13;
pub const SO_REUSEPORT: u8 = 15;
pub const SO_RCVTIMEO: u8 = 20;
pub const SO_SNDTIMEO: u8 = 21;

// TCP options
pub const TCP_NODELAY: u8 = 1;
pub const TCP_KEEPIDLE: u8 = 4;
pub const TCP_KEEPINTVL: u8 = 5;
pub const TCP_QUICKACK: u8 = 12;
pub const TCP_FASTOPEN_CONNECT: u8 = 30;

// IP options
pub const IP_TTL: u8 = 2;
pub const IP_MULTICAST_TTL: u8 = 33;
pub const IP_MULTICAST_LOOP: u8 = 34;
pub const IP_ADD_MEMBERSHIP: u8 = 35;
pub const IP_DROP_MEMBERSHIP: u8 = 36;

// IPv6 options
pub const IPV6_MULTICAST_LOOP: u8 = 19;
pub const IPV6_JOIN_GROUP: u8 = 20;
pub const IPV6_LEAVE_GROUP: u8 = 21;
pub const IPV6_V6ONLY: u8 = 26;

/////////// code above is translated from C to Rust
pub type WasiFd = u32;
pub type WasiErrno = u16;
pub type WasiSize = u32;

#[derive(Copy, Clone, Debug)]
#[repr(u8, align(1))]
pub enum SocketOptName {
    SoReuseaddr(bool),
    SoReuseport(bool),
    // SoType = 1,
    // SoError = 2,
    // SoDontroute = 3,
    SoBroadcast(bool),
    SoSndbuf(WasiSize),
    SoRcvbuf(WasiSize),
    SoKeepalive(bool),
    // SoOobinline = 8,
    // SoLinger = 9,
    // SoRcvlowat = 10,
    SoRcvtimeo(u64),
    SoSndtimeo(u64),
    // SoAcceptconn = 13,
    // SoBindToDevice = 14,
}

#[link(wasm_import_module = "wasi_snapshot_preview1")]
extern "C" {
    pub fn sock_open(fd: i32, af: i32, socktype: i32, sockfd: i32) -> i32;
    pub fn sock_addr_resolve(
        host: i32,
        service: i32,
        hints: i32,
        addr_info: i32,
        addr_info_size: i32,
        max_info_size: i32,
    ) -> i32;

    fn sock_addr_local(fd: i32, addr: i32) -> i32;
    fn sock_addr_remote(fd: i32, addr: i32) -> i32;
    fn sock_bind(fd: i32, addr: i32) -> i32;
    fn sock_send_to(
        fd: i32,
        si_data: i32,
        si_data_len: i32,
        si_flags: i32,
        dest_addr: i32,
        so_data_len: i32,
    ) -> i32;
    fn sock_recv_from(
        fd: i32,
        ri_data: i32,
        ri_data_len: i32,
        ri_flags: i32,
        src_addr: i32,
        ro_data_len: i32,
    ) -> i32;
    fn sock_close(fd: i32) -> i32;
    fn sock_connect(fd: i32, addr: i32) -> i32;
    fn sock_get_recv_buf_size(fd: i32, size: i32) -> i32;
    fn sock_get_reuse_addr(fd: i32, reuse: i32) -> i32;
    fn sock_get_reuse_port(fd: i32, reuse: i32) -> i32;
    fn sock_get_send_buf_size(fd: i32, size: i32) -> i32;
    fn sock_listen(fd: i32, backlog: i32) -> i32;
    fn sock_set_recv_buf_size(fd: i32, size: i32) -> i32;
    fn sock_set_reuse_addr(fd: i32, reuse: i32) -> i32;
    fn sock_set_reuse_port(fd: i32, reuse: i32) -> i32;
    fn sock_set_send_buf_size(fd: i32, buf_len: i32) -> i32;
    fn sock_get_recv_timeout(fd: i32, timeout_us: i32) -> i32;
    fn sock_set_recv_timeout(fd: i32, timeout_us: i64) -> i32;
    fn sock_get_send_timeout(fd: i32, timeout_us: i32) -> i32;
    fn sock_set_send_timeout(fd: i32, timeout_us: i64) -> i32;
    fn sock_set_keep_alive(fd: i32, option: i32) -> i32;
    fn sock_get_keep_alive(fd: i32, option: i32) -> i32;
    fn sock_set_linger(fd: i32, is_enabled: i32, linger_s: i32) -> i32;
    fn sock_get_linger(fd: i32, is_enabled: i32, linger_s: i32) -> i32;
    fn sock_set_tcp_keep_idle(fd: i32, time_s: i32) -> i32;
    fn sock_get_tcp_keep_idle(fd: i32, time_s: i32) -> i32;
    fn sock_set_tcp_keep_intvl(fd: i32, time_s: i32) -> i32;
    fn sock_get_tcp_keep_intvl(fd: i32, time_s: i32) -> i32;
    fn sock_set_tcp_fastopen_connect(fd: i32, option: i32) -> i32;
    fn sock_get_tcp_fastopen_connect(fd: i32, option: i32) -> i32;
    fn sock_set_ip_multicast_loop(fd: i32, ipv6: i32, option: i32) -> i32;
    fn sock_get_ip_multicast_loop(fd: i32, ipv6: i32, option: i32) -> i32;
    fn sock_set_ip_multicast_ttl(fd: i32, option: i32) -> i32;
    fn sock_get_ip_multicast_ttl(fd: i32, option: i32) -> i32;
    fn sock_set_ip_add_membership(fd: i32, imr_multiaddr: i32, imr_interface: i32) -> i32;
    fn sock_set_ip_drop_membership(fd: i32, imr_multiaddr: i32, imr_interface: i32) -> i32;
    fn sock_set_broadcast(fd: i32, option: i32) -> i32;
    fn sock_get_broadcast(fd: i32, option: i32) -> i32;
    fn sock_set_tcp_no_delay(fd: i32, option: i32) -> i32;
    fn sock_get_tcp_no_delay(fd: i32, option: i32) -> i32;
    fn sock_set_tcp_quick_ack(fd: i32, option: i32) -> i32;
    fn sock_get_tcp_quick_ack(fd: i32, option: i32) -> i32;
    fn sock_set_ip_ttl(fd: i32, option: i32) -> i32;
    fn sock_get_ip_ttl(fd: i32, option: i32) -> i32;
    fn sock_set_ipv6_only(fd: i32, option: i32) -> i32;
    fn sock_get_ipv6_only(fd: i32, option: i32) -> i32;
}

pub fn wamr_sock_open(
    fd: WasiFd,
    af: WasiAddressFamily,
    socktype: WasiSockType,
    sockfd: *mut WasiFd,
) -> WasiErrno {
    let res = unsafe { sock_open(fd as i32, af as i32, socktype as i32, sockfd as i32) };
    res as WasiErrno
}

pub fn wamr_sock_addr_resolve(
    host: *const u8,
    service: *const u8,
    hints: *mut WasiAddrInfoHints,
    addr_info: *mut WasiAddrInfo,
    addr_info_size: WasiSize,
    max_info_size: *mut WasiSize,
) -> WasiErrno {
    let res = unsafe {
        sock_addr_resolve(
            host as i32,
            service as i32,
            hints as i32,
            addr_info as i32,
            addr_info_size as i32,
            max_info_size as i32,
        )
    };
    res as WasiErrno
}

pub fn wamr_sock_addr_local(fd: WasiFd, addr: *mut WasiAddr) -> WasiErrno {
    unsafe { sock_addr_local(fd as i32, addr as i32) as WasiErrno }
}
pub fn wamr_sock_addr_remote(fd: WasiFd, addr: *mut WasiAddr) -> WasiErrno {
    unsafe { sock_addr_remote(fd as i32, addr as i32) as WasiErrno }
}
pub fn wamr_sock_bind(fd: WasiFd, addr: *const WasiAddr) -> WasiErrno {
    unsafe { sock_bind(fd as i32, addr as i32) as WasiErrno }
}
// pub fn wamr_sock_send_to(
//     fd: WasiFd,
//     si_data: *const WasiCiovec,
//     si_data_len: u32,
//     si_flags: WasiSiflags,
//     dest_addr: *const WasiAddr,
//     so_data_len: *mut u32,
// ) -> WasiErrno {
//     unsafe {
//         sock_send_to(
//             fd as i32,
//             si_data as i32,
//             si_data_len as i32,
//             si_flags as i32,
//             dest_addr as i32,
//             so_data_len as i32,
//         ) as WasiErrno
//     }
// }
// pub fn wamr_sock_recv_from(
//     fd: WasiFd,
//     ri_data: *mut WasiCiovec,
//     ri_data_len: u32,
//     ri_flags: WasiRiflags,
//     src_addr: *mut WasiAddr,
//     ro_data_len: *mut u32,
// ) -> WasiErrno {
//     unsafe {
//         sock_recv_from(
//             fd as i32,
//             ri_data as i32,
//             ri_data_len as i32,
//             ri_flags as i32,
//             src_addr as i32,
//             ro_data_len as i32,
//         ) as WasiErrno
//     }
// }
pub fn wamr_sock_close(fd: WasiFd) -> WasiErrno {
    unsafe { sock_close(fd as i32) as WasiErrno }
}
pub fn wamr_sock_connect(fd: WasiFd, addr: *const WasiAddr) -> WasiErrno {
    unsafe { sock_connect(fd as i32, addr as i32) as WasiErrno }
}
pub fn wamr_sock_get_recv_buf_size(fd: WasiFd, size: *mut WasiSize) -> WasiErrno {
    unsafe { sock_get_recv_buf_size(fd as i32, size as i32) as WasiErrno }
}
pub fn wamr_sock_get_reuse_addr(fd: WasiFd, reuse: *mut bool) -> WasiErrno {
    unsafe { sock_get_reuse_addr(fd as i32, reuse as i32) as WasiErrno }
}
pub fn wamr_sock_get_reuse_port(fd: WasiFd, reuse: *mut bool) -> WasiErrno {
    unsafe { sock_get_reuse_port(fd as i32, reuse as i32) as WasiErrno }
}
pub fn wamr_sock_get_send_buf_size(fd: WasiFd, size: *mut WasiSize) -> WasiErrno {
    unsafe { sock_get_send_buf_size(fd as i32, size as i32) as WasiErrno }
}
pub fn wamr_sock_listen(fd: WasiFd, backlog: WasiSize) -> WasiErrno {
    unsafe { sock_listen(fd as i32, backlog as i32) as WasiErrno }
}
pub fn wamr_sock_set_recv_buf_size(fd: WasiFd, size: WasiSize) -> WasiErrno {
    unsafe { sock_set_recv_buf_size(fd as i32, size as i32) as WasiErrno }
}
pub fn wamr_sock_set_reuse_addr(fd: WasiFd, reuse: bool) -> WasiErrno {
    unsafe { sock_set_reuse_addr(fd as i32, reuse as i32) as WasiErrno }
}
pub fn wamr_sock_set_reuse_port(fd: WasiFd, reuse: bool) -> WasiErrno {
    unsafe { sock_set_reuse_port(fd as i32, reuse as i32) as WasiErrno }
}
pub fn wamr_sock_set_send_buf_size(fd: WasiFd, buf_len: WasiSize) -> WasiErrno {
    unsafe { sock_set_send_buf_size(fd as i32, buf_len as i32) as WasiErrno }
}
pub fn wamr_sock_get_recv_timeout(fd: WasiFd, timeout_us: *mut u64) -> WasiErrno {
    unsafe { sock_get_recv_timeout(fd as i32, timeout_us as i32) as WasiErrno }
}
pub fn wamr_sock_set_recv_timeout(fd: WasiFd, timeout_us: u64) -> WasiErrno {
    unsafe { sock_set_recv_timeout(fd as i32, timeout_us as i64) as WasiErrno }
}
pub fn wamr_sock_get_send_timeout(fd: WasiFd, timeout_us: *mut u64) -> WasiErrno {
    unsafe { sock_get_send_timeout(fd as i32, timeout_us as i32) as WasiErrno }
}
pub fn wamr_sock_set_send_timeout(fd: WasiFd, timeout_us: u64) -> WasiErrno {
    unsafe { sock_set_send_timeout(fd as i32, timeout_us as i64) as WasiErrno }
}
pub fn wamr_sock_set_keep_alive(fd: WasiFd, option: bool) -> WasiErrno {
    unsafe { sock_set_keep_alive(fd as i32, option as i32) as WasiErrno }
}
pub fn wamr_sock_get_keep_alive(fd: WasiFd, option: *mut bool) -> WasiErrno {
    unsafe { sock_get_keep_alive(fd as i32, option as i32) as WasiErrno }
}
pub fn wamr_sock_set_linger(fd: WasiFd, is_enabled: bool, linger_s: i32) -> WasiErrno {
    unsafe { sock_set_linger(fd as i32, is_enabled as i32, linger_s) as WasiErrno }
}
pub fn wamr_sock_get_linger(fd: WasiFd, is_enabled: *mut bool, linger_s: *mut i32) -> WasiErrno {
    unsafe { sock_get_linger(fd as i32, is_enabled as i32, linger_s as i32) as WasiErrno }
}
pub fn wamr_sock_set_tcp_keep_idle(fd: WasiFd, time_s: u32) -> WasiErrno {
    unsafe { sock_set_tcp_keep_idle(fd as i32, time_s as i32) as WasiErrno }
}
pub fn wamr_sock_get_tcp_keep_idle(fd: WasiFd, time_s: *mut u32) -> WasiErrno {
    unsafe { sock_get_tcp_keep_idle(fd as i32, time_s as i32) as WasiErrno }
}
pub fn wamr_sock_set_tcp_keep_intvl(fd: WasiFd, time_s: u32) -> WasiErrno {
    unsafe { sock_set_tcp_keep_intvl(fd as i32, time_s as i32) as WasiErrno }
}
pub fn wamr_sock_get_tcp_keep_intvl(fd: WasiFd, time_s: *mut u32) -> WasiErrno {
    unsafe { sock_get_tcp_keep_intvl(fd as i32, time_s as i32) as WasiErrno }
}
pub fn wamr_sock_set_tcp_fastopen_connect(fd: WasiFd, option: bool) -> WasiErrno {
    unsafe { sock_set_tcp_fastopen_connect(fd as i32, option as i32) as WasiErrno }
}
pub fn wamr_sock_get_tcp_fastopen_connect(fd: WasiFd, option: *mut bool) -> WasiErrno {
    unsafe { sock_get_tcp_fastopen_connect(fd as i32, option as i32) as WasiErrno }
}
pub fn wamr_sock_set_ip_multicast_loop(fd: WasiFd, ipv6: bool, option: bool) -> WasiErrno {
    unsafe { sock_set_ip_multicast_loop(fd as i32, ipv6 as i32, option as i32) as WasiErrno }
}
pub fn wamr_sock_get_ip_multicast_loop(fd: WasiFd, ipv6: bool, option: *mut bool) -> WasiErrno {
    unsafe { sock_get_ip_multicast_loop(fd as i32, ipv6 as i32, option as i32) as WasiErrno }
}
pub fn wamr_sock_set_ip_multicast_ttl(fd: WasiFd, option: u8) -> WasiErrno {
    unsafe { sock_set_ip_multicast_ttl(fd as i32, option as i32) as WasiErrno }
}
pub fn wamr_sock_get_ip_multicast_ttl(fd: WasiFd, option: *mut u8) -> WasiErrno {
    unsafe { sock_get_ip_multicast_ttl(fd as i32, option as i32) as WasiErrno }
}
pub fn wamr_sock_set_ip_add_membership(
    fd: WasiFd,
    imr_multiaddr: *const WasiAddrIp,
    imr_interface: u32,
) -> WasiErrno {
    unsafe {
        sock_set_ip_add_membership(fd as i32, imr_multiaddr as i32, imr_interface as i32)
            as WasiErrno
    }
}
pub fn wamr_sock_set_ip_drop_membership(
    fd: WasiFd,
    imr_multiaddr: *const WasiAddrIp,
    imr_interface: u32,
) -> WasiErrno {
    unsafe {
        sock_set_ip_drop_membership(fd as i32, imr_multiaddr as i32, imr_interface as i32)
            as WasiErrno
    }
}
pub fn wamr_sock_set_broadcast(fd: WasiFd, option: bool) -> WasiErrno {
    unsafe { sock_set_broadcast(fd as i32, option as i32) as WasiErrno }
}
pub fn wamr_sock_get_broadcast(fd: WasiFd, option: *mut bool) -> WasiErrno {
    unsafe { sock_get_broadcast(fd as i32, option as i32) as WasiErrno }
}
pub fn wamr_sock_set_tcp_no_delay(fd: WasiFd, option: bool) -> WasiErrno {
    unsafe { sock_set_tcp_no_delay(fd as i32, option as i32) as WasiErrno }
}
pub fn wamr_sock_get_tcp_no_delay(fd: WasiFd, option: *mut bool) -> WasiErrno {
    unsafe { sock_get_tcp_no_delay(fd as i32, option as i32) as WasiErrno }
}
pub fn wamr_sock_set_tcp_quick_ack(fd: WasiFd, option: bool) -> WasiErrno {
    unsafe { sock_set_tcp_quick_ack(fd as i32, option as i32) as WasiErrno }
}
pub fn wamr_sock_get_tcp_quick_ack(fd: WasiFd, option: *mut bool) -> WasiErrno {
    unsafe { sock_get_tcp_quick_ack(fd as i32, option as i32) as WasiErrno }
}
pub fn wamr_sock_set_ip_ttl(fd: WasiFd, option: u8) -> WasiErrno {
    unsafe { sock_set_ip_ttl(fd as i32, option as i32) as WasiErrno }
}
pub fn wamr_sock_get_ip_ttl(fd: WasiFd, option: *mut u8) -> WasiErrno {
    unsafe { sock_get_ip_ttl(fd as i32, option as i32) as WasiErrno }
}
pub fn wamr_sock_set_ipv6_only(fd: WasiFd, option: bool) -> WasiErrno {
    unsafe { sock_set_ipv6_only(fd as i32, option as i32) as WasiErrno }
}
pub fn wamr_sock_get_ipv6_only(fd: WasiFd, option: *mut bool) -> WasiErrno {
    unsafe { sock_get_ipv6_only(fd as i32, option as i32) as WasiErrno }
}