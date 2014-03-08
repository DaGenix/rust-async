// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! This module contains basic traits necessary for all asynchronous code.

// use native::net::{TcpStream, TcpListener, TcpAcceptor};

use std::libc::c_int;

use std::io::IoResult;
use std::clone::Clone;
use std::io::net::ip::SocketAddr;
use std::io::{Reader, Writer, Listener, Acceptor};
use std::io::IoResult;
use std::rt::rtio::{IoFactory, LocalIo, RtioSocket, RtioTcpListener};
use std::rt::rtio::{RtioTcpAcceptor, RtioTcpStream};

/// An asynchronous reader.
pub trait AsyncReader {
    fn async_read(&mut self, output: &mut [u8]) -> IoResult<uint>;
}

/// An asynchronous writer.
pub trait AsyncWriter {
    fn async_write(&mut self, input: &[u8]) -> IoResult<uint>;
}

pub trait AsyncListener<T, A: Acceptor<T>> {
    fn listen(self) -> IoResult<A>;
}

pub trait AsyncAcceptor<T> {
    fn accept(&mut self) -> IoResult<T>;
}




// Holds a set of pipelines. Each has a u64 id value.
// Accessed via a MutexArc
// Also holds a task pool
// When events are received, the relevent pipeline is looked up by id
// if the pipeline is currently available, the event is processed on a task thread
// if the pipeline is not currently available, the event is queued
// if the pipeline is closed, the event is dropped
// > PipelineCentral;

// Waits for an event, then dispatches it
// threadsafe - may be acceesed via an Arc
// > EpollWaiter;

pub enum SelectMode {
    Read,
    Write,
    Both
}

pub trait Selector {
    fn add(&self, once_: bool, fd: c_int, data: u64, mode: SelectMode) -> IoResult<()>;
    fn modify(&self, once_: bool, fd: c_int, data: u64, mode: SelectMode) -> IoResult<()>;
    fn remove(&self, fd: c_int) -> IoResult<()>;
}

pub trait SelectNotifier {
    fn notify(&self, events: &[NotifyEvent]);
}

pub struct NotifyEvent {
    mode: SelectMode,
    data: u64
}







// Structure that listens for readyness of sockets


// Structure that listens for Chan to complete



// various Waiters wait for events to occur. When an event occurs,
// it sends the event over a channel to the AsyncEngine. The AsyncEngine
// finds the pipeline and then sends it to a worker thread to handle the event.
// The worker thread takes ownership of the pipeline. When it completes processing, it returns
// it to the AsyncEngine over the same pipe as any other event.
// If an event shows up, but the pipeline it references is currently being operated on,
// the event is queued up and is not executed until the pipeline is returned.
/*
struct AsyncEngine {

}






pub struct TcpStream {
    priv obj: ~RtioTcpStream
}

impl TcpStream {
    fn new(s: ~RtioTcpStream) -> TcpStream {
        TcpStream { obj: s }
    }

    pub fn connect(addr: SocketAddr) -> IoResult<TcpStream> {
        LocalIo::maybe_raise(|io| {
            io.tcp_connect(addr).map(TcpStream::new)
        })
    }

    pub fn peer_name(&mut self) -> IoResult<SocketAddr> {
        self.obj.peer_name()
    }

    pub fn socket_name(&mut self) -> IoResult<SocketAddr> {
        self.obj.socket_name()
    }
}

impl Clone for TcpStream {
    fn clone(&self) -> TcpStream {
        TcpStream { obj: self.obj.clone() }
    }
}

impl Reader for TcpStream {
    fn read(&mut self, buf: &mut [u8]) -> IoResult<uint> { self.obj.read(buf) }
}

impl Writer for TcpStream {
    fn write(&mut self, buf: &[u8]) -> IoResult<()> { self.obj.write(buf) }
}

/// A structure representing a socket server. This listener is used to create a
/// `TcpAcceptor` which can be used to accept sockets on a local port.
///
/// # Example
///
/// ```rust
/// # fn main() {}
/// # fn foo() {
/// # #[allow(unused_must_use, dead_code)];
/// use std::io::net::tcp::TcpListener;
/// use std::io::net::ip::{Ipv4Addr, SocketAddr};
/// use std::io::{Acceptor, Listener};
///
/// let addr = SocketAddr { ip: Ipv4Addr(127, 0, 0, 1), port: 80 };
/// let listener = TcpListener::bind(addr);
///
/// // bind the listener to the specified address
/// let mut acceptor = listener.listen();
///
/// // accept connections and process them
/// # fn handle_client<T>(_: T) {}
/// for stream in acceptor.incoming() {
///     spawn(proc() {
///         handle_client(stream);
///     });
/// }
///
/// // close the socket server
/// drop(acceptor);
/// # }
/// ```
pub struct TcpListener {
    priv obj: ~RtioTcpListener
}

impl TcpListener {
    /// Creates a new `TcpListener` which will be bound to the specified local
    /// socket address. This listener is not ready for accepting connections,
    /// `listen` must be called on it before that's possible.
    ///
    /// Binding with a port number of 0 will request that the OS assigns a port
    /// to this listener. The port allocated can be queried via the
    /// `socket_name` function.
    pub fn bind(addr: SocketAddr) -> IoResult<TcpListener> {
        LocalIo::maybe_raise(|io| {
            io.tcp_bind(addr).map(|l| TcpListener { obj: l })
        })
    }

    /// Returns the local socket address of this listener.
    pub fn socket_name(&mut self) -> IoResult<SocketAddr> {
        self.obj.socket_name()
    }
}

impl Listener<TcpStream, TcpAcceptor> for TcpListener {
    fn listen(self) -> IoResult<TcpAcceptor> {
        self.obj.listen().map(|acceptor| TcpAcceptor { obj: acceptor })
    }
}

/// The accepting half of a TCP socket server. This structure is created through
/// a `TcpListener`'s `listen` method, and this object can be used to accept new
/// `TcpStream` instances.
pub struct TcpAcceptor {
    priv obj: ~RtioTcpAcceptor
}

impl Acceptor<TcpStream> for TcpAcceptor {
    fn accept(&mut self) -> IoResult<TcpStream> {
        self.obj.accept().map(TcpStream::new)
    }
}
*/
