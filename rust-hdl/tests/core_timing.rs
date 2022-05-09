use rust_hdl::core::check_logic_loops::check_logic_loops;
use rust_hdl::core::check_timing::check_timing;
use rust_hdl::core::prelude::*;
use rust_hdl::widgets::prelude::*;

#[test]
fn test_check_timing() {
    #[derive(LogicInterface, Default)]
    #[join = "ReadInterface"]
    struct WriteInterface {
        pub enable: Signal<In, Bit>,
        pub write: Signal<In, Bit>,
        pub full: Signal<Out, Bit>,
    }

    #[derive(LogicInterface, Default)]
    #[join = "WriteInterface"]
    struct ReadInterface {
        pub enable: Signal<Out, Bit>,
        pub write: Signal<Out, Bit>,
        pub full: Signal<In, Bit>,
    }

    #[derive(LogicBlock, Default)]
    struct Widget {
        pub write: WriteInterface,
        pub clock: Signal<In, Clock>,
        pub reset: Signal<In, Reset>,
        lm: DFF<Bit>,
    }

    impl Logic for Widget {
        #[hdl_gen]
        fn update(&mut self) {
            clock_reset!(self, clock, reset, lm);
            self.lm.d.next = self.write.enable.val();
            self.write.full.next = self.lm.q.val();
        }
    }

    #[derive(LogicBlock, Default)]
    struct Copper {
        pub clock: Signal<In, Clock>,
        pub reset: Signal<In, Reset>,
        pub data: WriteInterface,
        pub iface: ReadInterface,
        pub red: Signal<Out, Bit>,
        b1: Basic,
        b2: Basic,
        cc: DFF<Bits<14>>,
        loc1: Signal<Local, Bit>,
        loc2: Signal<Local, Bit>,
        loc3: Signal<Local, Bit>,
        widget: Widget,
    }

    impl Logic for Copper {
        #[hdl_gen]
        fn update(&mut self) {
            clock_reset!(self, clock, reset, b1, b2, widget);
            dff_setup!(self, clock, reset, cc);
            self.b1.enable.next = self.data.enable.val();
            self.b2.enable.next = self.data.enable.val();
            self.cc.d.next = self.cc.q.val();
            self.loc1.next = self.data.enable.val();
            self.loc2.next = self.loc1.val() | self.data.enable.val();
            self.loc3.next =
                self.loc1.val() & self.loc2.val() & self.data.enable.val() & self.data.write.val();
            self.data.full.next = self.loc1.val();
            self.b1.data.enable.next = true;
            self.b1.data.write.next = false;
            self.b2.data.enable.next = true;
            self.b2.data.write.next = false;
            ReadInterface::join(&mut self.iface, &mut self.widget.write);
            self.red.next = self.b1.red.val() | self.b2.red.val();
        }
    }

    #[derive(LogicState, Copy, Clone, Debug, PartialEq)]
    enum State {
        Start,
        Run,
        Stop,
    }

    #[derive(LogicBlock, Default)]
    struct Basic {
        pub clock: Signal<In, Clock>,
        pub reset: Signal<In, Reset>,
        pub data: WriteInterface,
        pub enable: Signal<In, Bit>,
        pub red: Signal<Out, Bit>,
        pub amber: Signal<Out, Bit>,
        pub green: Signal<Out, Bit>,
        counter: DFF<Bits<14>>,
        state: DFF<State>,
    }

    impl Logic for Basic {
        #[hdl_gen]
        fn update(&mut self) {
            dff_setup!(self, clock, reset, counter);
            if self.enable.val() {
                self.counter.d.next = self.counter.q.val() + 1_usize;
            }
            self.red.next = false;
            self.green.next = false;
            self.amber.next = false;
            match self.state.q.val() {
                State::Start => {
                    self.red.next = true;
                    self.state.d.next = State::Run;
                }
                State::Run => {
                    self.green.next = true;
                    self.state.d.next = State::Stop;
                }
                State::Stop => {
                    self.amber.next = true;
                    self.state.d.next = State::Start;
                }
                _ => {
                    self.state.d.next = State::Start;
                }
            }
        }
    }

    let mut uut = Copper::default();
    //    check_logic_loops(&uut).unwrap();
    check_timing(&uut);
    uut.clock.connect();
    uut.reset.connect();
    uut.data.link_connect_dest();
    uut.iface.link_connect_dest();
    uut.connect_all();
    check_connected(&uut);
}
