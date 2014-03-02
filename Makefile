# Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
# http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
# <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
# option. This file may not be copied, modified, or distributed
# except according to those terms.

include rust.mk

RUSTC ?= rustc
MVN ?= mvn
RUSTFLAGS ?= -O

.PHONY : all
all: rust-async

.PHONY : check
check: check-rust-async

.PHONY : clean
clean: clean-rust-async clean-rust-async-example

$(eval $(call RUST_CRATE,1,src/rust-async/lib.rs,rlib,))
$(eval $(call RUST_CRATE,2,src/example/example.rs,bin,-L .))

example: $(1_rust_crate_out) rust-async-example
