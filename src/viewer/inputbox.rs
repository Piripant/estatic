use viewer::input::InputState;

use piston_window::*;
use std::str::FromStr;
use std::string::ToString;

/// A GUI input box
pub struct InputBox {
    // Is the input box focused
    active: bool,
    description: String,

    // The current input held as a string
    input: String,

    // Offset from the screen upper left corner
    offset: (f64, f64),

    glyphs: Glyphs,
}

impl InputBox {
    pub fn new(factory: GfxFactory, offset: (f64, f64)) -> InputBox {
        let font = include_bytes!("../../assets/FiraSans-Regular.ttf");
        let glyphs = Glyphs::from_bytes(font, factory, TextureSettings::new()).unwrap();

        InputBox {
            active: false,
            description: String::new(),
            input: String::new(),
            offset,
            glyphs,
        }
    }

    pub fn description(mut self, description: &str) -> InputBox {
        self.description = description.to_string();
        self
    }

    /// Set the input box value from type T
    pub fn value<T>(mut self, value: T) -> InputBox
    where
        T: ToString,
    {
        self.input = value.to_string();
        self
    }

    // Get the input box value parsed to type T
    pub fn input<T>(&self, value: &mut T)
    where
        T: FromStr,
    {
        if !self.active {
            let new_value = self.input.parse();
            if new_value.is_ok() {
                *value = new_value.ok().unwrap();
            }
        }
    }

    // Render and process the input events
    pub fn update(&mut self, input: &mut InputState, c: &Context, g: &mut G2d) {
        let full_text = format!("{}: {}", self.description, self.input);

        let hitbox = text_hitbox(&full_text, 24, &mut self.glyphs);
        // The hitbox upper left and lower right coordinates
        let start = (self.offset.0, self.offset.1 - hitbox.1);
        let end = (self.offset.0 + hitbox.0, self.offset.1);
        rectangle(
            [0.9, 0.9, 0.9, 0.5], // red
            [start.0 - 2.0, start.1 - 2.0, hitbox.0 + 5.0, hitbox.1 + 5.0],
            c.transform,
            g,
        );

        text::Text::new_color([0.0, 0.0, 0.0, 1.0], 24)
            .draw(
                &full_text,
                &mut self.glyphs,
                &c.draw_state,
                c.transform.trans(self.offset.0, self.offset.1),
                g,
            )
            .unwrap();

        if input.pressed_mouse.is_some() {
            let cursor = input.cursor;
            if cursor.x > start.0 && cursor.x < end.0 && cursor.y > start.1 && cursor.y < end.1 {
                self.active = true;
                input.consume();
            } else {
                self.active = false;
            }
        }

        if input.pressed_keys.contains(&Key::Return) {
            self.active = false;
        }

        if self.active {
            for key in &input.pressed_keys {
                match key {
                    &Key::Backspace => {
                        self.input.pop();
                    }
                    &Key::Escape => {
                        self.input.clear();
                    }
                    key => {
                        let code = key.code();
                        let character = code as u8 as char;
                        self.input.push(character);
                    }
                }
            }

            input.processed();
        }
    }
}

/// Get the text width and height
fn text_hitbox(text: &str, size: u32, cache: &mut Glyphs) -> (f64, f64) {
    use piston_window::character::CharacterCache;

    let width = cache.width(size, text).unwrap();
    let mut height = 0.0;
    for ch in text.chars() {
        let character = cache.character(size, ch).unwrap();
        if character.top() > height {
            height = character.top();
        }
    }

    (width, height as f64)
}
