use crate::ads868x_sim::ADS868XSimulator;
use rust_hdl_core::prelude::*;
use rust_hdl_synth::yosys_validate;
use rust_hdl_widgets::spi_master::SPIConfig;

#[derive(LogicBlock)]
pub struct MuxedADS868XSimulators {
    // Input SPI bus
    pub mosi: Signal<In, Bit>,
    pub mclk: Signal<In, Bit>,
    pub msel: Signal<In, Bit>,
    pub miso: Signal<Out, Bit>,
    pub addr: Signal<In, Bits<3>>,
    pub clock: Signal<In, Clock>,
    adcs: [ADS868XSimulator; 8],
}

impl MuxedADS868XSimulators {
    pub fn new(config: SPIConfig) -> Self {
        Self {
            mosi: Default::default(),
            mclk: Default::default(),
            msel: Default::default(),
            miso: Default::default(),
            addr: Default::default(),
            clock: Default::default(),
            adcs: [
                ADS868XSimulator::new(config),
                ADS868XSimulator::new(config),
                ADS868XSimulator::new(config),
                ADS868XSimulator::new(config),
                ADS868XSimulator::new(config),
                ADS868XSimulator::new(config),
                ADS868XSimulator::new(config),
                ADS868XSimulator::new(config),
            ],
        }
    }
}

impl Logic for MuxedADS868XSimulators {
    #[hdl_gen]
    fn update(&mut self) {
        // Latch prevention
        self.miso.next = true;
        for i in 0_usize..8_usize {
            self.adcs[i].clock.next = self.clock.val();
            self.adcs[i].mosi.next = self.mosi.val();
            self.adcs[i].mclk.next = self.mclk.val();
            self.adcs[i].msel.next = true;
            if self.addr.val().index() == i {
                self.adcs[i].msel.next = self.msel.val();
                self.miso.next = self.adcs[i].miso.val();
            }
        }
    }
}

#[test]
fn test_mux_is_synthesizable() {
    let mut uut = MuxedADS868XSimulators::new(ADS868XSimulator::spi_hw());
    uut.mclk.connect();
    uut.mosi.connect();
    uut.msel.connect();
    uut.addr.connect();
    uut.clock.connect();
    uut.connect_all();
    println!("{}", generate_verilog(&uut));
    yosys_validate("mux_8689", &generate_verilog(&uut)).unwrap();
}
