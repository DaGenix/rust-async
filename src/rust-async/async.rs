// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::io::IoResult;
use std::ptr;
use std::mem;
use std::any::Void;

pub trait AsyncReader {
    fn async_read(&mut self, output: &mut [u8]) -> IoResult<uint>;
}

pub trait AsyncWriter {
    fn async_write(&mut self, input: &[u8]) -> IoResult<uint>;
}




trait Filter<Din, Dout, Uin, Uout> {
    fn down<L: PipelineUp<Uout>, N: PipelineDown<Dout>>(&self, data: Din, last: &L, next: &N);

    fn up<L: PipelineUp<Uout>, N: PipelineDown<Dout>>(&self, data: Uin, last: &L, next: &N);
}

trait PipelineDown<Din> {
    fn down(&self, data: Din);
}

impl <Din> PipelineDown<Din> for ~PipelineDown<Din> {
    fn down(&self, data: Din) {
        self.down(data);
    }
}

impl <'a, Din> PipelineDown<Din> for &'a PipelineDown<Din> {
    fn down(&self, data: Din) {
        self.down(data);
    }
}

trait PipelineUp<Uin> {
    fn up(&self, data: Uin);
}

impl <Uin> PipelineUp<Uin> for ~PipelineUp<Uin> {
    fn up(&self, data: Uin) {
        self.up(data);
    }
}

impl <'a, Uin> PipelineUp<Uin> for &'a PipelineUp<Uin> {
    fn up(&self, data: Uin) {
        self.up(data);
    }
}

struct PipelineStageDown<U, F, D> {
    up: U,
    filter: ~F,
    down: D
}

impl <
        Din, Dout, Uin, Uout,
        F: Filter<Din, Dout, Uin, Uout>,
        U: PipelineUp<Uout>,
        D: PipelineDown<Dout>
     >
        PipelineDown<Din>
        for PipelineStageDown<U, F, D> {
    fn down(&self, data: Din) {
        self.filter.down(data, &self.up, &self.down);
    }
}

struct PipelineStageRef<S> {
    stage: *S
}

impl <
        Din, Dout, Uin, Uout,
        F: Filter<Din, Dout, Uin, Uout>,
        U: PipelineUp<Uout>,
        D: PipelineDown<Dout>
     >
        PipelineUp<Uin>
        for PipelineStageRef<PipelineStageDown<U, F, D>> {
    fn up(&self, data: Uin) {
        let s = unsafe { self.stage.to_option().unwrap() };
        s.filter.up(data, &s.up, &s.down);
    }
}

struct AnyUp<Uin>;

impl <Uin> PipelineUp<Uin> for AnyUp<Uin> {
    fn up(&self, data: Uin) {}
}

struct AnyDown<Din>;

impl <Din> PipelineDown<Din> for AnyDown<Din> {
    fn down(&self, data: Din) {}
}

struct PipelineBuilder<F, N> {
    filter: ~F,
    next: N
}

struct PipelineTerm;

impl <Din, Dout, Uin, Uout, F: Filter<Din, Dout, Uin, Uout>> PipelineBuilder<F, PipelineTerm> {
    fn new(filter: ~F) -> PipelineBuilder<F, PipelineTerm> {
        PipelineBuilder {
            filter: filter,
            next: PipelineTerm
        }
    }
}

impl <
        Din, Dmid, Dout, Uin, Umid, Uout,
        F1: Filter<Din, Dmid, Umid, Uout>,
        F2: Filter<Dmid, Dout, Uin, Umid>,
        X
     >
        PipelineBuilder<F2, X> {
    fn filter(self, filter: ~F1) -> PipelineBuilder<F1, PipelineBuilder<F2, X>> {
        PipelineBuilder {
            filter: filter,
            next: self
        }
    }
}

impl <
        Din, Dout, Uin, Uout: Send,
        F: Filter<Din, Dout, Uin, Uout> + Send,
        D: PipelineDown<Dout> + PipelineSetup<Uin> + Send,
        N: BuildStage<Dout, D>
     >
        PipelineBuilder<F, N> {
    fn build(self) -> ~PipelineDown<Din> {
        let PipelineBuilder {filter, next} = self;

        let any_up: ~AnyUp<Uout> = ~AnyUp;
        let temp_up = any_up as ~PipelineUp<Uout>;

        let mut ps = ~PipelineStageDown {
            up: temp_up,
            filter: filter,
            down: next.build_stage()
        };

        let me = ~PipelineStageRef {
            stage: &*ps
        } as ~PipelineUp<Uin>;

        ps.down.setup(me);

        ps as ~PipelineDown<Din>
    }
}


trait BuildStage<Din, S: PipelineDown<Din>> {
    fn build_stage(self) -> S;
}

impl <
        Din, Dout, Uin, Uout: Send,
        F: Filter<Din, Dout, Uin, Uout> + Send,
        D: PipelineDown<Dout> + PipelineSetup<Uin> + Send,
        N: BuildStage<Dout, D>
     >
        BuildStage<Din, PipelineStageDown<~PipelineUp<Uout>, F, D>>
        for PipelineBuilder<F, N> {
    fn build_stage(self) -> PipelineStageDown<~PipelineUp<Uout>, F, D> {
        let PipelineBuilder {filter, next} = self;

        let any_up: ~AnyUp<Uout> = ~AnyUp;
        let temp_up = any_up as ~PipelineUp<Uout>;

        let ps = PipelineStageDown {
            up: temp_up,
            filter: filter,
            down: next.build_stage()
        };

        ps
    }
}

impl <Din> BuildStage<Din, AnyDown<Din>> for PipelineTerm {
    fn build_stage(self) -> AnyDown<Din> {
        AnyDown
    }
}

trait PipelineSetup<Uout> {
    fn setup(&mut self, up: ~PipelineUp<Uout>);
}

impl <
        Din, Dout, Uin, Uout: Send,
        F: Filter<Din, Dout, Uin, Uout> + Send,
        D: PipelineDown<Dout> + PipelineSetup<Uin> + Send
     >
        PipelineSetup<Uout>
        for PipelineStageDown<~PipelineUp<Uout>, F, D> {
    fn setup(&mut self, up: ~PipelineUp<Uout>) {
        self.up = up;
        let me = ~PipelineStageRef {
            stage: &*self
        } as ~PipelineUp<Uin>;
        self.down.setup(me);
    }
}

impl <X> PipelineSetup<X> for AnyDown<X> {
    fn setup(&mut self, _: ~PipelineUp<X>) {
    }
}



#[main]
fn main() {
    let pipeline = PipelineBuilder::new(~PrintU16Filter)
        .filter(~U32ToU16Filter)
        .filter(~U64ToU32Filter)
        .build();
    pipeline.down(65);
}





















struct U64ToU32Filter;

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

impl Filter<u16, (), (), u16> for PrintU16Filter {
    fn down<L: PipelineUp<u16>, N: PipelineDown<()>>(&self, data: u16, last: &L, next: &N) {
        println!("Value!: {}", data);
        last.up(data);
    }

    fn up<L: PipelineUp<u16>, N: PipelineDown<()>>(&self, data: (), last: &L, next: &N) {
    }
}
