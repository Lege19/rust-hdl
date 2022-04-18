use crate::core::prelude::*;
use crate::widgets::dff::DFF;

#[derive(Clone, Debug, LogicBlock, Default)]
pub struct AutoReset {
    pub reset: Signal<Out, ResetN>,
    pub clock: Signal<In, Clock>,
    dff: DFF<Bits<3>>,
}

impl Logic for AutoReset {
    #[hdl_gen]
    fn update(&mut self) {
        self.dff.clock.next = self.clock.val();
        self.dff.reset.next = true.into();
        self.dff.d.next = self.dff.q.val();
        self.reset.next = true.into();
        if !self.dff.q.val().all() {
            self.dff.d.next = self.dff.q.val() + 1_usize;
            self.reset.next = false.into();
        }
    }
}

#[test]
fn test_synch_reset_synchronizes() {
    let mut uut = AutoReset::default();
    uut.clock.connect();
    uut.connect_all();
    yosys_validate("sync_reset", &generate_verilog(&uut)).unwrap();
}