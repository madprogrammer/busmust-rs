extern crate adept2;
extern crate clap;

use clap::{App, SubCommand};
use adept2::dmgr;
use adept2::depp::Depp;
use adept2::djtag::Djtag;

fn main() {
    for d in dmgr::enum_devices().unwrap() {
        println!("{} ({})", d.name(), d.conn());
        //        println!("alias:        {}", d.alias().unwrap());
        println!("user name:    {}", d.user_name().unwrap());
        println!("product name: {}", d.product_name().unwrap());
        println!("serial num:   {}", d.serial_number().unwrap());
        let pdid = d.product_id().unwrap();
        println!("product id:   {}.{}", pdid.product(), pdid.variant());
        println!("device caps:  {:?}", d.device_caps().unwrap());
        println!("open count:   {}", d.open_count().unwrap());

        println!("\nDepp");
        let depp = Depp::new(&d).unwrap();
        println!("port count:   {}", depp.port_count().unwrap());
        for p in 0..depp.port_count().unwrap() {
            let prop = depp.port_properties(p as _).unwrap();
            println!("port properties [{}]: {:x}", p, prop);
        }

        println!("\nDjtag");
        let jtag = Djtag::new(&d).unwrap();
        println!("port count:   {}", jtag.port_count().unwrap());
        for p in 0..depp.port_count().unwrap() {
            let prop = jtag.port_properties(p as _).unwrap();
            println!("port properties [{}]: {:?}", p, prop);
            if let Ok(batch) = jtag.batch_properties(p as _) {
                println!("batch properties [{}]: {:?}", p, batch);
            }
        }
    }
}
