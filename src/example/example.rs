// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#[license = "MIT/ASL2"];
#[crate_id = "github.com/DaGenix/rust-async#rust-async-example:0.1"];

extern crate async = "rust-async";

use async::pipeline::{Filter, PipelineBuilder, PipelineDown, PipelineUp};

struct U64ToU32Filter;

impl Filter<u64, u32, u32, u64> for U64ToU32Filter {
    fn down<U: PipelineUp<u64>, D: PipelineDown<u32>>(&self, data: u64, _: &U, down: &D) {
        println!("hi64");
        down.down(data as u32);
    }

    fn up<U: PipelineUp<u64>, D: PipelineDown<u32>>(&self, data: u32, up: &U, _: &D) {
        up.up(data as u64);
        println!("bai!");
    }
}

struct U32ToU16Filter;

impl Filter<u32, u16, u16, u32> for U32ToU16Filter {
    fn down<U: PipelineUp<u32>, D: PipelineDown<u16>>(&self, data: u32, _: &U, down: &D) {
        println!("hi32");
        down.down(data as u16);
    }

    fn up<U: PipelineUp<u32>, D: PipelineDown<u16>>(&self, data: u16, up: &U, _: &D) {
        println!("Stuck in the middle");
        up.up(data as u32);
    }
}

struct PrintU16Filter;

impl Filter<u16, (), (), u16> for PrintU16Filter {
    fn down<U: PipelineUp<u16>, D: PipelineDown<()>>(&self, data: u16, up: &U, _: &D) {
        println!("Value!: {}", data);
        up.up(data);
    }

    fn up<U: PipelineUp<u16>, D: PipelineDown<()>>(&self, _: (), _: &U, _: &D) {
    }
}

fn main() {
    let pipeline = PipelineBuilder::new(PrintU16Filter)
        .filter(U32ToU16Filter)
        .filter(U64ToU32Filter)
        .build();
    pipeline.down(65);
}
