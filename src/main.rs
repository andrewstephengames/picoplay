#![no_main]
#![no_std]

use embassy_executor::Spawner;
use embassy_net::IpListenEndpoint;
use embassy_rp::{gpio::{Level, Output}, pwm::{Pwm, SetDutyCycle}};
use embassy_time::Timer;
use embassy_rp::pwm::Config as PwmConfig; 
use core::marker::Sized;
use defmt::{info, error};
use core::panic::PanicInfo;
use defmt_rtt as _;

mod irqs;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let peripherals = embassy_rp::init(Default::default());
    // let _on_light = Output::new(peripherals.PIN_16, Level::High);
    
    let mut config: PwmConfig = Default::default();
    
    config.top = 0x9088;

    config.compare_a = config.top / 100;
    
    let _on_light = Pwm::new_output_a(
        peripherals.PWM_SLICE0,
        peripherals.PIN_16,
        config.clone()
    );
    
    info!("PicoPlay has booted.");
    
    loop {
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    error!("Panic occurred: {:?}", info);
    loop {}
}