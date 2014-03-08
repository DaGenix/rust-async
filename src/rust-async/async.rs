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
