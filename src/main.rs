extern crate estatic;

fn main() {
    let world = estatic::world::World::new_empty(200, 200, 2);
    let view = estatic::viewer::ViewState::new(world);

    estatic::viewer::drawing::render_loop(view);
}
