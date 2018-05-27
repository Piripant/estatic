use Vector;
use piston_window::*;
use viewer::ViewState;

use super::*;

pub struct InputState {
    pub last_cursor: Vector,
    pub cursor: Vector,
    pub pressed_mouse: Option<MouseButton>,
    pub held_mouse: Option<MouseButton>,
    pub released_mouse: Option<MouseButton>,
    pub mouse_wheel: f64,
    pub pressed_keys: Vec<Key>,
    pub held_keys: Vec<Key>,
    pub released_keys: Vec<Key>,
}

impl InputState {
    pub fn new() -> InputState {
        InputState {
            last_cursor: Vector::new(0.0, 0.0),
            cursor: Vector::new(0.0, 0.0),
            pressed_mouse: None,
            held_mouse: None,
            released_mouse: None,
            mouse_wheel: 0.0,
            pressed_keys: Vec::new(),
            held_keys: Vec::new(),
            released_keys: Vec::new(),
        }
    }

    /// Called once all the input is been processed
    /// Sets as held all the keys/mouse buttons that were pressed
    pub fn processed(&mut self) {
        self.last_cursor = self.cursor;

        // Everything that was now pressed becomes held
        if self.pressed_mouse.is_some() {
            self.held_mouse = self.pressed_mouse;
            self.pressed_mouse = None;
        }

        // Everything that was now pressed becomes held
        for _ in 0..self.pressed_keys.len() {
            let key = self.pressed_keys.remove(0);
            self.held_keys.push(key);
        }

        // Remove all the released keys
        self.released_keys.clear();
        self.released_mouse = None;

        self.mouse_wheel = 0.0;
    }

    /// Updates the current Input State
    pub fn event(&mut self, e: &Event) {
        e.mouse_cursor(|x, y| {
            self.cursor.x = x;
            self.cursor.y = y;
        });

        e.mouse_scroll(|_dx, dy| {
            self.mouse_wheel += dy;
        });

        if let Some(Button::Keyboard(key)) = e.press_args() {
            // Add the key only if it wasn't already added
            if !self.pressed_keys.contains(&key) && !self.held_keys.contains(&key) {
                self.pressed_keys.push(key);
            }
        };

        // Remove the release keys form the held and pressed keys
        // (In case one key was pressed and released before the events were processed)
        if let Some(Button::Keyboard(key)) = e.release_args() {
            for i in 0..self.pressed_keys.len() {
                if self.pressed_keys[i] == key {
                    self.pressed_keys.remove(i);
                    break;
                }
            }

            for i in 0..self.held_keys.len() {
                if self.held_keys[i] == key {
                    self.held_keys.remove(i);
                    break;
                }
            }

            self.released_keys.push(key);
        }

        if let Some(Button::Mouse(button)) = e.press_args() {
            self.pressed_mouse = Some(button);
        }

        if let Some(Button::Mouse(button)) = e.release_args() {
            self.pressed_mouse = None;
            self.held_mouse = None;
            self.released_mouse = Some(button);
        }
    }
}

pub fn handle_input(view: &mut ViewState, input: &mut InputState) {
    if input.pressed_keys.contains(&Key::C) {
        view.charge = -view.charge;
    } else if input.pressed_keys.contains(&Key::P) {
        view.draw_settings.toggle(DrawSets::POTENTIAL);
        view.changed = true;
    } else if input.pressed_keys.contains(&Key::L) {
        view.draw_settings.toggle(DrawSets::FIELD_LINES);
        view.changed = true;
    } else if input.pressed_keys.contains(&Key::F) {
        view.draw_settings.toggle(DrawSets::FIELD);
        view.changed = true;
    } else if input.pressed_keys.contains(&Key::Space) {
        view.offset.x = -(view.world.width as f64 / 2.0);
        view.offset.y = view.world.height as f64 / 2.0;
    }

    handle_move(view, input);
    handle_edit(view, input);
}

fn handle_move(view: &mut ViewState, input: &InputState) {
    if let Some(MouseButton::Right) = input.held_mouse {
        if input.held_keys.contains(&Key::LShift) {
            let delta_mouse = input.cursor - input.last_cursor;
            view.offset += delta_mouse / view.scale;
        }
    }

    view.scale += input.mouse_wheel;
}

fn handle_edit(view: &mut ViewState, input: &InputState) {
    let cursor = view.get_world_pos(input.cursor.x, input.cursor.y);

    if view.world.in_bounds(cursor.x as i32, cursor.y as i32) {
        match input.held_mouse {
            Some(MouseButton::Left) => {
                view.changed =
                    view.world
                        .update_tile(view.charge, cursor.x as usize, cursor.y as usize);
            }
            Some(MouseButton::Right) => {
                view.changed = view.world
                    .update_tile(0, cursor.x as usize, cursor.y as usize);
            }

            _ => {}
        }
    }
}
