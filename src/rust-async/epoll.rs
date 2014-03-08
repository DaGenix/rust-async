// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::libc::c_int;

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
