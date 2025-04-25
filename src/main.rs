#![no_main]
#![no_std]

use embassy_executor::Spawner;
use core::marker::Sized;
use defmt::{info, error};
use core::panic::PanicInfo;
use defmt_rtt as _;

mod irqs;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("Hello");
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    error!("Panic occurred: {:?}", info);
    loop {}
}