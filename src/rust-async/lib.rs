// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#[license = "MIT/ASL2"];
#[crate_id = "github.com/DaGenix/rust-async#rust-async:0.1"];

extern crate extra;
extern crate native;
extern crate sync;

use std::libc::c_int;
use std::io::net::ip::SocketAddr;
use std::from_str::FromStr;

use sync::MutexArc;

use async::{Selector, Read, SelectNotifier, NotifyEvent};
use epoll::{epoll_create1, epoll_ctl, epoll_wait, epoll_event, EpollSelector};

use native::io::net::TcpListener;

pub mod async;
pub mod epoll;
pub mod pipeline;

// fn main() {
//     let epoll_fd: c_int;
//     unsafe { epoll_fd = epoll_create1(0); }
//
//     let localhost: SocketAddr = FromStr::from_str("127.0.0.1:2323").unwrap();
//     let listener = TcpListener::bind(localhost).unwrap().native_listen(128).unwrap();
//
//     let event = epoll_event { events: (epoll::EPOLLIN | epoll::EPOLLOUT) as u32, data: 0 };
//     unsafe { epoll_ctl(epoll_fd, epoll::EPOLL_CTL_ADD, listener.fd(), &event); }
//
//     let events = [epoll_event { events: 0, data: 0}, ..8];
//
//     unsafe {
//         let res = epoll_wait(epoll_fd, events.as_ptr(), 8, -1);
//         println!("res: {}", res);
//     }
// }

struct MyNotifier;

impl SelectNotifier for MyNotifier {
    fn notify(&self, events: &[NotifyEvent]) {
        println!("Ready!");
    }
}

fn main() {
    let localhost: SocketAddr = FromStr::from_str("127.0.0.1:2323").unwrap();
    let listener = TcpListener::bind(localhost).unwrap().native_listen(128).unwrap();

    let epoll = EpollSelector::new(MyNotifier).unwrap();
    epoll.add(true, listener.fd(), 42, Read);
    epoll.run();
}
