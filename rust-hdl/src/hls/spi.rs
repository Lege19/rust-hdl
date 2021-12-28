use crate::core::prelude::*;
use crate::hls::bridge::Bridge;
use crate::hls::bus::SoCBusResponder;
use crate::hls::miso_wide_port::MISOWidePort;
use crate::hls::mosi_port::MOSIPort;
use crate::hls::mosi_wide_port::MOSIWidePort;
use crate::widgets::prelude::*;
use crate::widgets::spi_master::{SPIMaster, SPIWires};

// HLS ports
// 0 - data in
// 1 - data out
// 2 - width in
// 3 - start/type
#[derive(LogicBlock)]
pub struct HLSSPIMaster<const D: usize, const A: usize, const W: usize> {
    pub spi: SPIWires,
    pub upstream: SoCBusResponder<D, A>,
    bridge: Bridge<D, A, 4>,
    data_outbound: MOSIWidePort<W, D>,
    data_inbound: MISOWidePort<W, D>,
    num_bits: MOSIPort<D>,
    start: MOSIPort<D>,
    core: SPIMaster<W>,
}

impl<const D: usize, const A: usize, const W: usize> Logic for HLSSPIMaster<D, A, W> {
    #[hdl_gen]
    fn update(&mut self) {
        self.core.clock.next = self.bridge.clock_out.val();
        self.core.data_outbound.next = self.data_outbound.port_out.val();
        self.data_inbound.port_in.next = self.core.data_inbound.val();
        self.data_inbound.strobe_in.next = self.core.transfer_done.val();
        self.core.bits_outbound.next = bit_cast::<16, D>(self.num_bits.port_out.val());
        self.core.continued_transaction.next = self.start.port_out.val().get_bit(0);
        self.core.start_send.next = self.start.strobe_out.val();
        self.upstream.link(&mut self.bridge.upstream);
        self.bridge.nodes[0].join(&mut self.data_outbound.bus);
        self.bridge.nodes[1].join(&mut self.data_inbound.bus);
        self.bridge.nodes[2].join(&mut self.num_bits.bus);
        self.bridge.nodes[3].join(&mut self.start.bus);
        self.spi.link(&mut self.core.wires);
        self.num_bits.ready.next = true;
        self.start.ready.next = !self.core.busy.val();
    }
}

impl<const D: usize, const A: usize, const W: usize> HLSSPIMaster<D, A, W> {
    pub fn new(config: SPIConfig) -> Self {
        Self {
            spi: Default::default(),
            upstream: Default::default(),
            bridge: Default::default(),
            data_outbound: Default::default(),
            data_inbound: Default::default(),
            num_bits: Default::default(),
            start: Default::default(),
            core: SPIMaster::new(config),
        }
    }
}

#[test]
fn test_hls_spi_master_is_synthesizable() {
    let spi_config = SPIConfig {
        clock_speed: 48_000_000,
        cs_off: true,
        mosi_off: true,
        speed_hz: 1_000_000,
        cpha: true,
        cpol: true,
    };
    let mut uut = HLSSPIMaster::<16, 8, 64>::new(spi_config);
    uut.upstream.link_connect_dest();
    uut.spi.link_connect_dest();
    uut.connect_all();
    let vlog = generate_verilog(&uut);
    yosys_validate("hls_spi", &vlog).unwrap();
}