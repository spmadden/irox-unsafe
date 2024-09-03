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

impl<'a> Future for NetFuture<'a> {
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
impl<'a> Drop for NetFuture<'a> {
    fn drop(&mut self) {
        unsafe {
            if let Err(_e) = WSACloseEvent(self.overlapped.hEvent) {
                // check?
            }
        }
    }
}
