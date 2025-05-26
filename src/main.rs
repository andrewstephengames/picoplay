#![no_main]
#![no_std]

use display_interface_spi::SPIInterface;
use embassy_executor::Spawner;
use embassy_embedded_hal::shared_bus::blocking::spi::SpiDevice;
use embassy_rp::{gpio::{Input, Level, Output, Pull}, peripherals::{self, SPI0}, pwm::{Pwm, SetDutyCycle}, rom_data::set_ns_api_permission, spi};
use embassy_sync::blocking_mutex::NoopMutex;
use embassy_time::{Delay, Timer};
use embedded_graphics::{pixelcolor::Rgb565, prelude::RgbColor};
use ili9341::{Ili9341, Orientation, DisplaySize240x320, ModeState};
use static_cell::StaticCell;
use embassy_rp::pwm::Config as PwmConfig;
use embassy_rp::spi::{Spi, Blocking, Config as SpiConfig};
use core::{cell::RefCell, marker::Sized};
use defmt::{info, error};
use crate::peripherals::SPI1;
// use embassy_rp::peripherals::SPI1;
use core::panic::PanicInfo;
use defmt_rtt as _;

mod irqs;

static SPI_BUS: StaticCell<NoopMutex<RefCell<Spi<'static, SPI1, Blocking>>>> = StaticCell::new();
#[embassy_executor::task]
async fn display_task(
    spi_bus: &'static NoopMutex<RefCell<Spi<'static, SPI1, Blocking>>>,
    mut cs: Output<'static>,
    mut dc: Output<'static>,
    mut reset: Output<'static>,
) {
    let spi_dev = SpiDevice::new(&spi_bus, cs);
    let iface = SPIInterface::new(spi_dev, dc);

    let mut delay = Delay;

    let mut display = Ili9341::new(iface, reset, &mut delay, Orientation::Landscape, DisplaySize240x320).unwrap();

    display.idle_mode(ModeState::Off).unwrap();
    display.invert_mode(ModeState::On).unwrap();
    let _ = display.normal_mode_frame_rate(ili9341::FrameRateClockDivision::Fosc, ili9341::FrameRate::FrameRate100);
    // display.clear(Rgb565::BLACK).unwrap();
}


#[embassy_executor::main]
async fn main(spawner: Spawner) {
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

    let mut spi = Spi::new_blocking(
        peripherals.SPI1,
        peripherals.PIN_14, // SCK
        peripherals.PIN_11, // MOSI
        peripherals.PIN_12, // MISO
        // peripherals.DMA_CH0,
        // peripherals.DMA_CH1,
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
    
    let delay = Delay;
    let mut duty: u64 = 100;
    let spi_bus = NoopMutex::new(RefCell::new(spi));
    let spi_bus = SPI_BUS.init(spi_bus);     // for sending to task
    
    // let mut bl = Pwm::new_output_b(
    //     peripherals.PWM_SLICE2,
    //     peripherals.PIN_5,
    //     config.clone()
    // );
    // display.clear(Rgb565::RED).unwrap();
    // 
    let mut delay2 = 50;

    spawner.spawn(display_task(spi_bus, cs, dc, reset)).unwrap();
    
    // loop {
    //     Timer::after_millis(delay2).await;
    //     if left_button.is_low() {
    //         info!("Left button pressed");
    //         duty -= 10;
    //         if duty < 10 {
    //             duty = 10;
    //         }
    //     }
    //     on_light.set_duty_cycle(duty.try_into().unwrap());
    //     Timer::after_millis(delay2).await;
    //     if ok_button.is_low() {
    //         info!("OK button pressed");
    //         duty = 100;
    //     }
    //     on_light.set_duty_cycle(duty.try_into().unwrap());
    //     Timer::after_millis(delay2).await;
    //     if right_button.is_low() {
    //         info!("Right button pressed");
    //         duty += 10;
    //     }
    //     on_light.set_duty_cycle(duty.try_into().unwrap());
    //     Timer::after_millis(delay2).await;
    // }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    error!("Panic occurred: {:?}", info);
    loop {}
}