#![no_main]
#![no_std]

use embassy_executor::Spawner;
use embassy_net::IpListenEndpoint;
use embassy_rp::gpio::{Level, Output};
use embassy_time::Timer;
use core::marker::Sized;
use defmt::{info, error};
use core::panic::PanicInfo;
use defmt_rtt as _;

mod irqs;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let peripherals = embassy_rp::init(Default::default());
    let mut blue = Output::new(peripherals.PIN_15, Level::Low); //blue
    let _white = Output::new(peripherals.PIN_14, Level::High); //yellow
    let mut red = Output::new(peripherals.PIN_13, Level::Low); // red
    let freq = 100;
    loop {
        red.set_high();            
        Timer::after_millis(freq).await;
        red.set_low();            
        Timer::after_millis(freq).await;
        red.set_high();            
        Timer::after_millis(freq).await;
        red.set_low();
        blue.set_high();
        Timer::after_millis(freq).await;
        blue.set_low();
        Timer::after_millis(freq).await;
        blue.set_high();
        Timer::after_millis(freq).await;
        blue.set_low();
        Timer::after_millis(freq).await;
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    error!("Panic occurred: {:?}", info);
    loop {}
}