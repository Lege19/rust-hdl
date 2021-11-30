use rust_hdl::bsp::ok_core::prelude::*;
use rust_hdl::core::prelude::*;

mod test_common;

use rust_hdl::bsp::ok_xem7010::XEM7010;
use test_common::blinky::OpalKellyBlinky;

#[cfg(feature = "frontpanel")]
#[test]
fn test_opalkelly_xem_7010_synth_blinky() {
    let mut uut = OpalKellyBlinky::new::<XEM7010>();
    uut.hi.link_connect_dest();
    uut.connect_all();
    check_connected(&uut);
    XEM7010::synth(uut, target_path!("xem_7010/blinky"));
}