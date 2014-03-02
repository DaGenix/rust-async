// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! This module contains basic traits necessary for all asynchronous code.

use std::io::IoResult;

/// An asynchronous reader.
pub trait AsyncReader {
    fn async_read(&mut self, output: &mut [u8]) -> IoResult<uint>;
}

/// An asynchronous writer.
pub trait AsyncWriter {
    fn async_write(&mut self, input: &[u8]) -> IoResult<uint>;
}
