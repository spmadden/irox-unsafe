use std::future::Future;
use std::net::TcpStream;
use std::os::windows::io::{AsRawSocket, RawSocket};
use std::pin::Pin;
use std::task::{Context, Poll};

use windows::core::PSTR;
use windows::Win32::Networking::WinSock::*;
use windows::Win32::System::IO::OVERLAPPED;

use crate::error::Error;

pub struct AsyncSocket {
    pub socket: TcpStream,
    pub raw_sock: RawSocket,
}

impl AsyncSocket {
    pub fn new(socket: TcpStream) -> Self {
        Self {
            raw_sock: socket.as_raw_socket(),
            socket,
        }
    }

    pub fn write<'a>(&self, buf: &'a mut [u8]) -> Result<NetFuture<'a>, Error> {
        let hEvent = unsafe { WSACreateEvent()? };
        let mut buf = Pin::new(buf);
        let mut overlapped = Box::pin(OVERLAPPED {
            Internal: 0,
            InternalHigh: 0,
            Anonymous: Default::default(),
            hEvent,
        });
        let lpoverlapped = Some(overlapped.as_mut().get_mut() as *mut OVERLAPPED);
        let lpcompletionroutine = None;
        let lpnumberofbytessent = None;
        let dwflags = 0;
        let socket = SOCKET(self.raw_sock as usize);
        let lpbuffers = &[WSABUF {
            len: buf.len() as u32,
            buf: PSTR(buf.as_mut_ptr()),
        }];
        let res = unsafe {
            WSASend(
                socket,
                lpbuffers,
                lpnumberofbytessent,
                dwflags,
                lpoverlapped,
                lpcompletionroutine,
            )
        };
        if res != 0 {
            return Err(WSA_ERROR(res).into());
        }
        Ok(NetFuture {
            _buffer: buf,
            socket,
            overlapped,
            done: None,
        })
    }

    pub fn read<'a>(&self, _buf: &'a mut [u8]) -> Result<NetFuture<'a>, Error> {
        todo!()
    }
}

pub struct NetFuture<'a> {
    _buffer: Pin<&'a mut [u8]>,
    socket: SOCKET,
    overlapped: Pin<Box<OVERLAPPED>>,
    done: Option<u32>,
}

impl Future for NetFuture<'_> {
    type Output = Result<u32, Error>;

    fn poll(mut self: Pin<&mut Self>, ctx: &mut Context<'_>) -> Poll<Self::Output> {
        if let Some(done) = &self.done {
            return Poll::Ready(Ok(*done));
        }
        let mut transferred: u32 = 0;
        let mut lpdwflags = 0;
        let ret = unsafe {
            WSAGetOverlappedResult(
                self.socket,
                self.overlapped.as_mut().get_mut(),
                &mut transferred,
                false,
                &mut lpdwflags,
            )
        };
        match ret {
            Ok(()) => {
                // it's done.
                self.done = Some(transferred);
                Poll::Ready(Ok(transferred))
            }
            Err(_e) => {
                // maybe not done, maybe done with error.
                let err: Error = unsafe { WSAGetLastError() }.into();
                if err.is_ioincomplete() {
                    ctx.waker().wake_by_ref();
                    return Poll::Pending;
                }
                self.done = Some(0);
                Poll::Ready(Err(err))
            }
        }
    }
}
impl Drop for NetFuture<'_> {
    fn drop(&mut self) {
        unsafe {
            if let Err(_e) = WSACloseEvent(self.overlapped.hEvent) {
                // check?
            }
        }
    }
}
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct SocketOption {
    pub level: u32,
    pub optname: u32,
}
impl SocketOption {
    const fn new(level: u32, optname: u32) -> Self {
        Self { level, optname }
    }
}
pub const SOL_SOCKET: u32 = 0xffff;
pub const SO_REUSEADDR: u32 = 0x0004;
pub const SO_KEEPALIVE: u32 = 0x0008;
pub const SO_DONTROUTE: u32 = 0x0010;
pub const SO_BROADCAST: u32 = 0x0020;
pub const SO_USELOOPBACK: u32 = 0x0040;
pub const SO_LINGER: u32 = 0x0080;

pub const OPT_REUSEADDR: SocketOption = SocketOption::new(SOL_SOCKET, SO_REUSEADDR);

pub const SO_SNDBUF: u32 = 0x1001;
pub const SO_RCVBUF: u32 = 0x1002;
pub const SO_RCVTIMEO: u32 = 0x1006;
pub const SO_ERROR: u32 = 0x1007;
pub const SO_TYPE: u32 = 0x1008;

pub const IPP_IP: u32 = 0;
pub const IPP_IPV6: u32 = 100;
pub const IPP_UNSPEC: u32 = 0;
pub const IPP_TCP: u32 = 6;
pub const IPP_UDP: u32 = 17;

pub const IP_TOS: SocketOption = SocketOption::new(IPP_IP, 3);
pub const IP_TTL: SocketOption = SocketOption::new(IPP_IP, 4);
pub const IP_MTU_DISCOVER: SocketOption = SocketOption::new(IPP_IP, 71);
pub const IP_MTU: SocketOption = SocketOption::new(IPP_IP, 73);

pub trait SockOpts<Opt, Typ> {
    fn set(&mut self, value: Typ) -> Result<(), Error>;
    fn get(&mut self) -> Result<Typ, Error>;
}
pub fn getsockopt<S: AsRawSocket>(
    s: &mut S,
    level: u32,
    optname: u32,
    res: &mut [u8],
) -> Result<(), Error> {
    let mut optlen = res.len() as i32;
    let raw = s.as_raw_socket();
    let sock = SOCKET(raw as usize);
    let level = level as i32;
    let optname = optname as i32;
    let res = PSTR(res.as_mut_ptr());
    unsafe {
        let res =
            windows::Win32::Networking::WinSock::getsockopt(sock, level, optname, res, &mut optlen);
        if res != 0 {
            return Err(WSA_ERROR(res).into());
        }
    }
    Ok(())
}
pub fn getsockopt_u32<S: AsRawSocket>(s: &mut S, level: u32, optname: u32) -> Result<u32, Error> {
    let mut res = [0u8; 4];
    getsockopt(s, level, optname, &mut res)?;
    Ok(u32::from_le_bytes(res))
}
pub fn setsockopt<S: AsRawSocket>(
    s: &mut S,
    level: u32,
    optname: u32,
    optval: Option<&[u8]>,
) -> Result<(), Error> {
    let raw = s.as_raw_socket();
    let sock = SOCKET(raw as usize);
    let level = level as i32;
    let optname = optname as i32;

    unsafe {
        let res = windows::Win32::Networking::WinSock::setsockopt(sock, level, optname, optval);
        if res != 0 {
            return Err(WSA_ERROR(res).into());
        }
    }
    Ok(())
}
pub fn setsockopt_u32<S: AsRawSocket>(
    s: &mut S,
    level: u32,
    optname: u32,
    optval: Option<u32>,
) -> Result<(), Error> {
    let optval = optval.map(u32::to_le_bytes);
    setsockopt(s, level, optname, optval.as_ref().map(<[u8; 4]>::as_slice))?;
    Ok(())
}
pub struct SoRecvBuf;
impl<T> SockOpts<SoRecvBuf, u32> for T
where
    T: AsRawSocket,
{
    fn set(&mut self, value: u32) -> Result<(), Error> {
        setsockopt_u32(self, SOL_SOCKET, SO_RCVBUF, Some(value))
    }

    fn get(&mut self) -> Result<u32, Error> {
        getsockopt_u32(self, SOL_SOCKET, SO_RCVBUF)
    }
}
pub struct SoSendBuf;
impl<T> SockOpts<SoSendBuf, u32> for T
where
    T: AsRawSocket,
{
    fn set(&mut self, value: u32) -> Result<(), Error> {
        setsockopt_u32(self, SOL_SOCKET, SO_SNDBUF, Some(value))
    }

    fn get(&mut self) -> Result<u32, Error> {
        getsockopt_u32(self, SOL_SOCKET, SO_SNDBUF)
    }
}

#[cfg(test)]
mod tests {
    use crate::error::Error;
    use crate::net::{SoRecvBuf, SoSendBuf, SockOpts};
    use std::net::UdpSocket;

    #[test]
    pub fn test_getrecvbuf() -> Result<(), Error> {
        let mut sock = UdpSocket::bind("127.0.0.1:0")?;
        let recvbuf = SockOpts::<SoRecvBuf, _>::get(&mut sock)?;
        println!("recvbuf: {recvbuf}");
        assert_ne!(0, recvbuf);
        Ok(())
    }

    #[test]
    pub fn test_getsendbuf() -> Result<(), Error> {
        let mut sock = UdpSocket::bind("127.0.0.1:0")?;
        let sendbuf = SockOpts::<SoSendBuf, _>::get(&mut sock)?;
        println!("sendbuf: {sendbuf}");
        assert_ne!(0, sendbuf);
        Ok(())
    }
}
