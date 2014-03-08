// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::io::IoResult;

pub enum SelectMode {
    SelectRead,
    SelectWrite,
    SelectBoth,
    SelectIgnore
}

pub enum ReadyMode {
    ReadyRead,
    ReadyWrite,
    ReadyBoth
}

#[must_use]
pub trait SelectorHandle {
    fn modify(&mut self, mode: SelectMode, rearm: bool) -> IoResult<()>;
}

pub trait Selector<S, H: SelectorHandle> {
    fn register(&self, selectable: S, data: u64, mode: SelectMode, rearm: bool) -> IoResult<H>;
}

pub struct SelectEvent {
    mode: ReadyMode,
    data: u64
}

pub trait SelectNotifier {
    fn notify(&self, events: &[SelectEvent]);
}
