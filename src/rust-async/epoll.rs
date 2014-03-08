// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use async::{SelectMode, Read, Write, Both, Selector, SelectNotifier, NotifyEvent};

use std::libc::c_int;
use std::io::IoResult;

use sync::{Arc, MutexArc};

pub static EPOLL_CTL_ADD: c_int = 1;
pub static EPOLL_CTL_MOD: c_int = 3;
pub static EPOLL_CTL_DEL: c_int = 2;

pub static EPOLLIN: u32 = 0x001;
pub static EPOLLOUT: u32 = 0x004;
pub static EPOLLRDHUP: u32 = 0x2000;
pub static EPOLLPRI: u32 = 0x002;
pub static EPOLLERR: u32 = 0x008;
pub static EPOLLHUP: u32 = 0x010;
pub static EPOLLET: u32 = 0x80000000;
pub static EPOLLONESHOT: u32 = 0x40000000;

pub struct epoll_event {
    events: u32,
    data: u64 // This is really a union of size max(u64, void*)
}

extern {
    pub fn epoll_create1(flags: c_int) -> c_int;

    pub fn epoll_ctl(epfd: c_int, op: c_int, fd: c_int, event: *epoll_event) -> c_int;

    pub fn epoll_wait(
        epfd: c_int,
        events: *epoll_event,
        maxevents: c_int,
        timeout: c_int) -> c_int;
}

pub struct EpollSelector<N> {
    priv epoll_fd: c_int,
    priv notifier: N
}

impl <N: SelectNotifier> EpollSelector<N> {
    pub fn new(notifier: N) -> IoResult<EpollSelector<N>> {
        let epoll_fd: c_int;
        unsafe { epoll_fd = epoll_create1(0); }
        Ok(EpollSelector {
            epoll_fd: epoll_fd,
            notifier: notifier
        })
    }

    pub fn run(&self) {
        let mut events = [epoll_event { events: 0, data: 0}, ..8];
        let mut notify_events = [NotifyEvent { mode: Read, data: 0 }, ..8];

        loop {
            let res = unsafe { epoll_wait(self.epoll_fd, events.as_ptr(), events.len() as c_int, -1) };

            if res <= 0 {
                fail!("Unexpected return code from epoll_wait: {}", res);
            }

            // Copying the events isn't free, but its cheaper than acquiring a lock
            // multiple times (which might occur when calling the notifier)
            // TODO - Is this worth it? In the common case, there will probably only be a single
            // event.
            for i in range(0, res as uint) {
                notify_events[i].data = events[i].data;

                // TODO - this isn't quite right
                notify_events[i].mode = match events[i].events {
                    _ => Both
                };
            }

            self.notifier.notify(notify_events.slice_to(res as uint));
        }
    }

    pub fn close(&self) {
        fail!("Not yet supported");
    }
}

fn events_flag(once_: bool, mode: SelectMode) -> u32 {
    let events = match mode {
        Read => EPOLLIN,
        Write => EPOLLOUT,
        Both => EPOLLIN | EPOLLOUT
    };
    if once_ {
        events | EPOLLONESHOT
    } else {
        events
    }
}

impl <N> Selector for EpollSelector<N> {
    fn add(&self, once_: bool, fd: c_int, data: u64, mode: SelectMode) -> IoResult<()> {
        let events = events_flag(once_, mode);
        let event = epoll_event { events: events, data: data };
        unsafe { epoll_ctl(self.epoll_fd, EPOLL_CTL_ADD, fd, &event); }
        Ok(())
    }

    fn modify(&self, once_: bool, fd: c_int, data: u64, mode: SelectMode) -> IoResult<()> {
        let events = events_flag(once_, mode);
        let event = epoll_event { events: events, data: data };
        unsafe { epoll_ctl(self.epoll_fd, EPOLL_CTL_MOD, fd, &event); }
        Ok(())
    }

    fn remove(&self, fd: c_int) -> IoResult<()> {
        unsafe { epoll_ctl(self.epoll_fd, EPOLL_CTL_MOD, fd, 0 as *epoll_event); }
        Ok(())
    }
}
