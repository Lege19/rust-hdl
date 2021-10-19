use rust_hdl_bsp_ok_xem7010::XEM7010;
use rust_hdl_core::prelude::*;
use rust_hdl_ok_core::prelude::*;
use rust_hdl_ok_frontpanel_sys::OkError;
use rust_hdl_test_core::target_path;
use rust_hdl_test_ok_common::pipe::{test_opalkelly_pipe_fifo_runtime, OpalKellyPipeFIFOTest};

#[test]
fn test_opalkelly_xem_7010_synth_pipe_fifo() {
    let mut uut = OpalKellyPipeFIFOTest::new::<XEM7010>();
    uut.hi.sig_inout.connect();
    uut.hi.sig_in.connect();
    uut.hi.sig_out.connect();
    uut.hi.sig_aa.connect();
    uut.connect_all();
    XEM7010::synth(uut, target_path!("xem_7010/pipe_fifo"));
    test_opalkelly_pipe_fifo_runtime(target_path!("xem_7010/pipe_fifo/top.bit")).unwrap();
}
