pub mod drawing;
pub mod input;
pub mod inputbox;

use Vector;
use world::*;

bitflags! {
    pub struct DrawSets: u32 {
        const POTENTIAL = 0b00000001;
        const FIELD = 0b00000010;
        const FIELD_LINES = 0b00000100;
    }
}

/// The view on the world used for rendering
pub struct ViewState {
    pub world: World,
    // The field has changed and so the view must be updated
    pub changed: bool,
    // The editing charge
    pub charge: i8,

    // The draw setting (show potential, show field, show lines)
    pub draw_settings: DrawSets,

    // The scale of the visualization
    pub scale: f64,
    // The offset from the center
    pub offset: Vector,

    // The window width & height
    pub width: u32,
    pub height: u32,
}

impl ViewState {
    pub fn new(world: World) -> ViewState {
        let mut view = ViewState {
            world,
            changed: true,
            charge: 127,
            draw_settings: DrawSets::POTENTIAL | DrawSets::FIELD | DrawSets::FIELD_LINES,
            scale: 10.0,
            width: 1,
            height: 1,
            offset: Vector::new(0.0, 0.0),
        };

        view.center_view();
        view
    }

    /// Convert world coordinates to view coordinates
    pub fn get_screen_pos(&self, x: f64, y: f64) -> Vector {
        Vector::new(
            (x + self.offset.x) * self.scale + self.width as f64 / 2.0,
            (-y + self.offset.y) * self.scale + self.height as f64 / 2.0,
        )
    }

    /// Convert view coordinates to world coordinates
    pub fn get_world_pos(&self, x: f64, y: f64) -> Vector {
        Vector::new(
            (x - self.width as f64 / 2.0) / self.scale - self.offset.x,
            (-y + self.height as f64 / 2.0) / self.scale + self.offset.y,
        )
    }

    /// Are the coordinates inside the screen?
    pub fn in_screen(&self, x: f64, y: f64) -> bool {
        if x > self.width as f64 || y > self.height as f64 || x < 0.0 || y < 0.0 {
            false
        } else {
            true
        }
    }

    /// Centers the view on the world
    pub fn center_view(&mut self) {
        self.offset.x = -(self.world.width as f64 / 2.0);
        self.offset.y = self.world.height as f64 / 2.0;
    }
}
