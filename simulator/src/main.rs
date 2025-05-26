use embedded_graphics::*;
use embedded_graphics::{
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::{Circle, Line, Rectangle, PrimitiveStyle},
    mono_font::{ascii::FONT_6X9, MonoTextStyle},
    text::Text,
};
use embedded_graphics_simulator::*;
use embedded_graphics_simulator::{BinaryColorTheme, SimulatorDisplay, Window, OutputSettingsBuilder};

const WINDOW_X: i32 = 320;
const WINDOW_Y: i32 = 240;
const CONSOLE_NAME: &str = "PicoPlay";

// struct Button {
//     rect: Rectangle,
//     label: String,
//     active: bool,
// }

// impl <C> Button {
//     fn new(&self, top_left: Point, label: &str) -> &self {
//         self.rect.top_left = top_left;
//         self.label = String::from(&str);
//         self.rect.size = Size {width: self.label.len() as u32, height: 10};
//         self.active = false;
//     }
//     fn draw(&self, display: SimulatorDisplay<C>, text_style: MonoTextStyle<C>, line_style: Styled) {
//         self.rect.into_styled(line_style).draw(&mut display);
//         Text::new(self.label.as_str(), self.rect.top_left, text_style).draw(&mut display)?;
//     }
// }

enum State {
    Menu,
    RetroHeroes,
    Other,
}

fn main() -> Result<(), core::convert::Infallible> {
    let mut display = SimulatorDisplay::<BinaryColor>::new(Size::new(WINDOW_X as u32, WINDOW_Y as u32));

    let line_style = PrimitiveStyle::with_stroke(BinaryColor::On, 1);
    let text_style = MonoTextStyle::new(&FONT_6X9, BinaryColor::On);

    Text::new(CONSOLE_NAME, Point::new(WINDOW_X/2-2*8, 20), text_style).draw(&mut display)?;
    Text::new("What would you like to play?", Point::new(80, 80), text_style).draw(&mut display)?;

    let retroheroes_label = String::from("Retro Heroes");
    let other_games_label = String::from("Other Games");
    
    Rectangle::new(Point::new(WINDOW_X/2-retroheroes_label.len() as i32 * 5/2, 100), Size::new(retroheroes_label.len() as u32 *8, 30))
        .into_styled(line_style)
        .draw(&mut display)?;
    Text::new(retroheroes_label.as_str(), Point::new(WINDOW_X/2i32-retroheroes_label.len() as i32 * 2, 120), text_style).draw(&mut display)?;

    Rectangle::new(Point::new(WINDOW_X/2-other_games_label.len() as i32 * 5/2, 130), Size::new(other_games_label.len() as u32 *8, 30))
        .into_styled(line_style)
        .draw(&mut display)?;
    Text::new(other_games_label.as_str(), Point::new(WINDOW_X/2i32-other_games_label.len() as i32 * 2, 140), text_style).draw(&mut display)?;

    let output_settings = OutputSettingsBuilder::new()
        .theme(BinaryColorTheme::OledBlue)
        .build();
    Window::new(CONSOLE_NAME, &output_settings).show_static(&display);
    
    // let mut retroheroes_button: Button;
    // retroheroes_button = Button::new(&retroheroes_button, Point::new (WINDOW_X/2 - 30, 100), "Retro Heroes");
    // retroheroes_button.draw(&retroheroes_button, &mut display, text_style, line_style);

    // buttons.insert(Button::new(Point {x: }))
    
    
    Ok(())
}