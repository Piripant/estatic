use piston_window::*;
use viewer::input;
use viewer::inputbox::InputBox;

use super::*;

use image;
use image::ImageBuffer;

pub fn render_loop(mut view: ViewState) {
    let mut input_state = input::InputState::new();

    view.offset.x = -(view.world.width as f64 / 2.0);
    view.offset.y = view.world.height as f64 / 2.0;

    let title = "Estatic";
    let mut window: PistonWindow = WindowSettings::new(title, [640, 480])
        .build()
        .unwrap_or_else(|e| panic!("Failed to build PistonWindow: {}", e));

    let mut texture = empty_texture(&mut window.factory, view.world.width, view.world.height);
    let mut field_lines = Vec::new();

    let mut width_input = InputBox::new(window.factory.clone(), "Width", (10.0, 24.0));
    let mut height_input = InputBox::new(window.factory.clone(), "Height", (10.0, 50.0));

    let mut width = view.world.width;
    let mut height = view.world.height;

    while let Some(e) = window.next() {
        input_state.event(&e);

        width_input.input(&mut width);
        height_input.input(&mut height);
        if width != view.world.width || height != view.world.height {
            view.world = World::new_empty(width, height);
            texture = empty_texture(&mut window.factory, width, height);
            view.changed = true;
        }

        if view.changed {
            view.world.calculate_field();
            field_lines = view.world.calculate_lines();

            update_texture(
                &view.world,
                view.draw_settings,
                &mut texture,
                &mut window.encoder,
            );
            view.changed = false;
        }

        if let Some(_args) = e.render_args() {
            view.width = window.size().width;
            view.height = window.size().height;

            window.draw_2d(&e, |c, g| {
                clear([1.0, 1.0, 1.0, 1.0], g);
                let position = view.get_screen_pos(0.0, view.world.height as f64);

                // Render the map
                let trans = c.transform
                    .trans(position.x, position.y)
                    .scale(view.scale, view.scale);
                image(&texture, trans, g);

                if view.draw_settings.contains(DrawSets::FIELD_LINES) {
                    for field_line in &field_lines {
                        for i in 0..field_line.len() - 1 {
                            let position = &field_line[i];
                            let n_position = &field_line[i + 1];

                            let pos = view.get_screen_pos(position.x as f64, position.y as f64);
                            let npos =
                                view.get_screen_pos(n_position.x as f64, n_position.y as f64);

                            let line_data = [pos.x, pos.y, npos.x, npos.y];
                            line([0.1, 0.1, 0.1, 0.8], 1.0, line_data, c.transform, g);
                        }
                    }
                }

                width_input.update(&mut input_state, &c, g);
                height_input.update(&mut input_state, &c, g);

                input::handle_input(&mut view, &mut input_state);
                input_state.processed();
            });
        }
    }
}

fn empty_texture(factory: &mut GfxFactory, width: u32, height: u32) -> G2dTexture {
    use piston_window::texture::{CreateTexture, Format};

    CreateTexture::create(
        factory,
        Format::Rgba8,
        &[0u8; 4],
        [width, height],
        &TextureSettings::new().filter(Filter::Nearest),
    ).unwrap()
}

fn update_texture(
    world: &World,
    settings: DrawSets,
    texture: &mut G2dTexture,
    encoder: &mut GfxEncoder,
) {
    use image::Pixel;

    let mut imgbuf = ImageBuffer::new(world.width, world.height);

    for (x, y, pixel) in imgbuf.enumerate_pixels_mut() {
        // Flip the y axis
        let y = world.height - 1 - y;

        let &(ref force, potential) = world
            .field
            .get(&Vector::new(x as f64 + 0.5, y as f64 + 0.5));
        let charge = world.tiles[y as usize][x as usize];

        // Draw tiles
        let tile_pixel = {
            let intensity = charge.abs() as u8 * 2;

            if charge > 0 {
                image::Rgba([intensity, 0, 0, 255])
            } else if charge < 0 {
                image::Rgba([0, 0, intensity, 255])
            } else {
                image::Rgba([0, 0, 0, 0])
            }
        };

        // Draw potential
        let pot_pixel = {
            let intensity = 255.0 - potential.abs();
            let intensity = if intensity < 0.0 { 0.0 } else { intensity };

            if potential > 0.0 {
                image::Rgba([255, intensity as u8, intensity as u8, 127])
            } else {
                image::Rgba([intensity as u8, intensity as u8, 255, 127])
            }
        };

        let field_pixel = {
            let force = 255.0 - force.norm();
            let force = if force < 0.0 { 0.0 } else { force };

            image::Rgba([force as u8, force as u8, force as u8, 255])
        };

        if settings.contains(DrawSets::FIELD) {
            *pixel = field_pixel;
            if settings.contains(DrawSets::POTENTIAL) {
                pixel.blend(&pot_pixel);
            }
            pixel.blend(&tile_pixel);
        } else if settings.contains(DrawSets::POTENTIAL) {
            *pixel = pot_pixel;
            pixel.blend(&tile_pixel);
        } else {
            *pixel = tile_pixel;
        }
    }

    texture.update(encoder, &imgbuf).unwrap();
}
