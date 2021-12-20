use rust_hdl::core::prelude::*;
use rust_hdl::widgets::prelude::*;

#[derive(LogicBlock, Default)]
struct MOSIPortTest {
    bus: SoCBusController<16, 2>,
    bridge: Bridge<16, 2, 2>,
    port_a: MOSIPort<16>,
    port_b: MOSIPort<16>,
    clock: Signal<In, Clock>,
}

impl Logic for MOSIPortTest {
    #[hdl_gen]
    fn update(&mut self) {
        self.bus.clock.next = self.clock.val();
        self.bus.join(&mut self.bridge.upstream);
        self.bridge.nodes[0].join(&mut self.port_a.bus);
        self.bridge.nodes[1].join(&mut self.port_b.bus);
    }
}

#[test]
fn test_port_test_synthesizes() {
    let mut uut = MOSIPortTest::default();
    uut.bus.clock.connect();
    uut.bus.from_master.connect();
    uut.bus.address.connect();
    uut.bus.strobe.connect();
    uut.port_a.ready.connect();
    uut.port_b.ready.connect();
    uut.clock.connect();
    uut.connect_all();
    let vlog = generate_verilog(&uut);
    yosys_validate("test_port", &vlog).unwrap();
}
#[test]
fn test_port_test_works() {
    let mut uut = MOSIPortTest::default();
    uut.bus.clock.connect();
    uut.bus.from_master.connect();
    uut.bus.address.connect();
    uut.bus.strobe.connect();
    uut.port_a.ready.connect();
    uut.port_b.ready.connect();
    uut.clock.connect();
    uut.connect_all();
    let mut sim = Simulation::new();
    sim.add_clock(5, |x: &mut Box<MOSIPortTest>| x.clock.next = !x.clock.val());
    sim.add_testbench(move |mut sim: Sim<MOSIPortTest>| {
        let mut x = sim.init()?;
        wait_clock_true!(sim, clock, x);
        x.bus.address.next = 1_usize.into();
        x.bus.from_master.next = 0xDEAD_u16.into();
        wait_clock_cycle!(sim, clock, x);
        x = sim.watch(|x| x.bus.ready.val(), x)?;
        x.bus.strobe.next = true;
        wait_clock_cycle!(sim, clock, x);
        x.bus.strobe.next = false;
        x.bus.address.next = 0_usize.into();
        x.bus.from_master.next = 0xBEEF_u16.into();
        wait_clock_cycle!(sim, clock, x);
        x = sim.watch(|x| x.bus.ready.val(), x)?;
        x.bus.strobe.next = true;
        wait_clock_cycle!(sim, clock, x);
        x.bus.strobe.next = false;
        wait_clock_cycle!(sim, clock, x);
        sim.done(x)
    });
    sim.add_testbench(move |mut sim: Sim<MOSIPortTest>| {
        let mut x = sim.init()?;
        x.port_a.ready.next = true;
        x = sim.watch(|x| x.port_a.strobe_out.val(), x)?;
        sim_assert!(sim, x.port_a.port_out.val() == 0xBEEF_u16, x);
        sim.done(x)
    });
    sim.add_testbench(move |mut sim: Sim<MOSIPortTest>| {
        let mut x = sim.init()?;
        x.port_b.ready.next = true;
        x = sim.watch(|x| x.port_b.strobe_out.val(), x)?;
        sim_assert!(sim, x.port_b.port_out.val() == 0xDEAD_u16, x);
        sim.done(x)
    });
    sim.run_traced(
        Box::new(uut),
        1000,
        std::fs::File::create(vcd_path!("mosi_port.vcd")).unwrap(),
    )
    .unwrap();
}

#[test]
fn test_port_pipeline() {
    let mut uut = MOSIPortTest::default();
    uut.bus.clock.connect();
    uut.bus.from_master.connect();
    uut.bus.address.connect();
    uut.bus.strobe.connect();
    uut.port_a.ready.connect();
    uut.port_b.ready.connect();
    uut.clock.connect();
    uut.connect_all();
    let mut sim = Simulation::new();
    sim.add_clock(5, |x: &mut Box<MOSIPortTest>| x.clock.next = !x.clock.val());
    sim.add_testbench(move |mut sim: Sim<MOSIPortTest>| {
        let mut x = sim.init()?;
        wait_clock_true!(sim, clock, x);
        x.bus.address.next = 1_usize.into();
        x = sim.watch(|x| x.bus.ready.val(), x)?;
        for val in [0xDEAD_u16, 0xBEEF, 0xBABE, 0xCAFE] {
            x.bus.from_master.next = val.into();
            x.bus.strobe.next = true;
            wait_clock_cycle!(sim, clock, x);
        }
        x.bus.strobe.next = false;
        wait_clock_cycles!(sim, clock, x, 10);
        sim.done(x)
    });
    sim.add_testbench(move |mut sim: Sim<MOSIPortTest>| {
        let mut x = sim.init()?;
        x.port_b.ready.next = true;
        for val in [0xDEAD_u16, 0xBEEF, 0xBABE, 0xCAFE] {
            x = sim.watch(|x| x.port_b.strobe_out.val(), x)?;
            sim_assert!(sim, x.port_b.port_out.val() == val, x);
            wait_clock_cycle!(sim, clock, x);
        }
        sim.done(x)
    });
    sim.run_traced(
        Box::new(uut),
        1000,
        std::fs::File::create(vcd_path!("mosi_port_pipeline.vcd")).unwrap(),
    )
    .unwrap();
}

#[derive(LogicBlock, Default)]
struct MOSIWidePortTest {
    bus: SoCBusController<16, 2>,
    bridge: Bridge<16, 2, 2>,
    port_a: MOSIWidePort<64, 16>,
    port_b: MOSIWidePort<64, 16>,
    clock: Signal<In, Clock>,
}

impl Logic for MOSIWidePortTest {
    #[hdl_gen]
    fn update(&mut self) {
        self.bus.clock.next = self.clock.val();
        self.bus.join(&mut self.bridge.upstream);
        self.bridge.nodes[0].join(&mut self.port_a.bus);
        self.bridge.nodes[1].join(&mut self.port_b.bus);
    }
}

#[test]
fn test_wport_test_synthesizes() {
    let mut uut = MOSIWidePortTest::default();
    uut.clock.connect();
    uut.bus.address.connect();
    uut.bus.from_master.connect();
    uut.bus.strobe.connect();
    uut.connect_all();
    let vlog = generate_verilog(&uut);
    yosys_validate("wide_test_port", &vlog).unwrap();
}

#[test]
fn test_wide_port_test_works() {
    let mut uut = MOSIWidePortTest::default();
    uut.clock.connect();
    uut.bus.address.connect();
    uut.bus.from_master.connect();
    uut.bus.strobe.connect();
    uut.connect_all();
    let mut sim = Simulation::new();
    sim.add_clock(5, |x: &mut Box<MOSIWidePortTest>| {
        x.clock.next = !x.clock.val()
    });
    sim.add_testbench(move |mut sim: Sim<MOSIWidePortTest>| {
        let mut x = sim.init()?;
        wait_clock_true!(sim, clock, x);
        x.bus.address.next = 0_usize.into();
        wait_clock_cycle!(sim, clock, x);
        x = sim.watch(|x| x.bus.ready.val(), x)?;
        for val in [0xDEAD_u16, 0xBEEF_u16, 0xCAFE_u16, 0x1234_u16] {
            x.bus.strobe.next = true;
            x.bus.from_master.next = val.into();
            wait_clock_cycle!(sim, clock, x);
        }
        x.bus.strobe.next = false;
        wait_clock_cycle!(sim, clock, x);
        x.bus.address.next = 1_usize.into();
        wait_clock_cycle!(sim, clock, x);
        x = sim.watch(|x| x.bus.ready.val(), x)?;
        for val in [
            0xBABE_u16, 0x5EA1_u16, 0xFACE_u16, 0xABCD_u16, 0xBABA_u16, 0xCECE_u16, 0x4321_u16,
            0x89AB_u16,
        ] {
            x.bus.strobe.next = true;
            x.bus.from_master.next = val.into();
            wait_clock_cycle!(sim, clock, x);
        }
        x.bus.strobe.next = false;
        wait_clock_cycles!(sim, clock, x, 10);
        sim.done(x)
    });
    sim.add_testbench(move |mut sim: Sim<MOSIWidePortTest>| {
        let mut x = sim.init()?;
        x = sim.watch(|x| x.port_a.strobe_out.val(), x)?;
        sim_assert!(sim, x.port_a.port_out.val() == 0xDEADBEEFCAFE1234_u64, x);
        sim.done(x)
    });
    sim.add_testbench(move |mut sim: Sim<MOSIWidePortTest>| {
        let mut x = sim.init()?;
        x = sim.watch(|x| x.port_b.strobe_out.val(), x)?;
        sim_assert!(sim, x.port_b.port_out.val() == 0xBABE5EA1FACEABCD_u64, x);
        wait_clock_cycle!(sim, clock, x);
        x = sim.watch(|x| x.port_b.strobe_out.val(), x)?;
        sim_assert!(sim, x.port_b.port_out.val() == 0xBABACECE432189AB_u64, x);
        wait_clock_cycle!(sim, clock, x);
        sim.done(x)
    });
    sim.run_traced(
        Box::new(uut),
        1000,
        std::fs::File::create(vcd_path!("mosi_wide_port.vcd")).unwrap(),
    )
    .unwrap();
}

#[derive(LogicBlock, Default)]
struct MOSIPortFIFOTest {
    bus: SoCPortController<16>,
    port_a: MOSIPort<16>,
    fifo: SynchronousFIFO<Bits<16>, 4, 5, 1>,
    clock: Signal<In, Clock>,
}

impl Logic for MOSIPortFIFOTest {
    #[hdl_gen]
    fn update(&mut self) {
        self.bus.clock.next = self.clock.val();
        self.bus.join(&mut self.port_a.bus);
        self.fifo.clock.next = self.clock.val();
        self.fifo.data_in.next = self.port_a.port_out.val();
        self.fifo.write.next = self.port_a.strobe_out.val();
        self.port_a.ready.next = !self.fifo.full.val();
    }
}

#[test]
fn test_mosi_port_fifo_synthesizes() {
    let mut uut = MOSIPortFIFOTest::default();
    uut.bus.strobe.connect();
    uut.bus.from_master.connect();
    uut.bus.select.connect();
    uut.fifo.read.connect();
    uut.clock.connect();
    uut.connect_all();
    let vlog = generate_verilog(&uut);
    yosys_validate("mosi_port_fifo", &vlog).unwrap();
}

#[test]
fn test_mosi_port_fifo_works() {
    let mut uut = MOSIPortFIFOTest::default();
    uut.bus.select.connect();
    uut.bus.strobe.connect();
    uut.bus.from_master.connect();
    uut.fifo.read.connect();
    uut.clock.connect();
    uut.connect_all();
    let mut sim = Simulation::new();
    sim.add_clock(5, |x: &mut Box<MOSIPortFIFOTest>| {
        x.clock.next = !x.clock.val()
    });
    sim.add_testbench(move |mut sim: Sim<MOSIPortFIFOTest>| {
        let mut x = sim.init()?;
        wait_clock_true!(sim, clock, x);
        x.bus.select.next = true;
        wait_clock_cycle!(sim, clock, x);
        for val in [0xDEAD_u16, 0xBEEF_u16, 0xBABE_u16, 0xCAFE_u16] {
            x = sim.watch(|x| x.bus.ready.val(), x)?;
            x.bus.from_master.next = val.into();
            x.bus.strobe.next = true;
            wait_clock_cycle!(sim, clock, x);
            x.bus.strobe.next = false;
        }
        wait_clock_cycles!(sim, clock, x, 100);
        sim.done(x)
    });
    sim.run_traced(
        Box::new(uut),
        1000,
        std::fs::File::create(vcd_path!("mosi_fifo.vcd")).unwrap(),
    )
    .unwrap();
}
