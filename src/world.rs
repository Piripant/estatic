use Vector;

#[derive(Debug, Clone)]
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
    pub fn get(&self, position: &Vector) -> &(Vector, f64) {
        let x = position.x * self.ratio as f64;
        let y = position.y * self.ratio as f64;

        &self.grid[y as usize][x as usize]
    }

    #[inline]
    pub fn get_mut(&mut self, position: &Vector) -> &mut (Vector, f64) {
        let x = position.x * self.ratio as f64;
        let y = position.y * self.ratio as f64;

        &mut self.grid[y as usize][x as usize]
    }
}

#[derive(Debug, Clone)]
pub struct World {
    pub height: u32,
    pub width: u32,
    pub tiles: Vec<Vec<i8>>,
    pub updated_tiles: Vec<(i8, usize, usize)>,
    pub field: FieldGrid,
}

impl World {
    pub fn new_empty(height: u32, width: u32) -> World {
        // Init the tiles and field to an empty space
        let mut tiles = Vec::new();
        for _ in 0..height {
            tiles.push(vec![0 as i8; width as usize]);
        }

        let field = FieldGrid::new(height as usize, width as usize, 3);
        let updated_tiles = Vec::new();

        World {
            height,
            width,
            tiles,
            updated_tiles,
            field,
        }
    }

    pub fn update_tile(&mut self, charge: i8, x: usize, y: usize) -> bool {
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
                if charge != 0 {
                    charges.push((x as usize, y as usize));
                }
            }
        }

        charges
    }

    pub fn get_borders(&self) -> Vec<(i8, usize, usize)> {
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

                if self.in_bounds(nx, ny) {
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
        let width = self.width as usize * self.field.ratio as usize;
        let height = self.height as usize * self.field.ratio as usize;

        for x in 0..width {
            for y in 0..height {
                let &mut (ref mut field_force, ref mut potential) = &mut self.field.grid[y][x];

                let real_position =
                    Vector::new(x as f64 + 0.5, y as f64 + 0.5) / self.field.ratio as f64;

                for &(ref old_charge, ref cx, ref cy) in &self.updated_tiles {
                    let charge = &self.tiles[*cy as usize][*cx as usize];
                    let n_position = Vector::new(*cx as f64 + 0.5, *cy as f64 + 0.5);
                    let delta = real_position - n_position;

                    if *old_charge != 0 {
                        let (old_field, old_potential) = get_field(old_charge, &delta);
                        *potential -= old_potential;
                        *field_force -= old_field;
                    }
                    if *charge != 0 {
                        let (new_field, new_potential) = get_field(charge, &delta);
                        *potential += new_potential;
                        *field_force += new_field;
                    }
                }
            }
        }

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
                if angle.round() != old_angle.round() {
                    line.push(position);
                    old_angle = angle;
                }

                old_position = position;
                position += force.normalize() * charge / self.field.ratio as f64;
                y = position.y.floor() as i32;
                x = position.x.floor() as i32;

                length += 1;
            }

            line.push(old_position);
            lines.push(line);
        }

        lines
    }
}

fn get_field(charge: &i8, delta: &Vector) -> (Vector, f64) {
    use std::f64;

    if delta.norm() != 0.0 {
        (
            delta.normalize() * *charge as f64 / delta.norm_squared(),
            *charge as f64 / delta.norm(),
        )
    } else {
        (Vector::new(f64::MAX, f64::MAX), f64::MAX)
    }
}
