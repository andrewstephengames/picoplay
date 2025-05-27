#![no_main]
#![no_std]

use display_interface_spi::SPIInterface;
use embedded_graphics::{draw_target::DrawTarget, mono_font::{ascii::FONT_10X20, MonoTextStyle}, pixelcolor::BinaryColor, prelude::{Point, Size, WebColors}, primitives::{PrimitiveStyle, Rectangle}, text::Text};
use embassy_executor::Spawner;
use embedded_graphics::prelude::Primitive;
use embassy_embedded_hal::shared_bus::blocking::spi::SpiDevice;
use embassy_rp::{gpio::{Input, Level, Output, Pull}, peripherals::{self, SPI0}, pwm::{Pwm, SetDutyCycle}, rom_data::set_ns_api_permission, spi, Peripherals};
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
use embedded_graphics::Drawable;
use defmt_rtt as _;

mod irqs;

static SPI_BUS: StaticCell<NoopMutex<RefCell<Spi<'static, SPI1, Blocking>>>> = StaticCell::new();

const WINDOW_X: i32 = 320;
const WINDOW_Y: i32 = 240;
const CONSOLE_NAME: &str = "PicoPlay";

struct Menu {
    retro_heroes_active: bool,
    other_games_active: bool,
}

#[embassy_executor::task]
async fn display_task(
    spi_bus: &'static NoopMutex<RefCell<Spi<'static, SPI1, Blocking>>>,
    mut cs: Output<'static>,
    mut dc: Output<'static>,
    mut reset: Output<'static>,
    mut left: Input<'static>,
    mut ok: Input<'static>,
    mut right: Input<'static>
) {
    let spi_dev = SpiDevice::new(&spi_bus, cs);
    let iface = SPIInterface::new(spi_dev, dc);

    let mut delay = Delay;
    
    let mut menu= Menu {retro_heroes_active: true, other_games_active: false};

    let mut display = Ili9341::new(
        iface,
        reset,
        &mut delay,
        Orientation::Landscape,
        DisplaySize240x320
    ).unwrap();

	    display.idle_mode(ModeState::Off).unwrap();
	    // display.invert_mode(ModeState::On).unwrap();
	    let _ = display.normal_mode_frame_rate(ili9341::FrameRateClockDivision::Fosc, ili9341::FrameRate::FrameRate100);
	    display.clear(Rgb565::CSS_LIGHT_BLUE).unwrap();
	    let mut rect_style = PrimitiveStyle::with_stroke(Rgb565::BLACK, 1);
	    let text_style = MonoTextStyle::new(&FONT_10X20, Rgb565::BLACK);
    loop {
	
	    Text::new(CONSOLE_NAME, Point::new(WINDOW_X/2-2*8, 20), text_style).draw(&mut display);
	    Text::new("What would you like to play?", Point::new(20, 80), text_style).draw(&mut display);
	
	    let retroheroes_label = "Retro Heroes";
	    let other_games_label = "Other Games";
        let scale = 5;
        let button_scale = 12;
        let stroke_width = 2;

        if left.is_low() || right.is_low() {
            menu.retro_heroes_active = !menu.retro_heroes_active;
            menu.other_games_active = !menu.other_games_active;
        }
        
        if ok.is_low() {
            if menu.retro_heroes_active == true {
                info!("Retro Heroes launching");
            }
            if menu.other_games_active == true {
                info!("Other games launching")
            }
        }
        
        
        if menu.retro_heroes_active == true {
            rect_style = PrimitiveStyle::with_stroke(Rgb565::CSS_LIME, stroke_width);
        } else {
            rect_style = PrimitiveStyle::with_stroke(Rgb565::BLACK, stroke_width);
        }
	    Rectangle::new(Point::new(WINDOW_X/2-retroheroes_label.len() as i32 * scale, 100), Size::new(retroheroes_label.len() as u32 * button_scale, 30))
	        .into_styled(rect_style)
	        .draw(&mut display);
	    Text::new(retroheroes_label, Point::new(WINDOW_X/2i32-retroheroes_label.len() as i32 * scale, 120), text_style).draw(&mut display);

        if menu.other_games_active == true {
            rect_style = PrimitiveStyle::with_stroke(Rgb565::GREEN, stroke_width);
        } else {
            rect_style = PrimitiveStyle::with_stroke(Rgb565::BLACK, stroke_width);
        }
	
	    Rectangle::new(Point::new(WINDOW_X/2-other_games_label.len() as i32 * scale, 130), Size::new(other_games_label.len() as u32 *button_scale, 30))
	        .into_styled(rect_style)
	        .draw(&mut display);
	    Text::new(other_games_label, Point::new(WINDOW_X/2i32-other_games_label.len() as i32 * scale, 150), text_style).draw(&mut display);
    }
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

    let mut buzzer_config: PwmConfig = Default::default();
    buzzer_config.top = 0x9088;

    let mut volume: u16 = 1;
    buzzer_config.compare_b = buzzer_config.top / volume;

    let mut buzzer = Pwm::new_output_b(
        peripherals.PWM_SLICE2,
        peripherals.PIN_21,
        buzzer_config.clone()
    );
    
    let mut volume2 = 0;
    buzzer.set_duty_cycle(volume2);

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

    spawner.spawn(display_task(spi_bus, cs, dc, reset, left_button, ok_button, right_button)).unwrap();

    
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