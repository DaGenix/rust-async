// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#[license = "MIT/ASL2"];
#[crate_id = "github.com/DaGenix/rust-async-example#rust-async-example:0.1"];

extern crate async = "rust-async";

use async::pipeline::{Filter, PipelineBuilder, PipelineDown, PipelineUp};

struct U64ToU32Filter;

#[allow(unused_variable)]
impl Filter<u64, u32, u32, u64> for U64ToU32Filter {
    fn down<L: PipelineUp<u64>, N: PipelineDown<u32>>(&self, data: u64, last: &L, next: &N) {
        println!("hi64");
        next.down(data as u32);
    }

    fn up<L: PipelineUp<u64>, N: PipelineDown<u32>>(&self, data: u32, last: &L, next: &N) {
        last.up(data as u64);
        println!("bai!");
    }
}

struct U32ToU16Filter;

#[allow(unused_variable)]
impl Filter<u32, u16, u16, u32> for U32ToU16Filter {
    fn down<L: PipelineUp<u32>, N: PipelineDown<u16>>(&self, data: u32, last: &L, next: &N) {
        println!("hi32");
        next.down(data as u16);
    }

    fn up<L: PipelineUp<u32>, N: PipelineDown<u16>>(&self, data: u16, last: &L, next: &N) {
        println!("Stuck in the middle");
        last.up(data as u32);
    }
}

struct PrintU16Filter;

#[allow(unused_variable)]
impl Filter<u16, (), (), u16> for PrintU16Filter {
    fn down<L: PipelineUp<u16>, N: PipelineDown<()>>(&self, data: u16, last: &L, next: &N) {
        println!("Value!: {}", data);
        last.up(data);
    }

    fn up<L: PipelineUp<u16>, N: PipelineDown<()>>(&self, data: (), last: &L, next: &N) {
    }
}

fn main() {
    let pipeline = PipelineBuilder::new(~PrintU16Filter)
        .filter(~U32ToU16Filter)
        .filter(~U64ToU32Filter)
        .build();
    pipeline.down(65);
}
