use Vector;

#[derive(Debug, Clone)]
/// A field grid which can be bigger than the tiles grid
pub struct FieldGrid {
    ratio: u8,
    grid: Vec<Vec<(Vector, f64)>>,
}

impl FieldGrid {
    fn new(original_width: usize, original_height: usize, ratio: u8) -> FieldGrid {
        let width = original_width * ratio as usize;
        let height = original_height * ratio as usize;

        let mut grid = Vec::new();
        for _ in 0..height {
            let row = vec![(Vector::new(0.0, 0.0), 0.0); width];
            grid.push(row);
        }

        FieldGrid { ratio, grid }
    }

    #[inline]
    // Get a field tile using tiles' coordinates
    pub fn get(&self, position: &Vector) -> &(Vector, f64) {
        let x = position.x * self.ratio as f64;
        let y = position.y * self.ratio as f64;

        &self.grid[y as usize][x as usize]
    }

    #[inline]
    // Get a mutable reference to the field vector using tiles' coordinates
    pub fn get_mut(&mut self, position: &Vector) -> &mut (Vector, f64) {
        let x = position.x * self.ratio as f64;
        let y = position.y * self.ratio as f64;

        &mut self.grid[y as usize][x as usize]
    }
}

#[derive(Debug, Clone)]
/// The world containing all information for the simulation
pub struct World {
    pub height: u32,
    pub width: u32,
    pub tiles: Vec<Vec<i8>>,
    // (old_charge, x, y)
    pub updated_tiles: Vec<(i8, usize, usize)>,
    pub field: FieldGrid,
}

impl World {
    pub fn new_empty(width: u32, height: u32, resolution: u8) -> World {
        // Init the tiles and field to an empty space
        let mut tiles = Vec::new();
        for _ in 0..height {
            tiles.push(vec![0 as i8; width as usize]);
        }

        // The field_ratio must be an odd number
        // So there are always centered tiles in the subdivision
        let field_ratio = 2 * resolution - 1;
        let field = FieldGrid::new(width as usize, height as usize, field_ratio);
        let updated_tiles = Vec::new();

        World {
            height,
            width,
            tiles,
            updated_tiles,
            field,
        }
    }

    pub fn set_resolution(&mut self, resolution: u8) {
        // The field_ratio must be an odd number
        // So there are always centered tiles in the subdivision
        let field_ratio = 2 * resolution - 1;
        self.field = FieldGrid::new(self.width as usize, self.height as usize, field_ratio);

        // The first number must be 0 because we have already reset the field
        // When we created the new one with the new resolution
        let charges = self.get_charges()
            .iter()
            .map(|&charge| (0, charge.0, charge.1))
            .collect();
        self.updated_tiles = charges;
    }

    pub fn resolution(&self) -> u8 {
        (self.field.ratio + 1) / 2
    }

    pub fn update_tile(&mut self, charge: i8, x: usize, y: usize) -> bool {
        // If the tiles doesnt already have this charge
        if self.tiles[y][x] != charge {
            self.updated_tiles.push((self.tiles[y][x], x, y));
            self.tiles[y][x] = charge;
            true
        } else {
            false
        }
    }

    pub fn in_bounds(&self, x: i32, y: i32) -> bool {
        if x >= 0 && x < self.width as i32 && y >= 0 && y < self.height as i32 {
            true
        } else {
            false
        }
    }

    pub fn get_charges(&self) -> Vec<(usize, usize)> {
        let mut charges = Vec::new();
        for x in 0..self.width {
            for y in 0..self.height {
                let charge = self.tiles[y as usize][x as usize];
                // If the tile is charged
                if charge != 0 {
                    charges.push((x as usize, y as usize));
                }
            }
        }

        charges
    }

    pub fn get_borders(&self) -> Vec<(i8, usize, usize)> {
        // A moore neighborhood
        let directions = [
            (-1, 0),
            (1, 0),
            (0, -1),
            (0, 1),
            (1, 1),
            (1, -1),
            (-1, 1),
            (-1, -1),
        ];

        let charges = self.get_charges();
        let mut borders = Vec::new();

        for &(cx, cy) in &charges {
            for &(dx, dy) in directions.iter() {
                let nx = cx as i32 + dx;
                let ny = cy as i32 + dy;

                // If the neighbor is within the grid
                if self.in_bounds(nx, ny) {
                    // If the neighbor is not charged & we haven't already added it
                    if self.tiles[ny as usize][nx as usize] == 0
                        && !borders.contains(&(self.tiles[cy][cx], nx as usize, ny as usize))
                    {
                        borders.push((self.tiles[cy][cx], nx as usize, ny as usize));
                    }
                }
            }
        }

        borders
    }

    pub fn calculate_field(&mut self) {
        // Update the grid using the field grid coordinates
        let width = self.width as usize * self.field.ratio as usize;
        let height = self.height as usize * self.field.ratio as usize;

        for x in 0..width {
            for y in 0..height {
                let &mut (ref mut field_force, ref mut potential) = &mut self.field.grid[y][x];

                // The position of the field on the tiles grid
                let real_position =
                    Vector::new(x as f64 + 0.5, y as f64 + 0.5) / self.field.ratio as f64;

                for &(ref old_charge, ref cx, ref cy) in &self.updated_tiles {
                    // Charge of the updated tile
                    let charge = &self.tiles[*cy as usize][*cx as usize];
                    // The position of neighbor
                    let n_position = Vector::new(*cx as f64 + 0.5, *cy as f64 + 0.5);
                    // The distance between the position of the updated tile
                    // and the position of the field tile we are updating
                    let delta = real_position - n_position;

                    if *old_charge != 0 {
                        // Remove the field that was once generated
                        let (old_field, old_potential) = get_field(old_charge, &delta);
                        *potential -= old_potential;
                        *field_force -= old_field;
                    }
                    if *charge != 0 {
                        // Add the new field of the updated charge
                        let (new_field, new_potential) = get_field(charge, &delta);
                        *potential += new_potential;
                        *field_force += new_field;
                    }
                }
            }
        }

        // All tiles have been updated
        self.updated_tiles.clear();
    }

    pub fn calculate_lines(&mut self) -> Vec<Vec<Vector>> {
        use std::f64;

        let max_length = 2000;
        let borders = self.get_borders();
        let mut lines = Vec::new();

        'charges: for &(charge, x, y) in &borders {
            let mut line = Vec::new();
            let (mut x, mut y) = (x as i32, y as i32);
            // The sign the charge is needed to move in the right direction along the field
            let charge = charge.signum() as f64;

            let mut old_angle: f64 = f64::INFINITY;
            let mut position = Vector::new(x as f64 + 0.5, y as f64 + 0.5);
            let mut old_position = position;

            let mut length = 0;
            while self.in_bounds(x, y) && length < max_length {
                if self.tiles[y as usize][x as usize] != 0 {
                    if charge < 0.0 {
                        continue 'charges;
                    }
                    break;
                }

                let force = &self.field.get(&position).0;

                let angle = f64::atan2(force.y, force.x) * 10.0;
                // Only push the new point
                // If there was a significant change in the direction of the field line
                // (So we can save memory for straight lines)
                if angle.round() != old_angle.round() {
                    line.push(position);
                    old_angle = angle;
                }

                // Move the position along the field
                old_position = position;
                position += force.normalize() * charge / self.field.ratio as f64;
                y = position.y.floor() as i32;
                x = position.x.floor() as i32;

                length += 1;
            }

            // Add the last position
            // So that even straight lines have a last point
            line.push(old_position);
            lines.push(line);
        }

        lines
    }
}

#[inline]
/// Calculate the eletric field & potential of `charge` with distance `delta`
fn get_field(charge: &i8, delta: &Vector) -> (Vector, f64) {
    use std::f64;

    if delta.norm() != 0.0 {
        (
            delta.normalize() * *charge as f64 / delta.norm_squared(),
            *charge as f64 / delta.norm(),
        )
    } else {
        (Vector::new(0.0, 0.0), 0.0)
    }
}
