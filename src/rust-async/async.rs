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

struct PipelineStageUp<U, F, D> {
    up: U,
    filter: *F,
    down: D
}

impl <
        Din, Dout, Uin, Uout,
        F: Filter<Din, Dout, Uin, Uout>,
        U: PipelineUp<Uout>,
        D: PipelineDown<Dout>
     >
        PipelineUp<Uin>
        for PipelineStageUp<U, F, D> {
    fn up(&self, data: Uin) {
        unsafe { self.filter.to_option() }.unwrap().up(data, &self.up, &self.down);
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
        D: PipelineDown<Dout> + Send,
        N: BuildStage<Dout, D>
     >
        PipelineBuilder<F, N> {
    fn build(self) -> ~PipelineDown<Din> {
        let PipelineBuilder {filter, next} = self;

        let any_up: ~AnyUp<Uout> = ~AnyUp;
        let up = any_up as ~PipelineUp<Uout>;

        let ps = ~PipelineStageDown {
            up: up,
            filter: filter,
            down: next.build_stage()
        };
//        let me: *PipelineStageDown<Void, F, D> = &*ps;
//        ps.setup(me);
        ps as ~PipelineDown<Din>
    }
}



/*
struct PipelineDownRef<D> {
    pipeline_down: *D
}

impl <Din> PipelineDown<Din> for PipelineDownRef<?> {

}
*/

struct Pipeline<U, D> {
    up_head: U,
    down_head: D
}





trait BuildStage<Din, S: PipelineDown<Din>> {
    fn build_stage(self) -> S;
}

impl <
        Din, Dout, Uin, Uout,
        F: Filter<Din, Dout, Uin, Uout> + Send,
        D: PipelineDown<Dout> + Send,
        N: BuildStage<Dout, D>
     >
        BuildStage<Din, PipelineStageDown<~PipelineUp<Uout>, F, D>>
        for PipelineBuilder<F, N> {
    fn build_stage(self) -> PipelineStageDown<~PipelineUp<Uout>, F, D> {
        let PipelineBuilder {filter, next} = self;

        let any_up: ~AnyUp<Uout> = ~AnyUp;
        let up = any_up as ~PipelineUp<Uout>;

        let ps = PipelineStageDown {
            up: up,
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

/*
trait PipelineSetup<Uout, U: PipelineUp<Uout>> {
    fn setup(&mut self, up: *U);
}

impl <
        Din, Dout, Uin, Uout,
        F: Filter<Din, Dout, Uin, Uout>,
        U: PipelineUp<Uout>,
        D: PipelineSetup<Uin, PipelineStageDown<U, F, D>>
     >
        PipelineSetup<Uin, PipelineStageDown<U, F, D>>
        for PipelineStageDown<U, F, D> {
    fn setup(&mut self, up: *U) {
        self.up = up;
        let me: *PipelineStageDown<U, F, D> = self;
        self.down.setup(me);
    }
}
*/

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
    }
}

struct U32ToU16Filter;

impl Filter<u32, u16, u16, u32> for U32ToU16Filter {
    fn down<L: PipelineUp<u32>, N: PipelineDown<u16>>(&self, data: u32, last: &L, next: &N) {
        println!("hi32");
        next.down(data as u16);
    }

    fn up<L: PipelineUp<u32>, N: PipelineDown<u16>>(&self, data: u16, last: &L, next: &N) {
        last.up(data as u32);
    }
}

struct PrintU16Filter;

impl Filter<u16, (), (), u16> for PrintU16Filter {
    fn down<L: PipelineUp<u16>, N: PipelineDown<()>>(&self, data: u16, last: &L, next: &N) {
        println!("Value!: {}", data);
    }

    fn up<L: PipelineUp<u16>, N: PipelineDown<()>>(&self, data: (), last: &L, next: &N) {
    }
}




















/*


trait PipelineDown<Din> {
    fn down(&self, data: Din);
}

trait PipelineUp<Uin> {
    fn up(&self, data: Uin);
}

impl <Din, Dout, Uin, Uout, C: Filter<Din, Dout, Uin, Uout>, N: PipelineDown<Dout>> PipelineDown<Din> for PipelineStage<C, N> {
    fn down(&self, data: Din) {
        self.current.down(data, &AnyUp, &self.next)
    }
}

impl <Din, Dout, Uin, Uout, C: Filter<Din, Dout, Uin, Uout>, N: PipelineDown<Dout>> PipelineUp<Uin> for PipelineStage<C, N> {
    fn up(&self, data: Uin) {
        self.current.up(data, &AnyUp, &self.next)
    }
}

struct AnyUp;

impl <X> PipelineUp<X> for AnyUp {
    fn up(&self, data: X) { }
}

struct AnyDown;

impl <X> PipelineDown<X> for AnyDown {
    fn down(&self, data: X) { }
}

struct NullUp;

impl PipelineUp<()> for NullUp {
    fn up(&self, data: ()) { }
}

struct NullDown;

impl PipelineDown<()> for NullDown {
    fn down(&self, data: ()) { }
}

trait Filter<Din, Dout, Uin, Uout> {
    fn down<L: PipelineUp<Uout>, N: PipelineDown<Dout>>(&self, data: Din, last: &L, next: &N);

    fn up<L: PipelineUp<Uout>, N: PipelineDown<Dout>>(&self, data: Uin, last: &L, next: &N);
}

struct U64ToU32Filter;

impl Filter<u64, u32, u32, u64> for U64ToU32Filter {
    fn down<L: PipelineUp<u64>, N: PipelineDown<u32>>(&self, data: u64, last: &L, next: &N) {
        next.down(data as u32);
    }

    fn up<L: PipelineUp<u64>, N: PipelineDown<u32>>(&self, data: u32, last: &L, next: &N) {
        last.up(data as u64);
    }
}

struct U32ToU16Filter;

impl Filter<u32, u16, u16, u32> for U32ToU16Filter {
    fn down<L: PipelineUp<u32>, N: PipelineDown<u16>>(&self, data: u32, last: &L, next: &N) {
        next.down(data as u16);
    }

    fn up<L: PipelineUp<u32>, N: PipelineDown<u16>>(&self, data: u16, last: &L, next: &N) {
        last.up(data as u32);
    }
}

struct PrintU16Filter;

impl Filter<u16, (), (), u32> for PrintU16Filter {
    fn down<L: PipelineUp<u16>, N: PipelineDown<()>>(&self, data: u16, last: &L, next: &N) {
        println!("Value!: {}", data);
    }

    fn up<L: PipelineUp<u16>, N: PipelineDown<()>>(&self, data: (), last: &L, next: &N) {
    }
}

*/





























/*












struct PipelineStage<C, N> {
    current: C,
    next: N
}

impl <Din, Dout, Uin, Uout, C: Filter<Din, Dout, Uin, Uout>> PipelineStage<C, AnyDown> {
    fn new(handler: C) -> PipelineStage<C, AnyDown> {
        PipelineStage {
            current: handler,
            next: AnyDown
        }
    }
}

// impl <C1, C2, N> PipelineStage<C1, N> {
//     fn filter<F>(self, next: C2) -> PipelineStage<C1, PipelineStage<C2, AnyDown>> {
//
//     }
// }

// impl <L, C, N> PipelineStage<L, C, N> {
//     fn stage(current: C, next: N) -> PipelineStage<L, C, N> {
//         PipelineStage {
//             last: ptr::null(),
//             current: current,
//             next: next
//         }
//     }
// }

trait Test<F> {
    fn test(&self, data: F);
}

struct TestS;

impl Test<int> for TestS {
    fn test(&self, data: int) {
        println!("value: {}", data);
    }
}



#[main]
fn main() {
    use std::ptr;

    let t = ~TestS as ~Test<int>;
    t.test(43);

//    let mut u16printer = PipelineStage::handler(PrintU16Filter);
//    let mut u32tou16 = PipelineStage::stage(U32ToU16Filter, u16printer);
//    let mut pipeline = PipelineStage::stage(U64ToU32Filter, u32tou16);

    let pipeline = PipelineStage::new(PrintU16Filter);

    pipeline.down(56u16);
}






// trait <X, L: PipelineUp<X>> PipelineInit<L> {
//     fn init(&mut self, last: *L);
// }

// impl <L, C, N> PipelineInit<Y> for PipelineStage<L, C, N> {
//
// }




trait PipelineDown<Din> {
    fn down(&self, data: Din);
}

trait PipelineUp<Uin> {
    fn up(&self, data: Uin);
}

impl <Din, Dout, Uin, Uout, C: Filter<Din, Dout, Uin, Uout>, N: PipelineDown<Dout>> PipelineDown<Din> for PipelineStage<C, N> {
    fn down(&self, data: Din) {
        self.current.down(data, &AnyUp, &self.next)
    }
}

impl <Din, Dout, Uin, Uout, C: Filter<Din, Dout, Uin, Uout>, N: PipelineDown<Dout>> PipelineUp<Uin> for PipelineStage<C, N> {
    fn up(&self, data: Uin) {
        self.current.up(data, &AnyUp, &self.next)
    }
}

struct AnyUp;

impl <X> PipelineUp<X> for AnyUp {
    fn up(&self, data: X) { }
}

struct AnyDown;

impl <X> PipelineDown<X> for AnyDown {
    fn down(&self, data: X) { }
}

struct NullUp;

impl PipelineUp<()> for NullUp {
    fn up(&self, data: ()) { }
}

struct NullDown;

impl PipelineDown<()> for NullDown {
    fn down(&self, data: ()) { }
}

trait Filter<Din, Dout, Uin, Uout> {
    fn down<L: PipelineUp<Uout>, N: PipelineDown<Dout>>(&self, data: Din, last: &L, next: &N);

    fn up<L: PipelineUp<Uout>, N: PipelineDown<Dout>>(&self, data: Uin, last: &L, next: &N);
}

struct U64ToU32Filter;

impl Filter<u64, u32, u32, u64> for U64ToU32Filter {
    fn down<L: PipelineUp<u64>, N: PipelineDown<u32>>(&self, data: u64, last: &L, next: &N) {
        next.down(data as u32);
    }

    fn up<L: PipelineUp<u64>, N: PipelineDown<u32>>(&self, data: u32, last: &L, next: &N) {
        last.up(data as u64);
    }
}

struct U32ToU16Filter;

impl Filter<u32, u16, u16, u32> for U32ToU16Filter {
    fn down<L: PipelineUp<u32>, N: PipelineDown<u16>>(&self, data: u32, last: &L, next: &N) {
        next.down(data as u16);
    }

    fn up<L: PipelineUp<u32>, N: PipelineDown<u16>>(&self, data: u16, last: &L, next: &N) {
        last.up(data as u32);
    }
}

struct PrintU16Filter;

impl Filter<u16, (), (), u32> for PrintU16Filter {
    fn down<L: PipelineUp<u16>, N: PipelineDown<()>>(&self, data: u16, last: &L, next: &N) {
        println!("Value!: {}", data);
    }

    fn up<L: PipelineUp<u16>, N: PipelineDown<()>>(&self, data: (), last: &L, next: &N) {
    }
}


*/

// #[main]
// fn main() {
//     use std::ptr;
//
//     let mut u16printer = PipelineStage::handler(PrintU16Filter);
//     let mut u32tou16 = PipelineStage::stage(U32ToU16Filter, u16printer);
//     let mut pipeline = PipelineStage::stage(U64ToU32Filter, u32tou16);
//
//     pipeline.down(56u64);
// }

































/*


struct PipelineStage<L, C, N> {
    last: *L,
    current: C,
    next: N
}

impl <L, C> PipelineStage<L, C, NullDown> {
    fn handler(current: C) -> PipelineStage<L, C, NullDown> {
        PipelineStage {
            last: ptr::null(),
            current: current,
            next: NullDown
        }
    }
}

impl <L, C, N> PipelineStage<L, C, N> {
    fn stage(current: C, next: N) -> PipelineStage<L, C, N> {
        PipelineStage {
            last: ptr::null(),
            current: current,
            next: next
        }
    }
}

trait <X, L: PipelineUp<X>> PipelineInit<L> {
    fn init(&mut self, last: *L);
}

impl <L, C, N> PipelineInit<Y> for PipelineStage<L, C, N> {

}

trait PipelineDown<Din> {
    fn down(&self, data: Din);
}

trait PipelineUp<Uin> {
    fn up(&self, data: Uin);
}

impl <Din, Dout, Uin, Uout, C: Filter<Din, Dout, Uin, Uout>, L: PipelineUp<Uout>, N: PipelineDown<Dout>> PipelineDown<Din> for PipelineStage<L, C, N> {
    fn down(&self, data: Din) {
        match unsafe { self.last.to_option() } {
            Some(last) => self.current.down(data, last, &self.next),
            None => self.current.down(data, &AnyUp, &self.next)
        }
    }
}

impl <Din, Dout, Uin, Uout, C: Filter<Din, Dout, Uin, Uout>, L: PipelineUp<Uout>, N: PipelineDown<Dout>> PipelineUp<Uin> for PipelineStage<L, C, N> {
    fn up(&self, data: Uin) {
        match unsafe { self.last.to_option() } {
            Some(last) => self.current.up(data, last, &self.next),
            None => self.current.up(data, &AnyUp, &self.next)
        }
    }
}

struct AnyUp;

impl <X> PipelineUp<X> for AnyUp {
    fn up(&self, data: X) { }
}

struct AnyDown;

impl <X> PipelineDown<X> for AnyDown {
    fn down(&self, data: X) { }
}

struct NullUp;

impl PipelineUp<()> for NullUp {
    fn up(&self, data: ()) { }
}

struct NullDown;

impl PipelineDown<()> for NullDown {
    fn down(&self, data: ()) { }
}

trait Filter<Din, Dout, Uin, Uout> {
    fn down<L: PipelineUp<Uout>, N: PipelineDown<Dout>>(&self, data: Din, last: &L, next: &N);

    fn up<L: PipelineUp<Uout>, N: PipelineDown<Dout>>(&self, data: Uin, last: &L, next: &N);
}

struct U64ToU32Filter;

impl Filter<u64, u32, u32, u64> for U64ToU32Filter {
    fn down<L: PipelineUp<u64>, N: PipelineDown<u32>>(&self, data: u64, last: &L, next: &N) {
        next.down(data as u32);
    }

    fn up<L: PipelineUp<u64>, N: PipelineDown<u32>>(&self, data: u32, last: &L, next: &N) {
        last.up(data as u64);
    }
}

struct U32ToU16Filter;

impl Filter<u32, u16, u16, u32> for U32ToU16Filter {
    fn down<L: PipelineUp<u32>, N: PipelineDown<u16>>(&self, data: u32, last: &L, next: &N) {
        next.down(data as u16);
    }

    fn up<L: PipelineUp<u32>, N: PipelineDown<u16>>(&self, data: u16, last: &L, next: &N) {
        last.up(data as u32);
    }
}

struct PrintU16Filter;

impl Filter<u16, (), (), u32> for PrintU16Filter {
    fn down<L: PipelineUp<u16>, N: PipelineDown<()>>(&self, data: u16, last: &L, next: &N) {
        println!("Value!: {}", data);
    }

    fn up<L: PipelineUp<u16>, N: PipelineDown<()>>(&self, data: (), last: &L, next: &N) {
    }
}


#[main]
fn main() {
    use std::ptr;

    let mut u16printer = PipelineStage::handler(PrintU16Filter);
    let mut u32tou16 = PipelineStage::stage(U32ToU16Filter, u16printer);
    let mut pipeline = PipelineStage::stage(U64ToU32Filter, u32tou16);

    pipeline.down(56u64);
}


*/










/*
struct PipelineStage<L, C, N> {
    last: *L,
    current: C,
    next: N
}

// impl <L, C1, C2, N2> PipelineStage<L, C1, PipelineStage<Self, C2, N2>> {
//     fn new(current: C1, next: PipelineStage<Self, C2, N2>) {
//         let ps = PipelineStage {
//             last: std::ptr::null(),
//             current: current,
//             next: next
//         };
// //        ps.next.last = &ps;
//         ps
//     }
// }
//
// trait PipelineInit<Uout, L: PipelineUp<Uout>> {
//     fn init(&mut self, last: *L);
// }

trait PipelineDown<Din> {
    fn down(&self, data: Din);
}

trait PipelineUp<Uin> {
    fn up(&self, data: Uin);
}

// impl <> PipelineInit<L> for PipelineStage<L, C, N> {
//     fn init(&mut self, last: *L) {
//         self.last = last;
//         let me: *M = &*self;
//         match self.next {
//             Some(ref mut n) => n.init(me),
//             None => {}
//         }
//     }
// }
//
// impl <Din, Uin, Uout, C: Filter<Din, Uin>, L: PipelineUp<Uout>, M: PipelineUp<Uin>, N: PipelineInit<M>> PipelineInit<L> for PipelineStage<L, C, N> {
//     fn init(&mut self, last: *L) {
//         self.last = last;
//         let me: *M = &*self;
//         match self.next {
//             Some(ref mut n) => n.init(me),
//             None => {}
//         }
//     }
// }

impl <Din, Dout, Uin, Uout, C: Filter<Din, Dout, Uin, Uout>, L: PipelineUp<Uout>, N: PipelineDown<Dout>> PipelineDown<Din> for PipelineStage<L, C, N> {
    fn down(&self, data: Din) {
        self.current.down(data, &NullUp, &self.next);
    }
}

impl <Din, Dout, Uin, Uout, C: Filter<Din, Dout, Uin, Uout>, L: PipelineUp<Uout>, N: PipelineDown<Dout>> PipelineUp<Uin> for PipelineStage<L, C, N> {
    fn up(&self, data: Uin) {
        self.current.up(data, &NullUp, &self.next);
    }
}

struct NullUp;

impl <Uin> PipelineUp<Uin> for NullUp {
    fn up(&self, data: Uin) { }
}

struct NullDown;

impl <Din> PipelineDown<Din> for NullDown {
    fn down(&self, data: Din) { }
}


trait Filter<Din, Dout, Uin, Uout> {
    fn down<L: PipelineUp<Uout>, N: PipelineDown<Dout>>(&self, data: Din, last: &L, next: &N);

    fn up<L: PipelineUp<Uout>, N: PipelineDown<Dout>>(&self, data: Uin, last: &L, next: &N);
}



struct U64ToU32Filter;

impl Filter<u64, u32, u32, u64> for U64ToU32Filter {
    fn down<L: PipelineUp<u64>, N: PipelineDown<u32>>(&self, data: u64, last: &L, next: &N) {
        next.down(data as u32);
    }

    fn up<L: PipelineUp<u64>, N: PipelineDown<u32>>(&self, data: u32, last: &L, next: &N) {
        last.up(data as u64);
    }
}

struct U32ToU16Filter;

impl Filter<u32, u16, u16, u32> for U32ToU16Filter {
    fn down<L: PipelineUp<u32>, N: PipelineDown<u16>>(&self, data: u32, last: &L, next: &N) {
        next.down(data as u16);
    }

    fn up<L: PipelineUp<u32>, N: PipelineDown<u16>>(&self, data: u16, last: &L, next: &N) {
        last.up(data as u32);
    }
}

struct PrintU16Filter;

impl <X, Y> Filter<u16, X, Y, u32> for PrintU16Filter {
    fn down<L: PipelineUp<u16>, N: PipelineDown<X>>(&self, data: u16, last: &L, next: &N) {
        println!("Value: {}", data);
    }

    fn up<L: PipelineUp<u16>, N: PipelineDown<X>>(&self, data: Y, last: &L, next: &N) {
    }
}


#[main]
fn main() {
    use std::ptr;

    let x: *u32 = ptr::null();

    let pipeline = PipelineStage {
        last: &NullUp,
        current: U64ToU32Filter,
        next: PipelineStage {
            last: &NullUp,
            current: U32ToU16Filter,
            next: PipelineStage {
                last: &NullUp,
                current: PrintU16Filter,
                next: NullDown
            }
        }
    };

    pipeline.down(56u64);
}
*/





























/*






trait Filter {
    fn down<S: Filter, N: Filter>(&mut self, sender: &mut S, next: &mut N);
    fn up<S: Filter, N: Filter>(&mut self, sender: &mut S, next: &mut N);
}

struct NullFilter;

impl Filter for NullFilter {
    fn down<S: Filter, N: Filter>(&mut self, sender: &mut S, next: &mut N) {}
    fn up<S: Filter, N: Filter>(&mut self, sender: &mut S, next: &mut N) {}
}

struct Holder<L, C, N> {
    last: *mut L,
    current: C,
    next: Option<N>
}

impl <L, C, N> Holder {
    fn run(&mut self) {
        match next {
            Some(ref mut n) => {
                if self.last.is_null() {
                    self.current(self.last, n);
                } else {

                }
            }
            None => {

            }
        }
    }
}





struct Parent<C> {
    child: C,
    value: int
}

struct Child {
    value: int
}

trait Run {
    fn run(&mut self, sender: &mut R) { fail!() }
    fn mid<R: Run>(&mut self, sender: &mut R) { fail!() }
    fn end<R: Run>(&mut self, sender: &mut R) { fail!() }
}

impl <C: Run> Run for Parent<C> {
    fn run(&mut self) {
        println!("Run");
        self.child.mid(self);
    }
    fn end<R: Run>(&mut self, sender: &mut R) {
        println!("End");
    }
}

impl Run for Child {
    fn mid<R: Run>(&mut self, sender: &mut R) {
        println!("Mid");
        sender.end(self);
    }
}

#[main]
fn main() {
    let mut p = Parent {
        child: Child { value: 0 },
        value: 0
    };
    p.run();
}



struct Pipeline<S> {
    steps: S
}

struct PipelineContext;




// trait PipelineFilter<X, Y, S: PipelineSend<Y>, > {
//     fn filter(&mut self, input: X, sender: &mut S, next: N);
// }

*/


/*
trait PipelineStart<X> {
    fn enter(&mut self, data: X);
    fn exit(&mut self, data: X);
}

trait PipelineFilter<X> {
    fn decode(&mut self, data: X);
    fn encode(&mut self, data: X);
}

trait PipelineEnd<X> {
    fn process(&mut self, data: X);
}


struct PipelineStage<S, C, N> {
    sender: *mut S,
    current: C,
    next: Option<N>
}
*/


/*

trait Decoder<X, Y, FC: FilterContext<Y>> {
    fn decode(&mut self, data: X, ctx: &mut FC);
}

trait Encoder<X, Y, FC: FilterContext<Y>> {
    fn encode(&mut self, data: X, ctx: &mut FC);
}

trait FilterContext<X, Y> {
    fn decode(&mut self, data: X);
    fn encode(&mut self, data: Y);
}

struct PipelineStage<S, C, N> {
    sender: *mut S,
    current: C,
    next: Option<N>
}


struct S1;

struct S2;


impl Do for S1 {
    fn do_something<S: Do, N: Do>(&mut self, sender: Option<&mut S>, next: Option<&mut N>) {
        let none: Option<Do> = None;
        match next {
            Some(n) => n.do_something(Some(self), None),
            None => { }
        }
    }
}

impl Do for S2 {
    fn do_something<S: Do, N: Do>(&mut self, sender: Option<&mut S>, next: Option<&mut N>) {
        println!("hi");
    }
}
*/


/*
struct FirstInfo<C, N> {
    current: C,
    next: N
}

struct FilterInfo<S, C, N> {
    sender: *mut S,
    current: C,
    next: N
}

struct LastInfo<S, C> {
    sender: *mut S,
    current: C
}

enum PipelineStage<S, F1, F2, N, E> {
    First(FirstInfo<S, F>),
    Filter(FilterInfo<F, F2, N>),
    Last(LastInfo<S, C>)
}
*/

// trait PipelineDecoder<X, Y, Z, N: PipelineDecoder<Y>, S: PipelineEncoder<Z>> {
//     fn decode(&mut self, data: X, next: &mut N, sender: &mut S);
// }
//
// trait PipelineEncoder<X, Y, Z, N: PipelineEncoder<Y>, S: PipelineDecoder<Z>> {
//     fn encode(&mut self, data: X, next: &mut N, sender: &mut S);
// }





// #[main]
// fn main() {}

/*
struct Mid<S, N> {
    stage: S,
    next: N
}

enum PipelineStage<M, L> {
    PipelineMiddle(M),
    PipelineLast(L)
}

impl <R, S, M: PipelineRecv<R> + PipelineSend<S>, N: PipelineRecv<R>> PipelineRecv<R> for PipelineStage<M, N> {
    fn recv(&mut self, r: R) {
        match self {
            &PipelineMiddle(ref mut m) => {
                m.recv(r);
            }
            &PipelineLast(ref mut l) => {

            }
        }
    }
}






#[main]
fn main() {}


*/












/*
struct Stage1<S> {
    next: Option<S>
}

impl <N: PipelineRecv<u32>> PipelineRecv<u64> for Stage1<N> {
    fn recv(&mut self, r: u64) {
        match self.next {
            Some(ref mut n) => n.recv(r as u32),
            None => {}
        }
    }
}

struct Stage2<S> {
    next: Option<S>
}

impl <N: PipelineRecv<u16>> PipelineRecv<u32> for Stage2<N> {
    fn recv(&mut self, r: u32) {
        match self.next {
            Some(ref mut n) => n.recv(r as u16),
            None => {}
        }
    }
}

struct Stage3;

impl PipelineRecv<u16> for Stage3 {
    fn recv(&mut self, r: u16) {
        println!("Value: {}", r);
    }
}

#[main]
fn main() {
    let mut steps = Stage1 {
        next: Some(
            Stage2 {
                next: Some (
                    Stage3
                )
            }
        )
    };
    steps.recv(24u64);
}
*/














/*

struct Pipeline<S> {
    steps: S
}

struct PipelineContext;

trait PipelineRecv<T> {
    fn recv(&mut self, t: T);
}

trait PipelineSend<T> {
    fn send(&mut self, t: T);
}

struct Stage1<S> {
    next: Option<S>
}

struct Stage2<S> {
    next: Option<S>
}

struct Stage3;

impl <N: PipelineRecv<u32>> PipelineRecv<u64> for Stage1<N> {
    fn recv(&mut self, r: u64) {
        match self.next {
            Some(ref mut n) => n.recv(r as u32),
            None => {}
        }
    }
}

impl <N: PipelineRecv<u32>> PipelineRecv<u64> for Stage1<N> {
    fn recv(&mut self, r: u64) {
        match self.next {
            Some(ref mut n) => n.recv(r as u32),
            None => {}
        }
    }
}

impl <N: PipelineRecv<u16>> PipelineRecv<u32> for Stage2<N> {
    fn recv(&mut self, r: u32) {
        match self.next {
            Some(ref mut n) => n.recv(r as u16),
            None => {}
        }
    }
}

impl PipelineRecv<u16> for Stage3 {
    fn recv(&mut self, r: u16) {
        println!("Value: {}", r);
    }
}

#[main]
fn main() {
    let mut steps = Stage1 {
        next: Some(
            Stage2 {
                next: Some (
                    Stage3
                )
            }
        )
    };
    steps.recv(24u64);
}

*/



/*

struct Pipeline<S> {
    steps: S
}

struct PipelineContext;

trait PipelineStage<R, S> {
    fn recv(&mut self, r: R);
    fn send(&mut self, s: S);
}

struct Stage1<S> {
    next: Option<S>
}

struct Stage2<S> {
    next: Option<S>
}

struct Stage3;

impl <N: PipelineStage<u32, u32>> PipelineStage<u64, u64> for Stage1<N> {
    fn recv(&mut self, r: u64) {
        match self.next {
            Some(ref mut n) => n.recv(r as u32),
            None => {}
        }
    }
    fn send(&mut self, s: u64) {
    }
}

impl <N: PipelineStage<u16, u16>> PipelineStage<u32, u32> for Stage2<N> {
    fn recv(&mut self, r: u32) {
        match self.next {
            Some(ref mut n) => n.recv(r as u16),
            None => {}
        }
    }
    fn send(&mut self, s: u32) {
    }
}

impl PipelineStage<u16, u16> for Stage3 {
    fn recv(&mut self, r: u16) {
        println!("Value: {}", r);
    }
    fn send(&mut self, s: u16) {
    }
}

#[main]
fn main() {
    let mut steps = Stage1 {
        next: Some(
            Stage2 {
                next: Some (
                    Stage3
                )
            }
        )
    };
    steps.recv(24u64);
}

*/



/*


pub enum AsyncProducerResult {
    Done,
    Continue
}

pub trait AsyncProducer {
    fn produce<W: AsyncWriter>(&mut self, writer: &mut W) -> IoResult<AsyncProducerResult>;

    fn and_then<N: AsyncProduer>(self, next: N) -> AsyncChain<Self, N> {
        AsyncChain { first: self, second: next, true }
    }
}

struct AsyncChain<T1, T2> {
    first: T1,
    second: T2,
    on_first: bool
}

impl <T1: AsyncProducer, T2: AsyncProducer> AsyncProducer for AsyncProducerGroup<T1, T2> {
    fn produce<W: AsyncWriter>(&mut self, writer: &mut W) -> IoResult<AsyncProducerResult> {
        if on_first {
            match self.first.produce(
        }

        if !on_first {

        }
    }
}


*/


/*

pub enum AsyncProducerResult {
    Done,
    Continue
}

pub trait AsyncProducer {
    fn produce(&mut self, writer: &mut AsyncWriter) -> IoResult<AsyncProducerResult>;
}








pub struct AsyncProducerGroup {
    priv producers: ~[~AsyncProducer],
    priv idx: uint
}

impl AsyncProducerGroup {
    pub fn new() -> AsyncProducerGroup {
        AsyncProducerGroup { producers: ~[], idx: 0 }
    }

    pub fn add<'a>(&'a mut self, producer: ~AsyncProducer) -> &'a mut Self {
        self.producers.push(producer);
        self
    }
}

impl AsyncProducer for AsyncProducerGroup {
    fn produce(&mut self, writer: &mut AsyncWriter) -> IoResult<AsyncProducerResult> {
        loop {
            if self.idx >= self.producers.len() {
                return Ok(Done);
            }
            let p = self.producers[idx];
            match p.produce(writer) {
                Ok(Done) => self.idx += 1,
                Ok(Continue) => return Ok(Continue),
                Err(e) => return Err(e)
            }
        }
    }
}

struct AsyncOwnedBytesProducer {
    buff: ~[u8],
    idx: uint
}

impl AsyncProducer for AsyncOwnedBytesProducer {
    fn produce(&mut self, writer: &mut AsyncWriter) -> IoResult<AsyncProducerResult> {
        match writer.async_write(buff.slice_from(idx)) {
            Ok(cnt) => {
                self.idx += cnt;
                if self.idx >= self.buff.len() {
                    return Ok(Done);
                } else {
                    return Ok(Continue);
                }
            }
            Err(e) => return Err(e)
        }
    }
}

pub fn produce_bytes(data: ~[u8]) -> ~AsyncProducer {
    ~AsyncOwnedBytesProducer {
        buff: data,
        idx: 0
    }
}
*/
