// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

pub trait Filter<Din, Dout, Uin, Uout> {
    fn down<U: PipelineUp<Uout>, D: PipelineDown<Dout>>(&self, data: Din, up: &U, down: &D);

    fn up<U: PipelineUp<Uout>, D: PipelineDown<Dout>>(&self, data: Uin, up: &U, down: &D);
}

pub trait PipelineDown<Din> {
    fn down(&self, data: Din);
}

impl <Din> PipelineDown<Din> for ~PipelineDown<Din> {
    fn down(&self, data: Din) {
        self.down(data);
    }
}

pub trait PipelineUp<Uin> {
    fn up(&self, data: Uin);
}

impl <Uin> PipelineUp<Uin> for ~PipelineUp<Uin> {
    fn up(&self, data: Uin) {
        self.up(data);
    }
}

struct PipelineStage<U, F, D> {
    up: U,
    filter: F,
    down: D
}

impl <
        Din, Dout, Uin, Uout,
        F: Filter<Din, Dout, Uin, Uout>,
        U: PipelineUp<Uout>,
        D: PipelineDown<Dout>
     >
        PipelineDown<Din>
        for PipelineStage<U, F, D> {
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
        for PipelineStageRef<PipelineStage<U, F, D>> {
    fn up(&self, data: Uin) {
        let s = unsafe { self.stage.to_option().unwrap() };
        s.filter.up(data, &s.up, &s.down);
    }
}

struct AnyUp<Uin>;

impl <Uin> PipelineUp<Uin> for AnyUp<Uin> {
    fn up(&self, _: Uin) {}
}

struct AnyDown<Din>;

impl <Din> PipelineDown<Din> for AnyDown<Din> {
    fn down(&self, _: Din) {}
}

pub struct PipelineBuilder<F, N> {
    priv filter: F,
    priv next: N
}

struct PipelineTerm;

impl <Din, Dout, Uin, Uout, F: Filter<Din, Dout, Uin, Uout>> PipelineBuilder<F, PipelineTerm> {
    pub fn new(filter: F) -> PipelineBuilder<F, PipelineTerm> {
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
    pub fn filter(self, filter: F1) -> PipelineBuilder<F1, PipelineBuilder<F2, X>> {
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
    pub fn build(self) -> ~PipelineDown<Din> {
        let mut ps = ~self.build_stage();

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
        BuildStage<Din, PipelineStage<~PipelineUp<Uout>, F, D>>
        for PipelineBuilder<F, N> {
    fn build_stage(self) -> PipelineStage<~PipelineUp<Uout>, F, D> {
        let PipelineBuilder {filter, next} = self;

        let any_up: ~AnyUp<Uout> = ~AnyUp;
        let temp_up = any_up as ~PipelineUp<Uout>;

        let ps = PipelineStage {
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
        for PipelineStage<~PipelineUp<Uout>, F, D> {
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
