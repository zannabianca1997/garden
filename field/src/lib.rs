use std::{
    f64,
    ops::{Add, Mul},
};

use grid::Grid;
use nalgebra::{
    matrix, point, vector, Matrix2, Matrix2x3, Matrix3, Point2, Point3, Vector2, Vector3,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TrigType {
    Upper,
    Lower,
}

/// 2-D tiled smoothed field
#[derive(Debug, Clone)]
pub struct Field<T> {
    data: Grid<T>,

    from_square_coords: Matrix2<f64>,
    to_square_coords: Matrix2<f64>,

    lower_barycentric_coords_gradient: Matrix2x3<f64>,
    upper_barycentric_coords_gradient: Matrix2x3<f64>,

    res: f64,
    tile_x: f64,
    tile_y: f64,
}

impl<T> Field<T> {
    /// Returns:
    /// - The indices of the vertices of the containing triangle
    /// - The barycentric coordinates of the point inside the triangle
    /// - The gradient of the coordinates in respect of the position
    fn trig_data(&self, pos: Point2<f64>) -> ([(isize, isize); 3], Point3<f64>, TrigType) {
        // move to square coordinate
        let pos = self.to_square_coords * pos;
        trig_data_from_square_coords(pos)
    }

    /// Iter through all triangles
    fn iter_trigs(&self) -> impl Iterator<Item = ([(isize, isize); 3], TrigType)> {
        let rows = self.data.rows() as isize;
        let cols = self.data.cols() as isize;
        (0..rows).flat_map(move |row| {
            (0..cols).flat_map(move |col| {
                [
                    (
                        [(col, row + 1), (col + 1, row), (col, row)],
                        TrigType::Lower,
                    ),
                    (
                        [(col, row + 1), (col + 1, row + 1), (col + 1, row)],
                        TrigType::Upper,
                    ),
                ]
            })
        })
    }

    fn vertex(&self, (col, row): (isize, isize)) -> (Vector2<f64>, &T) {
        let d_col = (col
            + row.div_euclid(self.data.rows() as isize) * (self.data.rows() as isize / 2))
            .rem_euclid(self.data.cols() as isize) as usize;
        let d_row = row.rem_euclid(self.data.rows() as isize) as usize;
        (
            self.from_square_coords * vector![col as f64, row as f64],
            &self.data[(d_row, d_col)],
        )
    }

    pub fn map_with_coords<U>(self, f: impl Fn(Point2<f64>, T) -> U) -> Field<U> {
        let Field {
            data,
            from_square_coords,
            to_square_coords,
            lower_barycentric_coords_gradient,
            upper_barycentric_coords_gradient,
            res,
            tile_x,
            tile_y,
        } = self;

        let (rows, cols) = data.size();

        let old_data = data.into_vec();
        let mut data = Vec::with_capacity(old_data.len());

        let mut old_data = old_data.into_iter();

        for row in 0..rows {
            for col in 0..cols {
                let pos = self.from_square_coords * point![col as f64, row as f64];
                data.push(f(pos, old_data.next().unwrap()))
            }
        }

        let data = Grid::from_vec(data, cols);

        Field {
            data,
            from_square_coords,
            to_square_coords,
            lower_barycentric_coords_gradient,
            upper_barycentric_coords_gradient,
            res,
            tile_x,
            tile_y,
        }
    }

    pub fn map<U>(self, f: impl Fn(T) -> U) -> Field<U> {
        let Field {
            data,
            from_square_coords,
            to_square_coords,
            lower_barycentric_coords_gradient,
            upper_barycentric_coords_gradient,
            res,
            tile_x,
            tile_y,
        } = self;

        let cols = data.cols();
        let data = Grid::from_vec(data.into_vec().into_iter().map(f).collect(), cols);

        Field {
            data,
            from_square_coords,
            to_square_coords,
            lower_barycentric_coords_gradient,
            upper_barycentric_coords_gradient,
            res,
            tile_x,
            tile_y,
        }
    }

    pub fn new_from_fun(
        tile_x: f64,
        tile_y: f64,
        res: f64,
        fun: impl Fn(Point2<f64>) -> T,
    ) -> Self {
        Field::new(tile_x, tile_y, res).map_with_coords(|coord, ()| fun(coord))
    }

    pub fn min_by(&self, cmp: fn(&T, &T) -> std::cmp::Ordering) -> &T {
        self.data.iter().min_by(|a, b| cmp(a, b)).unwrap()
    }
    pub fn max_by(&self, cmp: fn(&T, &T) -> std::cmp::Ordering) -> &T {
        self.data.iter().max_by(|a, b| cmp(a, b)).unwrap()
    }

    fn trig_coord_grads(&self, trig_type: TrigType) -> &Matrix2x3<f64> {
        match trig_type {
            TrigType::Upper => &self.upper_barycentric_coords_gradient,
            TrigType::Lower => &self.lower_barycentric_coords_gradient,
        }
    }

    pub fn res(&self) -> f64 {
        self.res
    }

    pub fn tile_x(&self) -> f64 {
        self.tile_x
    }

    pub fn tile_y(&self) -> f64 {
        self.tile_y
    }
}

fn trig_data_from_square_coords(pos: Point2<f64>) -> ([(isize, isize); 3], Point3<f64>, TrigType) {
    let (col, u) = (pos.x.div_euclid(1.) as isize, pos.x.rem_euclid(1.));
    let (row, v) = (pos.y.div_euclid(1.) as isize, pos.y.rem_euclid(1.));

    if u + v < 1. {
        (
            [(col, row + 1), (col + 1, row), (col, row)],
            point![v, u, 1. - v - u],
            TrigType::Lower,
        )
    } else {
        (
            [(col, row + 1), (col + 1, row + 1), (col + 1, row)],
            point![1. - u, v + u - 1., 1. - v],
            TrigType::Upper,
        )
    }
}

impl<T> Field<T>
where
    T: Ord,
{
    pub fn min(&self) -> &T {
        self.min_by(T::cmp)
    }
    pub fn max(&self) -> &T {
        self.max_by(T::cmp)
    }
}

impl<T> Field<T>
where
    T: Default,
{
    pub fn new(tile_x: f64, tile_y: f64, res: f64) -> Self {
        let cols = (tile_x / res) as usize;
        // rows is kept even to ensure square tiling
        let rows = (tile_y * (1. / 3f64.sqrt()) / res) as usize * 2;

        let d_x = tile_x / cols as f64;
        let d_y = tile_y * (2. / 3f64.sqrt()) / rows as f64;

        let from_square_coords = matrix![d_x,0.;0.,d_y] * matrix![1.,0.5;0.,3f64.sqrt()/2.];
        let to_square_coords = from_square_coords.try_inverse().unwrap();

        /*
            Here we precalculate the gradient of the baricentric coordinates.
            It transforms as the transpose of the inverse of the basis transform.
        */
        //vector![v, u, 1. - v - u]
        let lower_barycentric_coords_gradient = to_square_coords.transpose()
            * matrix![
                0.,1.,-1.;
                1.,0.,-1.
            ];
        // vector![1. - u, v + u - 1., 1. - v]
        let upper_barycentric_coords_gradient = to_square_coords.transpose()
            * matrix![
                -1.,1.,0.;
                0.,1.,-1.
            ];

        Self {
            data: Grid::new(rows, cols),

            from_square_coords,
            to_square_coords,

            lower_barycentric_coords_gradient,
            upper_barycentric_coords_gradient,

            res,
            tile_x,
            tile_y,
        }
    }
}

impl<T> Field<T>
where
    T: Clone,
{
    pub fn new_filled(tile_x: f64, tile_y: f64, res: f64, value: T) -> Self {
        Field::new(tile_x, tile_y, res).map(|()| value.clone())
    }
}

impl<T> Field<T>
where
    T: Add<T, Output = T> + Mul<f64, Output = T> + Clone,
{
    /// Calculate gradient of a given triangle from the vertex indices and the precalculated coordinate gradients
    /// This *might* use a cache
    fn trig_gradient(&self, idxs: [(isize, isize); 3], trig_type: TrigType) -> Vector2<T> {
        let [g_x, g_y] = idxs
            .into_iter()
            .zip(self.trig_coord_grads(trig_type).column_iter())
            .map(|(v, g)| {
                let v = self.vertex(v).1;
                [v.clone() * g.x, v.clone() * g.y]
            })
            .reduce(|[a_x, a_y], [b_x, b_y]| [a_x + b_x, a_y + b_y])
            .unwrap();
        vector![g_x, g_y]
    }

    pub fn value(&self, pos: Point2<f64>) -> T {
        let (idxs, coords, _) = self.trig_data(pos);
        idxs.into_iter()
            .zip(coords.iter())
            .map(|(v, c)| self.vertex(v).1.clone() * *c)
            .reduce(Add::add)
            .unwrap()
    }

    pub fn gradient(&self, pos: Point2<f64>) -> Vector2<T> {
        let (idxs, _, trig_type) = self.trig_data(pos);
        self.trig_gradient(idxs, trig_type)
    }
}

impl Field<f64> {
    pub fn normal(&self, pos: Point2<f64>) -> Vector3<f64> {
        let gradient = self.gradient(pos);
        vector![-gradient.x, -gradient.y, 1.].normalize()
    }

    /// Calculate the max gradient norm
    pub fn max_gradient(&self) -> f64 {
        self.iter_trigs()
            .map(|(idxs, trig_type)| self.trig_gradient(idxs, trig_type).norm_squared())
            .max_by(f64::total_cmp)
            .unwrap()
            .sqrt()
    }

    /// Precalculate values for raycasting
    pub fn raycaster(&self, RaycasterOptions { epsilon, max_dist }: RaycasterOptions) -> Raycaster {
        Raycaster {
            max_heigth: *self.max_by(f64::total_cmp),
            min_heigth: *self.min_by(f64::total_cmp),
            max_gradient: self.max_gradient(),
            field: self,
            epsilon,
            max_dist,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct RaycasterOptions {
    pub epsilon: f64,
    pub max_dist: f64,
}

impl Default for RaycasterOptions {
    fn default() -> Self {
        Self {
            epsilon: 1e-5,
            max_dist: 1000.,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Raycaster<'f> {
    field: &'f Field<f64>,

    max_heigth: f64,
    min_heigth: f64,
    max_gradient: f64,

    epsilon: f64,
    max_dist: f64,
}

impl Raycaster<'_> {
    pub fn cast(&self, pos: Point3<f64>, dir: Vector3<f64>) -> Option<Point3<f64>> {
        // calculating where the ray will exit the surface semiplanes
        let i_min = (self.min_heigth - pos.z) / dir.z;
        let i_max = (self.max_heigth - pos.z) / dir.z;

        if i_min <= 0. && i_max <= 0. {
            // ray cannot intersect the surface
            return None;
        }

        // Calculating the entering and exiting cells
        let mut advanced = f64::min(self.epsilon, f64::min(i_min, i_max));
        let end = f64::min(f64::max(i_min, i_max), self.max_dist);

        // Opening of the max gradient cone
        let cone_opening = 1. / (dir.z.abs() + self.max_gradient * dir.xy().norm());

        while advanced < end {
            let current_pos = pos + dir * advanced;

            // checks if we hit the triangle under us. If not, return the height of the terrain under us

            let (trig, coords, _) = self.field.trig_data(current_pos.xy());
            let vertices = trig.map(|idx| {
                let (o, h) = self.field.vertex(idx);
                point![o.x, o.y, *h]
            });

            // Find the plane/line intersection

            if let Some(intersection) =
                Matrix3::from_columns(&[-dir, vertices[1] - vertices[0], vertices[2] - vertices[0]])
                    .lu()
                    .solve(&(pos - vertices[0]))
            {
                // intersection exist
                let t = intersection.x;
                let u = intersection.y;
                let v = intersection.z;

                // check if the intersection is inside the triangle AND in the positive semi-ray
                if t > 0. && u >= 0. && v >= 0. && u + v <= 1. {
                    return Some(pos + t * dir);
                }
            } else {
                // no intersection, ray is coplanar
            }

            let t_height =
                vertices[0].z * coords.x + vertices[1].z * coords.y + vertices[2].z * coords.z;

            // calculate how much can the ray advance with no repercussion

            // First: cone of the noise
            let mut delta = cone_opening * (t_height - current_pos.z).abs();

            // Second: prism
            for (v_1, v_2) in [
                (vertices[0], vertices[1]),
                (vertices[1], vertices[2]),
                (vertices[2], vertices[0]),
            ] {
                let [v_1, v_2] = [v_1, v_2].map(|p| p.xy());

                if let Some(intersection) = Matrix2::from_columns(&[-dir.xy(), v_1 - v_2])
                    .lu()
                    .solve(&(current_pos.xy() - v_2))
                {
                    // intersection exist
                    let t = intersection.x;
                    let u = intersection.y;

                    // Is the intersection in the side of the triangle?
                    if t > 0. && u >= 0. && u <= 1. {
                        delta = delta.max(t)
                    }
                } else {
                    // no intersection, ray is parallel
                }
            }

            advanced += delta.max(self.epsilon);
        }

        None
    }
}
