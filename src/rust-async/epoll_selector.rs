// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use epoll;
use epoll::epoll_event;

use native;

use select;
use select::{SelectMode, ReadyMode, Selector, SelectEvent, SelectorHandle, SelectNotifier};

use std::libc;
use std::libc::c_int;
use std::io;
use std::io::{IoResult, IoError};

use sync::{Arc, MutexArc};

pub trait EpollSelectable {
    fn get_fd(&self) -> c_int;
}

impl EpollSelectable for native::io::net::TcpAcceptor {
    fn get_fd(&self) -> c_int {
        self.fd()
    }
}

impl EpollSelectable for native::io::net::TcpStream {
    fn get_fd(&self) -> c_int {
        self.fd()
    }
}

struct Epoll<N> {
    epoll_fd: c_int,
    notifier: N
}

fn events_flags(mode: SelectMode, rearm: bool) -> u32 {
    let events = match mode {
        select::SelectRead => epoll::EPOLLIN,
        select::SelectWrite => epoll::EPOLLOUT,
        select::SelectBoth => epoll::EPOLLIN | epoll::EPOLLOUT,
        select::SelectIgnore => 0
    };
    if !rearm {
        events | epoll::EPOLLONESHOT
    } else {
        events
    }
}

fn epoll_ctl_error(res: c_int) -> IoResult<()> {
    let desc = match res {
        libc::EBADF => "Invalid file descriptor.",
        libc::ENOMEM => "There was insufficient memory to handle the requested operation.",
        libc::ENOSPC => "The limit imposed by /proc/sys/fs/epoll/max_user_watches was \
            encountered while trying to register.",
        n if n < 0 => fail!(format!("Unexpected error code {} - this is probably a bug.", n)),
        _ => return Ok(())
    };
    Err(IoError { kind: io::OtherIoError, desc: desc, detail: None })
}

impl <S: EpollSelectable, N> Epoll<N> {
    fn add(
            &self,
            selectable: &S,
            data: u64,
            mode: select::SelectMode,
            rearm: bool) -> IoResult<()> {
        let flags = events_flags(mode, rearm);
        let event = epoll_event { events: flags, data: data };
        let res = unsafe {
            epoll::epoll_ctl(
                self.epoll_fd,
                epoll::EPOLL_CTL_ADD,
                selectable.get_fd(),
                &event)
        };
        epoll_ctl_error(res)
    }

    fn modify(
            &self,
            selectable: &S,
            data: u64,
            mode: select::SelectMode,
            rearm: bool) -> IoResult<()> {
        let flags = events_flags(mode, rearm);
        let event = epoll_event { events: flags, data: data };
        let res = unsafe {
            epoll::epoll_ctl(
                self.epoll_fd,
                epoll::EPOLL_CTL_MOD,
                selectable.get_fd(),
                &event)
        };
        epoll_ctl_error(res)
    }

    fn remove(&self, selectable: &S) -> IoResult<()> {
        let event = epoll_event { events: 0, data: 0 };
        let res = unsafe {
            epoll::epoll_ctl(
                self.epoll_fd,
                epoll::EPOLL_CTL_DEL,
                selectable.get_fd(),
                &event)
        };
        epoll_ctl_error(res)
    }
}

pub struct EpollSelector<N> {
    priv epoll: Arc<Epoll<N>>
}

impl <N: SelectNotifier + Send + Freeze> EpollSelector<N> {
    pub fn new(notifier: N) -> IoResult<EpollSelector<N>> {
        let epoll_fd = unsafe { epoll::epoll_create1(0) };
        match epoll_fd {
            libc::EMFILE => return Err(IoError {
                kind: io::OtherIoError,
                desc: "The per-user limit on the number of epoll instances imposed by \
                    /proc/sys/fs/epoll/max_user_instances was encountered.",
                detail: None
            }),
            libc::ENFILE => return Err(IoError {
                kind: io::OtherIoError,
                desc: "The system limit on the total number of open files has been reached.",
                detail: None
            }),
            libc::ENOMEM => return Err(IoError {
                kind: io::OtherIoError,
                desc: "There was insufficient memory to create the kernel object.",
                detail: None
            }),
            n if n < 0 => fail!(format!("Unexpected error code {} - this is probably a bug.", n)),
            _ => {}
        }
        let epoll = Epoll {
            epoll_fd: epoll_fd,
            notifier: notifier
        };
        let epoll_selector = EpollSelector { epoll: Arc::new(epoll) };
        Ok(epoll_selector)
    }

    pub fn run(&self) {
        let mut events = [epoll_event { events: 0, data: 0}, ..8];
        let mut notify_events = [SelectEvent { mode: select::ReadyRead, data: 0 }, ..8];

        fn is_set(event: u32, flag: u32) -> bool { event & flag != 0 }

        let epoll = self.epoll.get();

        loop {
            let res = unsafe {
                epoll::epoll_wait(epoll.epoll_fd, events.as_ptr(), events.len() as c_int, -1)
            };

            match res {
                libc::EINTR => continue,
                n if n < 0 =>
                    fail!(format!("Unexpected error code {} - this is probably a bug.", n)),
                _ => {}
            }

            for i in range(0, res as uint) {
                notify_events[i].data = events[i].data;

                notify_events[i].mode =
                    if is_set(events[i].events, epoll::EPOLLERR) ||
                            is_set(events[i].events, epoll::EPOLLHUP) {
                        select::ReadyBoth
                    } else if is_set(events[i].events, epoll::EPOLLIN) &&
                            is_set(events[i].events, epoll::EPOLLOUT) {
                        select::ReadyBoth
                    } else if is_set(events[i].events, epoll::EPOLLIN) {
                        select::ReadyRead
                    } else if is_set(events[i].events, epoll::EPOLLOUT) {
                        select::ReadyWrite
                    } else {
                        unreachable!()
                    };
            }

            epoll.notifier.notify(notify_events.slice_to(res as uint));
        }
    }

    pub fn close(&self) {
        fail!("Not yet supported");
    }
}

impl <N: Send + Freeze, S: EpollSelectable> Selector<S, EpollSelectionHandle<N, S>> for EpollSelector<N> {
    fn register(
            &self,
            selectable: S,
            data: u64,
            mode: select::SelectMode,
            rearm: bool) -> IoResult<EpollSelectionHandle<N, S>> {
        try!(self.epoll.get().add(&selectable, data, mode, rearm));
        let h = EpollSelectionHandle {
            epoll: self.epoll.clone(),
            selectable: Some(selectable),
            data: data
        };
        Ok(h)
    }
}

struct EpollSelectionHandle<N, S> {
    epoll: Arc<Epoll<N>>,
    selectable: Option<S>,
    data: u64
}

impl <N: Send + Freeze, S: EpollSelectable> EpollSelectionHandle<N, S> {
    pub fn unwrap(mut self) -> S {
        let selectable = self.selectable.take_unwrap();
        // ignore potential error - is there anything we could do with it?
        let _ = self.epoll.get().remove(&selectable);
        selectable
    }
}

impl <N: Send + Freeze, S: EpollSelectable> SelectorHandle for EpollSelectionHandle<N, S> {
    fn modify(&mut self, mode: SelectMode, rearm: bool) -> IoResult<()> {
        self.epoll.get().modify(self.selectable.get_ref(), self.data, mode, rearm)
    }
}

// TODO - I *think* this is fine. I believe it will no longer be necessary once
// type bounds are allowed on type defintions: https://github.com/mozilla/rust/issues/8142
#[unsafe_destructor]
impl <N: Send + Freeze, S: EpollSelectable> Drop for EpollSelectionHandle<N, S> {
    fn drop(&mut self) {
        // ignore potential error - is there anything we could do with it?
        let _ = self.epoll.get().remove(&self.selectable.take_unwrap());
    }
}
