#![no_main]
#![no_std]

use embassy_executor::Spawner;
use embassy_rp::{gpio::{Input, Level, Output, Pull}, peripherals, pwm::{Pwm, SetDutyCycle}};
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
    
    let mut config: PwmConfig = Default::default();
    
    let mut left_button = Input::new (peripherals.PIN_18, Pull::Up);
    let mut ok_button = Input::new (peripherals.PIN_19, Pull::Up);
    let mut right_button = Input::new (peripherals.PIN_20, Pull::Up);
    
    config.top = 0x9088;
    
    let brightness = 1;

    config.compare_a = config.top / brightness;
    
    let mut on_light = Pwm::new_output_a(
        peripherals.PWM_SLICE0,
        peripherals.PIN_16,
        config.clone()
    );
    
    info!("PicoPlay has booted.");
    
    let delay = 50;
    let mut duty: u64 = 100;
    
    loop {
        Timer::after_millis(delay).await;
        if left_button.is_low() {
            info!("Left button pressed");
            duty -= 10;
            if duty < 10 {
                duty = 10;
            }
        }
        on_light.set_duty_cycle(duty.try_into().unwrap());
        Timer::after_millis(delay).await;
        if ok_button.is_low() {
            info!("OK button pressed");
            duty = 100;
        }
        on_light.set_duty_cycle(duty.try_into().unwrap());
        Timer::after_millis(delay).await;
        if right_button.is_low() {
            info!("Right button pressed");
            duty += 10;
        }
        on_light.set_duty_cycle(duty.try_into().unwrap());
        Timer::after_millis(delay).await;
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    error!("Panic occurred: {:?}", info);
    loop {}
}