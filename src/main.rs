#![no_main]
#![no_std]

use display_interface_spi::SPIInterface;
use embassy_embedded_hal::shared_bus::asynch::spi::SpiDevice;
use embassy_executor::Spawner;
use embassy_rp::{gpio::{Input, Level, Output, Pull}, peripherals, pwm::{Pwm, SetDutyCycle}, rom_data::set_ns_api_permission, spi};
use embassy_time::{Delay, Timer};
use embassy_rp::pwm::Config as PwmConfig;
use embassy_rp::spi::{Spi, Config as SpiConfig};
use ili9341::{Ili9341, Orientation};
use lcd_ili9341_spi::Lcd; 
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
    
    let mut spi_config = SpiConfig::default();
    spi_config.frequency = 40_000_000;

    let mut spi = Spi::new(
        peripherals.SPI1,
        peripherals.PIN_14, // SCK
        peripherals.PIN_11, // MOSI
        peripherals.PIN_12, // MISO
        peripherals.DMA_CH0,
        peripherals.DMA_CH1,
        spi_config   
    );
    
    let dc = Output::new(peripherals.PIN_10, Level::Low);
    let cs = Output::new(peripherals.PIN_13, Level::Low);
    let mut reset = Output::new(peripherals.PIN_15, Level::High);
    
    reset.set_low();
    Timer::after_millis(10);
    reset.set_high();
    Timer::after_millis(10);
    
    
    info!("PicoPlay has booted.");
    
    let delay = 50;
    let mut duty: u64 = 100;
    
    // let spi_peripheral = Spi::new(
    //     LCD_SPI,
    //     peripherals::spi
    // );

    // let iface = SPIInterface::new(spi, dc);

    // let mut display = Ili9341::new(
    //     iface,
    //     reset,
    //     &mut delay,
    //     Orientation::Landscape,
    //     ili9341::DisplaySize240x320,
    // );
    let mut bl = Pwm::new_output_b(
        peripherals.PWM_SLICE2,
        peripherals.PIN_5,
        config.clone()
    );
    let mut lcd = Lcd::new(spi, dc, reset, bl);
    let mut lcd_delay = Delay;
    let _ = lcd.init(&mut lcd_delay);
    // .unwrap();
    
    // display.clear(Rgb565::RED).unwrap()
    
    loop {
        let _ = lcd.clear(0x0000);
        let _ = lcd.set_backlight(30);
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