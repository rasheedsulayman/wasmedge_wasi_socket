use core::ffi;
use std::io;
use std::mem::MaybeUninit;
use std::net::{Ipv4Addr, Ipv6Addr, Shutdown, SocketAddr, SocketAddrV4, SocketAddrV6};
use std::os::wasi::prelude::{AsRawFd, FromRawFd, IntoRawFd, RawFd};

#[derive(Copy, Clone, Debug)]
#[repr(u8, align(1))]
pub enum AddressFamily {
    Unspec,
    Inet4,
    Inet6,
}

#[allow(unreachable_patterns)]
impl From<&SocketAddr> for AddressFamily {
    fn from(addr: &SocketAddr) -> Self {
        match addr {
            SocketAddr::V4(_) => AddressFamily::Inet4,
            SocketAddr::V6(_) => AddressFamily::Inet6,
            _ => AddressFamily::Unspec,
        }
    }
}

impl AddressFamily {
    pub fn is_unspec(&self) -> bool {
        matches!(*self, AddressFamily::Unspec)
    }

    pub fn is_v4(&self) -> bool {
        matches!(*self, AddressFamily::Inet4)
    }

    pub fn is_v6(&self) -> bool {
        matches!(*self, AddressFamily::Inet6)
    }
}

#[derive(Copy, Clone, Debug)]
#[repr(u8, align(1))]
pub enum SocketType {
    Any,
    Datagram,
    Stream,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct WasiAddress {
    pub buf: *const u8,
    pub size: usize,
}

unsafe impl Send for WasiAddress {}

#[derive(Copy, Clone, Debug)]
#[repr(u16, align(2))]
pub enum AiFlags {
    AiPassive,
    AiCanonname,
    AiNumericHost,
    AiNumericServ,
    AiV4Mapped,
    AiAll,
    AiAddrConfig,
}

#[derive(Copy, Clone, Debug)]
#[repr(u8, align(1))]
pub enum AiProtocol {
    IPProtoIP,
    IPProtoTCP,
    IPProtoUDP,
}

#[derive(Debug, Clone)]
#[repr(C)]
pub struct WasiSockaddr {
    pub family: AddressFamily,
    pub sa_data_len: u32,
    pub sa_data: *mut u8,
}

impl WasiSockaddr {
    pub fn new(family: AddressFamily, sa_data: &mut [u8]) -> WasiSockaddr {
        WasiSockaddr {
            family,
            sa_data_len: 14,
            sa_data: sa_data.as_mut_ptr(),
        }
    }
}

impl Default for WasiSockaddr {
    fn default() -> WasiSockaddr {
        WasiSockaddr {
            family: AddressFamily::Inet4,
            sa_data_len: 14,
            sa_data: std::ptr::null_mut(),
        }
    }
}

#[derive(Debug, Clone)]
#[repr(C, packed(4))]
pub struct WasiAddrinfo {
    pub ai_flags: AiFlags,
    pub ai_family: AddressFamily,
    pub ai_socktype: SocketType,
    pub ai_protocol: AiProtocol,
    pub ai_addrlen: u32,
    pub ai_addr: *mut WasiSockaddr,
    pub ai_canonname: *mut u8,
    pub ai_canonnamelen: u32,
    pub ai_next: *mut WasiAddrinfo,
}

impl WasiAddrinfo {
    pub fn default() -> WasiAddrinfo {
        WasiAddrinfo {
            ai_flags: AiFlags::AiPassive,
            ai_family: AddressFamily::Inet4,
            ai_socktype: SocketType::Stream,
            ai_protocol: AiProtocol::IPProtoTCP,
            ai_addr: std::ptr::null_mut(),
            ai_addrlen: 0,
            ai_canonname: std::ptr::null_mut(),
            ai_canonnamelen: 0,
            ai_next: std::ptr::null_mut(),
        }
    }

    /// Get Address Information
    ///
    /// As calling FFI, use buffer as parameter in order to avoid memory leak.
    /// TODO: make it actually return the same amount of info as getaddinfo in C
    pub fn get_addrinfo(
        node: &str,
        service: &str,
        hints: &WasiAddrinfo,
        max_reslen: usize,
    ) -> io::Result<Vec<socket_wamr::WasiAddrInfo>> {
        let mut node = node.to_string();
        let mut service = service.to_string();

        if !node.ends_with('\0') {
            node.push('\0');
        }

        if !service.ends_with('\0') {
            service.push('\0');
        }

        let socket_type = match hints.ai_socktype {
            SocketType::Any => socket_wamr::WasiSockType::SocketAny,
            SocketType::Datagram => socket_wamr::WasiSockType::SocketDgram,
            SocketType::Stream => socket_wamr::WasiSockType::SocketStream,
        };

        let address_family = match hints.ai_family {
            AddressFamily::Inet4 => socket_wamr::WasiAddressFamily::Inet4,
            AddressFamily::Inet6 => socket_wamr::WasiAddressFamily::Inet6,
            _ => socket_wamr::WasiAddressFamily::InetUnspec,
        };

        let mut wamr_hints = socket_wamr::WasiAddrInfoHints {
            type_: socket_type,
            family: address_family,
            hints_enabled: 0,
        };
        let mut addr_info_array: Vec<socket_wamr::WasiAddrInfo> =
            vec![socket_wamr::WasiAddrInfo::default(); max_reslen];
        let mut max_info_size = max_reslen as socket_wamr::WasiSize;
        let errno = socket_wamr::wamr_sock_addr_resolve(
            node.as_ptr(),
            service.as_ptr(),
            &mut wamr_hints,
            addr_info_array.as_mut_ptr(),
            max_reslen.try_into().unwrap(),
            &mut max_info_size,
        );

        if errno != 0 {
            return Err(io::Error::from_raw_os_error(errno.into()));
        }
        return Ok(addr_info_array);
    }
}

#[repr(C)]
pub struct IovecRead {
    pub buf: *mut u8,
    pub size: usize,
}

impl From<libc::iovec> for IovecRead {
    fn from(value: libc::iovec) -> Self {
        IovecRead {
            buf: value.iov_base.cast(),
            size: value.iov_len,
        }
    }
}

#[repr(C)]
pub struct IovecWrite {
    pub buf: *const u8,
    pub size: usize,
}

impl From<libc::iovec> for IovecWrite {
    fn from(value: libc::iovec) -> Self {
        IovecWrite {
            buf: value.iov_base.cast(),
            size: value.iov_len,
        }
    }
}

#[derive(Copy, Clone, Debug)]
#[repr(u8, align(1))]
pub enum SocketOptLevel {
    SolSocket = 0,
}

impl TryFrom<i32> for SocketOptLevel {
    type Error = io::Error;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::SolSocket),
            _ => Err(io::Error::from_raw_os_error(libc::EINVAL)),
        }
    }
}

#[derive(Copy, Clone, Debug)]
#[repr(u8, align(1))]
pub enum SocketOptName {
    SoReuseaddr = 0,
    SoType = 1,
    SoError = 2,
    SoDontroute = 3,
    SoBroadcast = 4,
    SoSndbuf = 5,
    SoRcvbuf = 6,
    SoKeepalive = 7,
    SoOobinline = 8,
    SoLinger = 9,
    SoRcvlowat = 10,
    SoRcvtimeo = 11,
    SoSndtimeo = 12,
    SoAcceptconn = 13,
    SoBindToDevice = 14,
}

impl TryFrom<i32> for SocketOptName {
    type Error = io::Error;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::SoReuseaddr),
            1 => Ok(Self::SoType),
            2 => Ok(Self::SoError),
            3 => Ok(Self::SoDontroute),
            4 => Ok(Self::SoBroadcast),
            5 => Ok(Self::SoSndbuf),
            6 => Ok(Self::SoRcvbuf),
            7 => Ok(Self::SoKeepalive),
            8 => Ok(Self::SoOobinline),
            9 => Ok(Self::SoLinger),
            10 => Ok(Self::SoRcvlowat),
            11 => Ok(Self::SoRcvtimeo),
            12 => Ok(Self::SoSndtimeo),
            13 => Ok(Self::SoAcceptconn),
            14 => Ok(Self::SoBindToDevice),

            _ => Err(io::Error::from_raw_os_error(libc::EINVAL)),
        }
    }
}

pub const MSG_PEEK: u16 = 1; // __WASI_RIFLAGS_RECV_PEEK
pub const MSG_WAITALL: u16 = 2; // __WASI_RIFLAGS_RECV_WAITALL

pub const MSG_TRUNC: u16 = 1; // __WASI_ROFLAGS_RECV_DATA_TRUNCATED

macro_rules! syscall {
    ($fn: ident ( $($arg: expr),* $(,)* ) ) => {{
        #[allow(unused_unsafe)]
        let res = unsafe { libc::$fn($($arg, )*) };
        if res == -1 {
            Err(std::io::Error::last_os_error())
        } else {
            Ok(res)
        }
    }};
}

fn fcntl_get(fd: RawFd, cmd: i32) -> io::Result<i32> {
    syscall!(fcntl(fd, cmd))
}

fn fcntl_add(fd: RawFd, get_cmd: i32, set_cmd: i32, flag: i32) -> io::Result<()> {
    let previous = syscall!(fcntl(fd, get_cmd))?;
    let new = previous | flag;
    if new != previous {
        syscall!(fcntl(fd, set_cmd, new)).map(|_| ())
    } else {
        // Flag was already set.
        Ok(())
    }
}

/// Remove `flag` to the current set flags of `F_GETFD`.
fn fcntl_remove(fd: RawFd, get_cmd: i32, set_cmd: i32, flag: i32) -> io::Result<()> {
    let previous = syscall!(fcntl(fd, get_cmd))?;
    let new = previous & !flag;
    if new != previous {
        syscall!(fcntl(fd, set_cmd, new)).map(|_| ())
    } else {
        // Flag was already set.
        Ok(())
    }
}

mod wasi_sock {
    use super::{IovecRead, IovecWrite, WasiAddress};

    #[link(wasm_import_module = "wasi_snapshot_preview1")]
    extern "C" {
        // pub fn sock_open(addr_family: u8, sock_type: u8, fd: *mut u32) -> u32;
        pub fn sock_bind(fd: u32, addr: *mut WasiAddress, port: u32) -> u32;
        pub fn sock_listen(fd: u32, backlog: u32) -> u32;
        pub fn sock_accept(fd: u32, fd: *mut u32) -> u32;
        // pub fn sock_connect(fd: u32, addr: *mut WasiAddress, port: u32) -> u32;
        pub fn sock_recv(
            fd: u32,
            buf: *mut IovecRead,
            buf_len: usize,
            flags: u16,
            recv_len: *mut usize,
            oflags: *mut usize,
        ) -> u32;

        pub fn sock_recv_from(
            fd: u32,
            buf: *mut IovecRead,
            buf_len: u32,
            addr: *mut u8,
            flags: u16,
            port: *mut u32,
            recv_len: *mut usize,
            oflags: *mut usize,
        ) -> u32;
        pub fn sock_send(
            fd: u32,
            buf: *const IovecWrite,
            buf_len: u32,
            flags: u16,
            send_len: *mut u32,
        ) -> u32;
        pub fn sock_send_to(
            fd: u32,
            buf: *const IovecWrite,
            buf_len: u32,
            addr: *const u8,
            port: u32,
            flags: u16,
            send_len: *mut u32,
        ) -> u32;
        pub fn sock_shutdown(fd: u32, flags: u8) -> u32;
        // pub fn sock_getpeeraddr(
        //     fd: u32,
        //     addr: *mut WasiAddress,
        //     addr_type: *mut u32,
        //     port: *mut u32,
        // ) -> u32;
        // pub fn sock_getlocaladdr(
        //     fd: u32,
        //     addr: *mut WasiAddress,
        //     addr_type: *mut u32,
        //     port: *mut u32,
        // ) -> u32;
        pub fn sock_getsockopt( // I just commented it's usage when in fact it needs to be substituted with WAMR socket ext apis
                                fd: u32,
                                level: i32,
                                name: i32,
                                flag: *mut i32,
                                flag_size: *mut u32,
        ) -> u32;

        #[allow(unused)]
        pub fn sock_setsockopt( // I replaced its usage everywhere where found possible, but it could've been enough only for POC
                                fd: u32,
                                level: i32,
                                name: i32,
                                flag: *const i32,
                                flag_size: u32,
        ) -> u32;
    }
}

#[derive(Debug)]
pub struct Socket {
    fd: RawFd,
}

use std::time::Duration;
use wasi_sock::*;

use crate::socket_wamr::{self, wamr_sock_bind, WasiAddrIp4, WasiAddrIp4Port, WasiAddrType};

fn from_timeval(duration: libc::timeval) -> Option<Duration> {
    if duration.tv_sec == 0 && duration.tv_usec == 0 {
        None
    } else {
        let sec = duration.tv_sec as u64;
        let nsec = (duration.tv_usec as u32) * 1000;
        Some(Duration::new(sec, nsec))
    }
}

impl Socket {
    pub fn new(addr_family: AddressFamily, sock_kind: SocketType) -> io::Result<Self> {
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
            Ok(Socket {
                fd: socket_fd as i32,
            })
        } else {
            Err(io::Error::from_raw_os_error(errno as i32))
        }
    }

    pub fn device(&self) -> io::Result<Option<Vec<u8>>> {
        let mut buf: [MaybeUninit<u8>; 0x10] = unsafe { MaybeUninit::uninit().assume_init() };
        let mut len = buf.len() as u32;
        let e = unsafe {
            sock_getsockopt(
                self.fd as u32,
                SocketOptLevel::SolSocket as i32,
                SocketOptName::SoBindToDevice as i32,
                &mut buf as *mut _ as *mut i32,
                &mut len,
            )
        };

        if e == 0 {
            if len == 0 {
                Ok(None)
            } else {
                let buf = &buf[..len as usize - 1];
                // TODO: use `MaybeUninit::slice_assume_init_ref` once stable.
                Ok(Some(unsafe { &*(buf as *const [_] as *const [u8]) }.into()))
            }
        } else {
            Err(io::Error::from_raw_os_error(e as i32))
        }
    }

    pub fn bind_device(&self, interface: Option<&[u8]>) -> io::Result<()> {
        let (value, len) = if let Some(interface) = interface {
            (interface.as_ptr(), interface.len())
        } else {
            (std::ptr::null(), 0)
        };

        unsafe {
            let e = sock_setsockopt(
                self.fd as u32,
                SocketOptLevel::SolSocket as u8 as i32,
                SocketOptName::SoBindToDevice as u8 as i32,
                value as *const i32,
                len as u32,
            );
            if e == 0 {
                Ok(())
            } else {
                Err(io::Error::from_raw_os_error(e as i32))
            }
        }
    }

    pub fn set_send_timeout(&self, duration: Option<Duration>) -> io::Result<()> {
        let duration_micros = match duration {
            Some(dur) => dur.as_micros(),
            None => 0,
        };
        self.setsockopt_socket(socket_wamr::SocketOptName::SoSndtimeo(
            duration_micros as u64,
        ))
    }

    pub fn get_send_timeout(&self) -> io::Result<Option<Duration>> {
        unsafe {
            let fd = self.fd;
            let mut payload: MaybeUninit<libc::timeval> = MaybeUninit::uninit();
            let mut len = std::mem::size_of::<libc::timeval>() as u32;

            let e = sock_getsockopt(
                fd as u32,
                SocketOptLevel::SolSocket as i32,
                SocketOptName::SoSndtimeo as i32,
                payload.as_mut_ptr().cast(),
                &mut len,
            );

            if e == 0 {
                Ok(from_timeval(payload.assume_init()))
            } else {
                Err(io::Error::from_raw_os_error(e as i32))
            }
        }
    }

    pub fn set_recv_timeout(&self, duration: Option<std::time::Duration>) -> io::Result<()> {
        let duration_micros = match duration {
            Some(dur) => dur.as_micros(),
            None => 0,
        };
        self.setsockopt_socket(socket_wamr::SocketOptName::SoRcvtimeo(
            duration_micros as u64,
        ))
    }

    pub fn get_recv_timeout(&self) -> io::Result<Option<Duration>> {
        unsafe {
            let fd = self.fd;
            let mut payload: MaybeUninit<libc::timeval> = MaybeUninit::uninit();
            let mut len = std::mem::size_of::<libc::timeval>() as u32;

            let e = sock_getsockopt(
                fd as u32,
                SocketOptLevel::SolSocket as i32,
                SocketOptName::SoRcvtimeo as i32,
                payload.as_mut_ptr().cast(),
                &mut len,
            );

            if e == 0 {
                Ok(from_timeval(payload.assume_init()))
            } else {
                Err(io::Error::from_raw_os_error(e as i32))
            }
        }
    }

    pub fn send(&self, buf: &[u8]) -> io::Result<usize> {
        unsafe {
            let mut send_len: u32 = 0;
            let vec = IovecWrite {
                buf: buf.as_ptr(),
                size: buf.len(),
            };
            let res = sock_send(self.as_raw_fd() as u32, &vec, 1, 0, &mut send_len);
            if res == 0 {
                Ok(send_len as usize)
            } else {
                Err(io::Error::from_raw_os_error(res as i32))
            }
        }
    }

    pub fn send_vectored(&self, bufs: &[io::IoSlice<'_>], flags: u16) -> io::Result<usize> {
        unsafe {
            let mut send_len: u32 = 0;

            let mut write_bufs = Vec::with_capacity(bufs.len());
            for b in bufs {
                write_bufs.push(IovecWrite {
                    buf: b.as_ptr().cast(),
                    size: b.len(),
                });
            }

            let res = sock_send(
                self.as_raw_fd() as u32,
                write_bufs.as_ptr(),
                write_bufs.len() as u32,
                flags,
                &mut send_len,
            );
            if res == 0 {
                Ok(send_len as usize)
            } else {
                Err(io::Error::from_raw_os_error(res as i32))
            }
        }
    }

    pub fn send_to(&self, buf: &[u8], addr: SocketAddr) -> io::Result<usize> {
        let port = addr.port() as u32;
        let vaddr = match addr {
            SocketAddr::V4(ipv4) => ipv4.ip().octets().to_vec(),
            SocketAddr::V6(ipv6) => ipv6.ip().octets().to_vec(),
        };
        let addr = WasiAddress {
            buf: vaddr.as_ptr(),
            size: vaddr.len(),
        };

        let vec = IovecWrite {
            buf: buf.as_ptr(),
            size: buf.len(),
        };

        let flags = 0;
        let mut send_len: u32 = 0;
        unsafe {
            let res = sock_send_to(
                self.fd as u32,
                &vec,
                1,
                &addr as *const WasiAddress as *const u8,
                port,
                flags,
                &mut send_len,
            );
            if res == 0 {
                Ok(send_len as usize)
            } else {
                Err(io::Error::from_raw_os_error(res as i32))
            }
        }
    }

    pub fn send_to_vectored(
        &self,
        bufs: &[io::IoSlice<'_>],
        addr: SocketAddr,
        flags: u16,
    ) -> io::Result<usize> {
        let port = addr.port() as u32;
        let vaddr = match addr {
            SocketAddr::V4(ipv4) => ipv4.ip().octets().to_vec(),
            SocketAddr::V6(ipv6) => ipv6.ip().octets().to_vec(),
        };
        let addr = WasiAddress {
            buf: vaddr.as_ptr(),
            size: vaddr.len(),
        };

        let mut write_bufs = Vec::with_capacity(bufs.len());
        for b in bufs {
            write_bufs.push(IovecWrite {
                buf: b.as_ptr().cast(),
                size: b.len(),
            });
        }

        let mut send_len: u32 = 0;
        unsafe {
            let res = sock_send_to(
                self.fd as u32,
                write_bufs.as_ptr(),
                write_bufs.len() as u32,
                &addr as *const WasiAddress as *const u8,
                port,
                flags,
                &mut send_len,
            );
            if res == 0 {
                Ok(send_len as usize)
            } else {
                Err(io::Error::from_raw_os_error(res as i32))
            }
        }
    }

    pub fn recv(&self, buf: &mut [u8]) -> io::Result<usize> {
        let flags = 0;
        let mut recv_len: usize = 0;
        let mut oflags: usize = 0;
        let mut vec = IovecRead {
            buf: buf.as_mut_ptr(),
            size: buf.len(),
        };

        unsafe {
            let res = sock_recv(
                self.as_raw_fd() as u32,
                &mut vec,
                1,
                flags,
                &mut recv_len,
                &mut oflags,
            );
            if res == 0 {
                Ok(recv_len)
            } else {
                Err(io::Error::from_raw_os_error(res as i32))
            }
        }
    }

    pub fn recv_with_flags(
        &self,
        buf: &mut [MaybeUninit<u8>],
        flags: u16,
    ) -> io::Result<(usize, usize)> {
        let mut recv_len: usize = 0;
        let mut oflags: usize = 0;
        let mut vec = IovecRead {
            buf: buf.as_mut_ptr().cast(),
            size: buf.len(),
        };

        unsafe {
            let res = sock_recv(
                self.as_raw_fd() as u32,
                &mut vec,
                1,
                flags,
                &mut recv_len,
                &mut oflags,
            );
            if res == 0 {
                Ok((recv_len, oflags))
            } else {
                Err(io::Error::from_raw_os_error(res as i32))
            }
        }
    }

    pub fn recv_vectored(&self, bufs: &mut [IovecRead], flags: u16) -> io::Result<(usize, usize)> {
        let mut recv_len: usize = 0;
        let mut oflags: usize = 0;

        unsafe {
            let res = sock_recv(
                self.as_raw_fd() as u32,
                bufs.as_mut_ptr(),
                bufs.len(),
                flags,
                &mut recv_len,
                &mut oflags,
            );
            if res == 0 {
                Ok((recv_len, oflags))
            } else {
                Err(io::Error::from_raw_os_error(res as i32))
            }
        }
    }

    pub fn recv_from(&self, buf: &mut [u8]) -> io::Result<(usize, SocketAddr)> {
        let flags = 0;
        let addr_buf = [0; 128];

        let mut addr = WasiAddress {
            buf: addr_buf.as_ptr(),
            size: 128,
        };

        let mut recv_buf = IovecRead {
            buf: buf.as_mut_ptr(),
            size: buf.len(),
        };

        let mut recv_len: usize = 0;
        let mut oflags: usize = 0;
        let mut sin_port: u32 = 0;
        unsafe {
            let res = sock_recv_from(
                self.as_raw_fd() as u32,
                &mut recv_buf,
                1,
                &mut addr as *mut WasiAddress as *mut u8,
                flags,
                &mut sin_port,
                &mut recv_len,
                &mut oflags,
            );
            if res == 0 {
                let sin_family = {
                    let mut d = [0, 0];
                    d.clone_from_slice(&addr_buf[0..2]);
                    u16::from_le_bytes(d) as u8
                };
                let sin_addr = if sin_family == AddressFamily::Inet4 as u8 {
                    let ip_addr = Ipv4Addr::new(addr_buf[2], addr_buf[3], addr_buf[4], addr_buf[5]);
                    SocketAddr::V4(SocketAddrV4::new(ip_addr, sin_port as u16))
                } else if sin_family == AddressFamily::Inet6 as u8 {
                    let mut ipv6_addr = [0u8; 16];
                    ipv6_addr.copy_from_slice(&addr_buf[2..18]);
                    let ip_addr = Ipv6Addr::from(ipv6_addr);
                    SocketAddr::V6(SocketAddrV6::new(ip_addr, sin_port as u16, 0, 0))
                } else {
                    unimplemented!("Address family not supported by protocol");
                };

                Ok((recv_len, sin_addr))
            } else {
                Err(io::Error::from_raw_os_error(res as i32))
            }
        }
    }

    pub fn recv_from_with_flags(
        &self,
        buf: &mut [MaybeUninit<u8>],
        flags: u16,
    ) -> io::Result<(usize, SocketAddr, usize)> {
        let addr_buf = [0; 128];

        let mut addr = WasiAddress {
            buf: addr_buf.as_ptr(),
            size: 128,
        };

        let mut recv_buf = IovecRead {
            buf: buf.as_mut_ptr().cast(),
            size: buf.len(),
        };

        let mut recv_len: usize = 0;
        let mut oflags: usize = 0;
        let mut sin_port: u32 = 0;
        unsafe {
            let res = sock_recv_from(
                self.as_raw_fd() as u32,
                &mut recv_buf,
                1,
                &mut addr as *mut WasiAddress as *mut u8,
                flags,
                &mut sin_port,
                &mut recv_len,
                &mut oflags,
            );
            if res == 0 {
                let sin_family = {
                    let mut d = [0, 0];
                    d.clone_from_slice(&addr_buf[0..2]);
                    u16::from_le_bytes(d) as u8
                };
                let sin_addr = if sin_family == AddressFamily::Inet4 as u8 {
                    let ip_addr = Ipv4Addr::new(addr_buf[2], addr_buf[3], addr_buf[4], addr_buf[5]);
                    SocketAddr::V4(SocketAddrV4::new(ip_addr, sin_port as u16))
                } else if sin_family == AddressFamily::Inet6 as u8 {
                    let mut ipv6_addr = [0u8; 16];
                    ipv6_addr.copy_from_slice(&addr_buf[2..18]);
                    let ip_addr = Ipv6Addr::from(ipv6_addr);
                    SocketAddr::V6(SocketAddrV6::new(ip_addr, sin_port as u16, 0, 0))
                } else {
                    unimplemented!("Address family not supported by protocol");
                };

                Ok((recv_len, sin_addr, oflags))
            } else {
                Err(io::Error::from_raw_os_error(res as i32))
            }
        }
    }

    pub fn recv_from_vectored(
        &self,
        bufs: &mut [IovecRead],
        flags: u16,
    ) -> io::Result<(usize, SocketAddr, usize)> {
        let addr_buf = [0; 128];

        let mut addr = WasiAddress {
            buf: addr_buf.as_ptr(),
            size: 128,
        };

        let mut recv_len: usize = 0;
        let mut oflags: usize = 0;
        let mut sin_port: u32 = 0;
        unsafe {
            let res = sock_recv_from(
                self.as_raw_fd() as u32,
                bufs.as_mut_ptr(),
                1,
                &mut addr as *mut WasiAddress as *mut u8,
                flags,
                &mut sin_port,
                &mut recv_len,
                &mut oflags,
            );
            if res == 0 {
                let sin_family = {
                    let mut d = [0, 0];
                    d.clone_from_slice(&addr_buf[0..2]);
                    u16::from_le_bytes(d) as u8
                };
                let sin_addr = if sin_family == AddressFamily::Inet4 as u8 {
                    let ip_addr = Ipv4Addr::new(addr_buf[2], addr_buf[3], addr_buf[4], addr_buf[5]);
                    SocketAddr::V4(SocketAddrV4::new(ip_addr, sin_port as u16))
                } else if sin_family == AddressFamily::Inet6 as u8 {
                    let mut ipv6_addr = [0u8; 16];
                    ipv6_addr.copy_from_slice(&addr_buf[2..18]);
                    let ip_addr = Ipv6Addr::from(ipv6_addr);
                    SocketAddr::V6(SocketAddrV6::new(ip_addr, sin_port as u16, 0, 0))
                } else {
                    unimplemented!("Address family not supported by protocol");
                };

                Ok((recv_len, sin_addr, oflags))
            } else {
                Err(io::Error::from_raw_os_error(res as i32))
            }
        }
    }

    pub fn nonblocking(&self) -> io::Result<bool> {
        let fd = self.as_raw_fd();
        let file_status_flags = fcntl_get(fd, libc::F_GETFL)?;
        Ok((file_status_flags & libc::O_NONBLOCK) != 0)
    }

    pub fn set_nonblocking(&self, nonblocking: bool) -> io::Result<()> {
        let fd = self.as_raw_fd();
        if nonblocking {
            fcntl_add(fd, libc::F_GETFL, libc::F_SETFL, libc::O_NONBLOCK)
        } else {
            fcntl_remove(fd, libc::F_GETFL, libc::F_SETFL, libc::O_NONBLOCK)
        }
    }

    pub fn connect(&self, addrs: &SocketAddr) -> io::Result<()> {
        let fd: u32 = self.as_raw_fd() as u32;
        let vaddr;
        let port;
        if let SocketAddr::V4(addrs) = addrs {
            vaddr = addrs.ip().octets();
            port = addrs.port();
        } else {
            return Err(io::Error::from(io::ErrorKind::Unsupported));
        }
        let wasi_addr = socket_wamr::WasiAddr {
            kind: socket_wamr::WasiAddrType::IPv4,
            addr: socket_wamr::WasiAddrUnion {
                ip4: WasiAddrIp4Port {
                    addr: WasiAddrIp4 {
                        n0: vaddr[0],
                        n1: vaddr[1],
                        n2: vaddr[2],
                        n3: vaddr[3],
                    },
                    port: port,
                },
            },
        };
        let errno = socket_wamr::wamr_sock_connect(fd, &wasi_addr as *const _);
        if errno != 0 {
            Err(io::Error::from_raw_os_error(errno as i32))
        } else {
            Ok(())
        }
    }

    pub fn bind(&self, addrs: &SocketAddr) -> io::Result<()> {
        unsafe {
            let fd = self.as_raw_fd();
            let mut vaddr: [u8; 16] = [0; 16];
            let port;
            let size;
            match addrs {
                SocketAddr::V4(addr) => {
                    let ip = addr.ip().octets();
                    (&mut vaddr[0..4]).clone_from_slice(&ip);
                    port = addr.port();
                    size = 4;
                }
                SocketAddr::V6(addr) => {
                    let ip = addr.ip().octets();
                    vaddr.clone_from_slice(&ip);
                    port = addr.port();
                    size = 16;
                }
            }
            let mut addr = WasiAddress {
                buf: vaddr.as_ptr(),
                size,
            };
            let res = sock_bind(fd as u32, &mut addr, port as u32);
            if res != 0 {
                Err(io::Error::from_raw_os_error(res as i32))
            } else {
                Ok(())
            }
        }
    }

    pub fn listen(&self, backlog: i32) -> io::Result<()> {
        unsafe {
            let fd = self.as_raw_fd();
            let res = sock_listen(fd as u32, backlog as u32);
            if res != 0 {
                Err(io::Error::from_raw_os_error(res as i32))
            } else {
                Ok(())
            }
        }
    }

    pub fn accept(&self, nonblocking: bool) -> io::Result<Self> {
        unsafe {
            let mut fd: u32 = 0;
            let res = sock_accept(self.as_raw_fd() as u32, &mut fd);
            if res != 0 {
                Err(io::Error::from_raw_os_error(res as i32))
            } else {
                let s = Socket { fd: fd as i32 };
                s.set_nonblocking(nonblocking)?;
                Ok(s)
            }
        }
    }

    pub fn get_local(&self) -> io::Result<SocketAddr> {
        let fd: u32 = self.fd as u32;
        let mut wasi_addr = socket_wamr::WasiAddr::default();
        let errno = socket_wamr::wamr_sock_addr_local(fd, &mut wasi_addr);

        if errno != 0 {
            Err(io::Error::from_raw_os_error(errno as i32))
        } else {
            if wasi_addr.kind == socket_wamr::WasiAddrType::IPv4 {
                let addr = unsafe { wasi_addr.addr.ip4.addr };
                let ip_addr = Ipv4Addr::new(addr.n0, addr.n1, addr.n2, addr.n3);
                let port = unsafe { wasi_addr.addr.ip4.port };
                Ok(SocketAddr::V4(SocketAddrV4::new(ip_addr, port)))
            } else if wasi_addr.kind == socket_wamr::WasiAddrType::IPv6 {
                let addr = unsafe { wasi_addr.addr.ip6.addr };
                let port = unsafe { wasi_addr.addr.ip6.port };
                let ip_addr = Ipv6Addr::new(
                    addr.n0, addr.n1, addr.n2, addr.n3, addr.h0, addr.h1, addr.h2, addr.h3,
                );
                Ok(SocketAddr::V6(SocketAddrV6::new(ip_addr, port, 0, 0)))
            } else {
                Err(io::Error::from(io::ErrorKind::Unsupported))
            }
        }
    }

    pub fn get_peer(&self) -> io::Result<SocketAddr> {
        let fd: u32 = self.fd as u32;
        let mut wasi_addr = socket_wamr::WasiAddr::default();
        let errno = socket_wamr::wamr_sock_addr_remote(fd, &mut wasi_addr);

        if errno != 0 {
            Err(io::Error::from_raw_os_error(errno as i32))
        } else {
            if wasi_addr.kind == socket_wamr::WasiAddrType::IPv4 {
                let addr = unsafe { wasi_addr.addr.ip4.addr };
                let ip_addr = Ipv4Addr::new(addr.n0, addr.n1, addr.n2, addr.n3);
                let port = unsafe { wasi_addr.addr.ip4.port };
                Ok(SocketAddr::V4(SocketAddrV4::new(ip_addr, port)))
            } else if wasi_addr.kind == socket_wamr::WasiAddrType::IPv6 {
                let addr = unsafe { wasi_addr.addr.ip6.addr };
                let port = unsafe { wasi_addr.addr.ip6.port };
                let ip_addr = Ipv6Addr::new(
                    addr.n0, addr.n1, addr.n2, addr.n3, addr.h0, addr.h1, addr.h2, addr.h3,
                );
                Ok(SocketAddr::V6(SocketAddrV6::new(ip_addr, port, 0, 0)))
            } else {
                Err(io::Error::from(io::ErrorKind::Unsupported))
            }
        }
    }

    pub fn take_error(&self) -> io::Result<()> {
        Ok(())
        // unsafe {
        //     let fd = self.fd;
        //     let mut error = 0;
        //     let mut len = std::mem::size_of::<i32>() as u32;
        //     let res = sock_getsockopt(
        //         fd as u32,
        //         SocketOptLevel::SolSocket as i32,
        //         SocketOptName::SoError as i32,
        //         &mut error,
        //         &mut len,
        //     );
        //     if res == 0 && error == 0 {
        //         Ok(())
        //     } else if res == 0 && error != 0 {
        //         Err(io::Error::from_raw_os_error(error))
        //     } else {
        //         Err(io::Error::from_raw_os_error(res as i32))
        //     }
        // }
    }

    pub fn is_listener(&self) -> io::Result<bool> {
        unsafe {
            let fd = self.fd;
            let mut val = 0;
            let mut len = std::mem::size_of::<i32>() as u32;
            let res = sock_getsockopt(
                fd as u32,
                SocketOptLevel::SolSocket as i32,
                SocketOptName::SoAcceptconn as i32,
                &mut val,
                &mut len,
            );
            if res != 0 {
                Err(io::Error::from_raw_os_error(res as i32))
            } else {
                Ok(val != 0)
            }
        }
    }

    pub fn r#type(&self) -> io::Result<SocketType> {
        unsafe {
            let fd = self.fd;
            let mut val = 0;
            let mut len = std::mem::size_of::<i32>() as u32;
            let res = sock_getsockopt(
                fd as u32,
                SocketOptLevel::SolSocket as i32,
                SocketOptName::SoType as i32,
                &mut val,
                &mut len,
            );
            if res != 0 {
                Err(io::Error::from_raw_os_error(res as i32))
            } else {
                match val {
                    1 => Ok(SocketType::Datagram),
                    2 => Ok(SocketType::Stream),
                    _ => Err(io::Error::from_raw_os_error(libc::EINVAL)),
                }
            }
        }
    }

    pub fn broadcast(&self) -> io::Result<bool> {
        unsafe {
            let fd = self.fd;
            let mut val = 0;
            let mut len = std::mem::size_of::<i32>() as u32;
            let res = sock_getsockopt(
                fd as u32,
                SocketOptLevel::SolSocket as i32,
                SocketOptName::SoBroadcast as i32,
                &mut val,
                &mut len,
            );
            if res != 0 {
                Err(io::Error::from_raw_os_error(res as i32))
            } else {
                Ok(val != 0)
            }
        }
    }

    pub fn keepalive(&self) -> io::Result<bool> {
        unsafe {
            let fd = self.fd;
            let mut val = 0;
            let mut len = std::mem::size_of::<i32>() as u32;
            let res = sock_getsockopt(
                fd as u32,
                SocketOptLevel::SolSocket as i32,
                SocketOptName::SoKeepalive as i32,
                &mut val,
                &mut len,
            );
            if res != 0 {
                Err(io::Error::from_raw_os_error(res as i32))
            } else {
                Ok(val != 0)
            }
        }
    }

    pub fn recv_buffer_size(&self) -> io::Result<usize> {
        unsafe {
            let fd = self.fd;
            let mut val = 0;
            let mut len = std::mem::size_of::<i32>() as u32;
            let res = sock_getsockopt(
                fd as u32,
                SocketOptLevel::SolSocket as i32,
                SocketOptName::SoRcvbuf as i32,
                &mut val,
                &mut len,
            );
            if res != 0 {
                Err(io::Error::from_raw_os_error(res as i32))
            } else {
                Ok(val as usize)
            }
        }
    }

    pub fn send_buffer_size(&self) -> io::Result<usize> {
        unsafe {
            let fd = self.fd;
            let mut val = 0;
            let mut len = std::mem::size_of::<i32>() as u32;
            let res = sock_getsockopt(
                fd as u32,
                SocketOptLevel::SolSocket as i32,
                SocketOptName::SoSndbuf as i32,
                &mut val,
                &mut len,
            );
            if res != 0 {
                Err(io::Error::from_raw_os_error(res as i32))
            } else {
                Ok(val as usize)
            }
        }
    }

    pub fn reuse_address(&self) -> io::Result<bool> {
        unsafe {
            let fd = self.fd;
            let mut val = 0;
            let mut len = std::mem::size_of::<i32>() as u32;
            let res = sock_getsockopt(
                fd as u32,
                SocketOptLevel::SolSocket as i32,
                SocketOptName::SoReuseaddr as i32,
                &mut val,
                &mut len,
            );
            if res != 0 {
                Err(io::Error::from_raw_os_error(res as i32))
            } else {
                Ok(val != 0)
            }
        }
    }

    pub fn setsockopt<T>(
        &self,
        level: SocketOptLevel,
        name: SocketOptName,
        payload: T,
    ) -> io::Result<()> {
        unsafe {
            let fd = self.fd as u32;
            let flag = &payload as *const T as *const i32;
            let flag_size = std::mem::size_of::<T>() as u32;
            let e = sock_setsockopt(fd, level as u8 as i32, name as u8 as i32, flag, flag_size);
            if e == 0 {
                Ok(())
            } else {
                Err(io::Error::from_raw_os_error(e as i32))
            }
        }
    }

    pub fn setsockopt_socket(&self, name: socket_wamr::SocketOptName) -> io::Result<()> {
        let fd = self.fd as u32;
        let errno = match name {
            socket_wamr::SocketOptName::SoBroadcast(value) => {
                socket_wamr::wamr_sock_set_broadcast(fd, value)
            }
            socket_wamr::SocketOptName::SoKeepalive(value) => {
                socket_wamr::wamr_sock_set_keep_alive(fd, value)
            }
            socket_wamr::SocketOptName::SoRcvbuf(value) => {
                socket_wamr::wamr_sock_set_recv_buf_size(fd, value)
            }
            socket_wamr::SocketOptName::SoReuseaddr(value) => {
                socket_wamr::wamr_sock_set_reuse_addr(fd, value)
            }
            socket_wamr::SocketOptName::SoReuseport(value) => {
                socket_wamr::wamr_sock_set_reuse_port(fd, value)
            }
            socket_wamr::SocketOptName::SoSndbuf(value) => {
                socket_wamr::wamr_sock_set_send_buf_size(fd, value)
            }
            socket_wamr::SocketOptName::SoRcvtimeo(value) => {
                socket_wamr::wamr_sock_set_recv_timeout(fd, value)
            }
            socket_wamr::SocketOptName::SoSndtimeo(value) => {
                socket_wamr::wamr_sock_set_send_timeout(fd, value)
            }
        };
        if errno == 0 {
            Ok(())
        } else {
            Err(io::Error::from_raw_os_error(errno as i32))
        }
    }

    pub fn getsockopt_socket(&self, name: socket_wamr::SocketOptName) -> io::Result<()> {
        let fd = self.fd as u32;
        let errno = match name {
            socket_wamr::SocketOptName::SoBroadcast(value) => {
                socket_wamr::wamr_sock_set_broadcast(fd, value)
            }
            socket_wamr::SocketOptName::SoKeepalive(value) => {
                socket_wamr::wamr_sock_set_keep_alive(fd, value)
            }
            socket_wamr::SocketOptName::SoRcvbuf(value) => {
                socket_wamr::wamr_sock_set_recv_buf_size(fd, value)
            }
            socket_wamr::SocketOptName::SoReuseaddr(value) => {
                socket_wamr::wamr_sock_set_reuse_addr(fd, value)
            }
            socket_wamr::SocketOptName::SoReuseport(value) => {
                socket_wamr::wamr_sock_set_reuse_port(fd, value)
            }
            socket_wamr::SocketOptName::SoSndbuf(value) => {
                socket_wamr::wamr_sock_set_send_buf_size(fd, value)
            }
            socket_wamr::SocketOptName::SoRcvtimeo(value) => {
                socket_wamr::wamr_sock_set_recv_timeout(fd, value)
            }
            socket_wamr::SocketOptName::SoSndtimeo(value) => {
                socket_wamr::wamr_sock_set_send_timeout(fd, value)
            }
        };
        if errno == 0 {
            Ok(())
        } else {
            Err(io::Error::from_raw_os_error(errno as i32))
        }
    }

    pub fn shutdown(&self, how: Shutdown) -> io::Result<()> {
        unsafe {
            let flags = match how {
                Shutdown::Read => 1,
                Shutdown::Write => 2,
                Shutdown::Both => 3,
            };
            let res = sock_shutdown(self.as_raw_fd() as u32, flags);
            if res == 0 {
                Ok(())
            } else {
                Err(io::Error::from_raw_os_error(res as i32))
            }
        }
    }
}

impl Drop for Socket {
    fn drop(&mut self) {
        let _ = self.shutdown(Shutdown::Both);
        unsafe { libc::close(self.fd) };
    }
}

impl AsRawFd for Socket {
    fn as_raw_fd(&self) -> RawFd {
        self.fd
    }
}

impl IntoRawFd for Socket {
    fn into_raw_fd(self) -> RawFd {
        let fd = self.fd;
        std::mem::forget(self);
        fd
    }
}

impl FromRawFd for Socket {
    unsafe fn from_raw_fd(fd: RawFd) -> Self {
        Socket { fd }
    }
}