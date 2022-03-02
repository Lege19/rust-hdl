use crate::core::prelude::*;
use crate::widgets::prelude::*;

#[derive(Copy, Clone, PartialEq, Debug, LogicState)]
enum State {
    Idle,
    Start,
    Reading,
    Waiting,
    Flag,
    Ack,
    AckHold,
    WaitEndTransaction,
    CheckStop,
    Writing,
    WaitSCLHigh,
    WaitSCLLow,
    WaitSCLHighAck,
    CollectAck,
}

#[derive(LogicBlock, Default)]
pub struct I2CTarget {
    pub sda: Signal<InOut, Bit>,
    pub scl: Signal<InOut, Bit>,
    pub clock: Signal<In, Clock>,
    pub from_bus: Signal<Out, Bits<8>>,
    pub bus_write: Signal<Out, Bit>,
    pub active: Signal<In, Bit>,
    pub stop: Signal<Out, Bit>,
    pub to_bus: Signal<In, Bits<8>>,
    pub write_enable: Signal<In, Bit>,
    pub ack: Signal<Out, Bit>,
    pub nack: Signal<Out, Bit>,
    pub write_ok: Signal<Out, Bit>,
    state: DFF<State>,
    sda_driver: OpenDrainBuffer,
    scl_driver: OpenDrainBuffer,
    scl_is_high: Signal<Local, Bit>,
    sda_is_high: Signal<Local, Bit>,
    sda_flop: DFF<Bit>,
    clear_sda: Signal<Local, Bit>,
    set_sda: Signal<Local, Bit>,
    read_bit: DFF<Bit>,
    count: DFF<Bits<4>>,
    accum: DFF<Bits<8>>,
}

impl Logic for I2CTarget {
    #[hdl_gen]
    fn update(&mut self) {
        Signal::<InOut, Bit>::link(&mut self.sda, &mut self.sda_driver.bus);
        Signal::<InOut, Bit>::link(&mut self.scl, &mut self.scl_driver.bus);
        // Clock the internal structures
        self.state.clk.next = self.clock.val();
        self.sda_flop.clk.next = self.clock.val();
        self.read_bit.clk.next = self.clock.val();
        self.count.clk.next = self.clock.val();
        self.accum.clk.next = self.clock.val();
        // Latch prevention
        self.state.d.next = self.state.q.val();
        self.sda_flop.d.next = self.sda_flop.q.val();
        self.read_bit.d.next = self.read_bit.q.val();
        self.count.d.next = self.count.q.val();
        self.accum.d.next = self.accum.q.val();
        self.scl_driver.enable.next = false;
        self.sda_driver.enable.next = self.sda_flop.q.val();
        self.sda_is_high.next = self.sda_driver.read_data.val();
        self.scl_is_high.next = self.scl_driver.read_data.val();
        self.set_sda.next = false;
        self.clear_sda.next = false;
        self.from_bus.next = self.accum.q.val();
        self.bus_write.next = false;
        self.stop.next = false;
        self.ack.next = false;
        self.nack.next = false;
        self.write_ok.next = false;
        // For now, can only read bits
        match self.state.q.val() {
            State::Idle => {
                self.set_sda.next = true;
                self.count.d.next = 0_usize.into();
                if !self.sda_is_high.val() & self.scl_is_high.val() {
                    self.state.d.next = State::Start;
                }
            }
            State::Start => {
                if self.sda_is_high.val() {
                    self.state.d.next = State::Idle;
                } else if !self.sda_is_high.val() & !self.scl_is_high.val() {
                    self.state.d.next = State::Reading;
                }
            }
            State::Writing => {
                if self.accum.q.val().get_bit(7_usize) {
                    self.set_sda.next = true;
                } else {
                    self.clear_sda.next = true;
                }
                self.count.d.next = self.count.q.val() + 1_usize;
                self.accum.d.next = self.accum.q.val() << 1_usize;
                self.state.d.next = State::WaitSCLHigh;
                if self.count.q.val() == 8_usize {
                    self.state.d.next = State::WaitSCLHighAck;
                }
            }
            State::WaitSCLHighAck => {
                if self.scl_is_high.val() {
                    self.set_sda.next = true;
                    self.state.d.next = State::CollectAck;
                }
            }
            State::CollectAck => {
                if !self.scl_is_high.val() {
                    if self.sda_is_high.val() {
                        self.nack.next = true;
                    } else {
                        self.ack.next = true;
                    }
                    self.state.d.next = State::Reading;
                }
            }
            State::WaitSCLHigh => {
                if self.scl_is_high.val() {
                    self.state.d.next = State::WaitSCLLow;
                }
            }
            State::WaitSCLLow => {
                if !self.scl_is_high.val() {
                    self.state.d.next = State::Writing;
                }
            }
            State::Reading => {
                self.write_ok.next = true;
                if self.write_enable.val() {
                    self.accum.d.next = self.to_bus.val();
                    self.count.d.next = 0_usize.into();
                    self.state.d.next = State::Writing;
                }
                if self.scl_is_high.val() {
                    self.read_bit.d.next = self.sda_is_high.val();
                    self.state.d.next = State::Waiting;
                }
            }
            State::Waiting => {
                if !self.scl_is_high.val() {
                    self.accum.d.next = (self.accum.q.val() << 1_usize)
                        | bit_cast::<8, 1>(self.read_bit.q.val().into());
                    self.count.d.next = self.count.q.val() + 1_usize;
                    if self.count.q.val() == 7_usize {
                        self.state.d.next = State::Flag;
                    } else {
                        self.state.d.next = State::Reading;
                    }
                }
                if !self.read_bit.q.val() & self.sda_is_high.val() & self.scl_is_high.val() {
                    self.stop.next = true;
                    self.state.d.next = State::Idle;
                }
                if self.read_bit.q.val() & !self.sda_is_high.val() & self.scl_is_high.val() {
                    self.stop.next = true;
                    self.state.d.next = State::Idle;
                }
            }
            State::Flag => {
                self.bus_write.next = true;
                self.accum.d.next = 0_u8.into();
                self.count.d.next = 0_u8.into();
                self.state.d.next = State::Ack;
            }
            State::Ack => {
                self.clear_sda.next = self.active.val();
                if !self.active.val() {
                    self.state.d.next = State::WaitEndTransaction;
                }
                if self.scl_is_high.val() {
                    self.state.d.next = State::AckHold;
                }
            }
            State::AckHold => {
                if !self.scl_is_high.val() {
                    self.set_sda.next = true;
                    self.state.d.next = State::Reading;
                }
            }
            State::WaitEndTransaction => {
                // A STOP condition is signalled by
                // CLK -> H, SDA -> L
                // CLK -> H, SDA -> H
                if self.scl_is_high.val() & !self.sda_is_high.val() {
                    self.state.d.next = State::CheckStop;
                }
            }
            State::CheckStop => {
                if self.scl_is_high.val() & self.sda_is_high.val() {
                    self.state.d.next = State::Idle;
                    self.stop.next = true;
                }
            }
        }
        if self.set_sda.val() {
            self.sda_flop.d.next = false;
        }
        if self.clear_sda.val() {
            self.sda_flop.d.next = true;
        }
    }
}