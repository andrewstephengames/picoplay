#![no_main]
#![no_std]

use biski64::Biski64Rng;
use cyw43::JoinOptions;
use display_interface_spi::SPIInterface;
use embassy_net::StackResources;
use embedded_graphics::{draw_target::DrawTarget, image::{Image, ImageRawLE}, mono_font::{ascii::FONT_10X20, MonoTextStyle}, pixelcolor::BinaryColor, prelude::{Point, Size, WebColors}, primitives::{Line, PrimitiveStyle, Rectangle}, text::Text};
use embassy_executor::Spawner;
use embedded_graphics::prelude::Primitive;
use embassy_embedded_hal::shared_bus::blocking::spi::SpiDevice;
use embassy_rp::{gpio::{Input, Level, Output, Pull}, pac::{pll::regs::Prim, rosc::Rosc}, peripherals::{self, SPI0}, pwm::{Pwm, SetDutyCycle}, rom_data::set_ns_api_permission, spi, Peripherals};
use embassy_sync::blocking_mutex::NoopMutex;
use embassy_time::{Delay, Instant, Timer};
use embedded_graphics::{pixelcolor::Rgb565, prelude::RgbColor};
use ili9341::{Ili9341, Orientation, DisplaySize240x320, ModeState};
use rand_core::{RngCore, SeedableRng};
use static_cell::StaticCell;
use heapless::String;
use embassy_rp::pwm::Config as PwmConfig;
use embassy_rp::spi::{Spi, Blocking, Config as SpiConfig};
use core::{cell::RefCell, marker::Sized};
use defmt::{info, error};
use crate::peripherals::SPI1;
// use embassy_rp::peripherals::SPI1;
use core::panic::PanicInfo;
use core::fmt::Write;
use embedded_graphics::Drawable;
use defmt_rtt as _;

mod irqs;
mod secrets;

static SPI_BUS: StaticCell<NoopMutex<RefCell<Spi<'static, SPI1, Blocking>>>> = StaticCell::new();

const WINDOW_X: i32 = 320;
const WINDOW_Y: i32 = 240;
const CONSOLE_NAME: &str = "PicoPlay";
const WIFI_NETWORK: &str = secrets::ssid;
const WIFI_PASSWORD: &str = secrets::password;
const SOCK: usize = 4;
static RESOURCES: StaticCell<StackResources<SOCK>> = StaticCell::<StackResources<SOCK>>::new();

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
    mut right: Input<'static>,
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
	    display.clear(Rgb565::WHITE).unwrap();
	    let mut rect_style = PrimitiveStyle::with_stroke(Rgb565::BLACK, 1);
	    let mut text_style = MonoTextStyle::new(&FONT_10X20, Rgb565::BLACK);
        
        let mut rng = Biski64Rng::seed_from_u64(Instant::now().as_ticks());
        
    loop {
	
        text_style = MonoTextStyle::new(&FONT_10X20, Rgb565::BLACK);
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
	        display.clear(Rgb565::WHITE).unwrap();
            if menu.retro_heroes_active == true {
                info!("Retro Heroes launching");
                let mut player1_hp = 100;
                let mut player2_hp = 100;
                let mut player1_won = false;
                let mut player2_won = false;
                let mut line_style: PrimitiveStyle<Rgb565>;
                loop {
                    let player1 = ImageRawLE::new(include_bytes!("../res/player1.raw"), 128);
                    let player1_image = Image::new(&player1, Point::new(10, 10));
                    let player2 = ImageRawLE::new(include_bytes!("../res/player2.raw"), 128);
                    let player2_image = Image::new(&player2, Point::new(200, 10));
                    player1_image.draw(&mut display).unwrap();
                    player2_image.draw(&mut display).unwrap();
                    rect_style = PrimitiveStyle::with_fill(Rgb565::WHITE);
	                // Rectangle::new(Point::new(10, 10), Size::new(200, 200))
	                //     .into_styled(rect_style)
	                //     .draw(&mut display);
                    if !player1_won && !player2_won {
	                    if left.is_low() && right.is_high() && ok.is_high() {
	                        info!("Player 1 used attack!");
		                    Rectangle::new(Point::new(138, 0), Size::new(62, 138))
		                        .into_styled(rect_style)
		                        .draw(&mut display);
	                        line_style = PrimitiveStyle::with_stroke(Rgb565::RED, stroke_width);
	                        let y = rng.next_u64() as i32 % 140;
	                        Line::new(
	                            Point::new(140, rng.next_u64() as i32 %140),
	                            Point::new(200, y))
	                            .into_styled(line_style)
	                            .draw(&mut display);
		                    Rectangle::new(Point::new(200, y), Size::new(10, 10))
		                        .into_styled(line_style)
		                        .draw(&mut display);
	                        player2_hp -= 10;
		                    Rectangle::new(Point::new(200, 138), Size::new(100, 250))
		                        .into_styled(rect_style)
		                        .draw(&mut display);
	                    }
	                    if ok.is_low() && left.is_high() && right.is_high() {
	                        info!("Player 1 used heal!");
		                    Rectangle::new(Point::new(138, 0), Size::new(62, 138))
		                        .into_styled(rect_style)
		                        .draw(&mut display);
	                        line_style = PrimitiveStyle::with_stroke(Rgb565::YELLOW, stroke_width);
                            let y = rng.next_u64() as i32 % 140;
	                        Line::new(
	                            Point::new(140, rng.next_u64() as i32 %140),
	                            Point::new(200, y as i32 % 140))
	                            .into_styled(line_style)
	                            .draw(&mut display);
	                        Rectangle::new(Point::new(200, y), Size::new(10, 10))
	                            .into_styled(line_style)
	                            .draw(&mut display);
	                        player1_hp += 10;
		                    Rectangle::new(Point::new(30, 140), Size::new(140, 250))
		                        .into_styled(rect_style)
		                        .draw(&mut display);
	                    }
	                    if right.is_low() && ok.is_high() && left.is_high() {
	                        info!("Player 1 used special attack");
		                    Rectangle::new(Point::new(138, 0), Size::new(62, 138))
		                        .into_styled(rect_style)
		                        .draw(&mut display);
	                        line_style = PrimitiveStyle::with_stroke(Rgb565::CSS_PURPLE, stroke_width);
                            let y = rng.next_u64() as i32 % 140;
	                        Line::new(
	                            Point::new(140, rng.next_u64() as i32 % 140),
	                            Point::new(200, y))
	                            .into_styled(line_style)
	                            .draw(&mut display);
	                        Rectangle::new(Point::new(200, y), Size::new(10, 10))
	                            .into_styled(line_style)
	                            .draw(&mut display);
	                        player2_hp -= 20;
		                    Rectangle::new(Point::new(200, 138), Size::new(100, 250))
		                        .into_styled(rect_style)
		                        .draw(&mut display);
	                    }
                    }
                    if player1_hp < 0 {
                        player1_hp = 0;
                    }
                    if player1_hp > 100 {
                        player1_hp = 100;
                    }
                    if player2_hp < 0 {
                        player2_hp = 0;
                    }
                    if player2_hp > 100 {
                        player2_hp = 100;
                    }
                    if player1_hp == 0 {
                        player1_won = true;
	                    Text::new("Player 2 won the game!", Point::new(20, 80), text_style).draw(&mut display);
                    }
                    if player2_hp == 0 {
                        player1_won = true;
	                    Text::new("Player 1 won the game!", Point::new(20, 80), text_style).draw(&mut display);
                    }
                    let mut hp_buf1: String<32> = String::new();
                    write!(&mut hp_buf1, "HP: {}", player1_hp).unwrap();
	                Text::new(&hp_buf1, Point::new(30, 160), text_style).draw(&mut display);
                    let mut hp_buf2: String<32> = String::new();
                    write!(&mut hp_buf2, "HP: {}", player2_hp).unwrap();
	                Text::new(&hp_buf2, Point::new(200, 160), text_style).draw(&mut display);
                    if ok.is_low() && left.is_low() && right.is_high() {
	                    display.clear(Rgb565::WHITE).unwrap();
                        break;
                    }
                    if ok.is_low() && left.is_low() && right.is_low() {
	                    display.clear(Rgb565::WHITE).unwrap();
                    }
                    
                }
            }
            if menu.other_games_active == true {
                info!("Other games launching")
            }
        }
        
        
        if ok.is_low() && left.is_low() && right.is_low() {
	        display.clear(Rgb565::WHITE).unwrap();
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
    
    let mut connect_light = Output::new(peripherals.PIN_16, Level::Low);
    
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
    
    let (net_device, mut control) = embassy_lab_utils::init_wifi!(&spawner, peripherals).await;
    let config = embassy_net::Config::dhcpv4(Default::default());
    let stack = embassy_lab_utils::init_network_stack(&spawner, net_device, &RESOURCES, config);
        loop {
        match control.join(WIFI_NETWORK, JoinOptions::new(WIFI_PASSWORD.as_bytes())).await {
            Ok(_) => break,
            Err(err) => {
                info!("join failed with status={}", err.status);
            }
        }
    }
    info!("waiting for DHCP...");
    while !stack.is_config_up() {
        Timer::after_millis(100).await;
    }
    info!("DHCP is now up!");
    connect_light.set_high();
    spawner.spawn(display_task(spi_bus, cs, dc, reset, left_button, ok_button, right_button)).unwrap();

}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    error!("Panic occurred: {:?}", info);
    loop {}
}