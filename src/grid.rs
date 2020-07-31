use cgmath::vec2;
use fnv::FnvHashMap;
use itertools::iproduct;
use std::iter::{FromIterator, IntoIterator};

use crate::math::{Vec2, Vec2i, Vector2Ex};

pub type XY = Vec2i;
pub type WH = (usize, usize);

pub trait GridItem {
    fn rotate_in_grid(self, turns: isize) -> Self
    where
        Self: Sized,
    {
        self
    }

    fn flip_x_in_grid(self) -> Self
    where
        Self: Sized,
    {
        self
    }

    fn flip_y_in_grid(self) -> Self
    where
        Self: Sized,
    {
        self
    }
}

/// Sparse grid for storing things that do not necessarily take up
/// a 1x1 slot
#[derive(Clone, Debug)]
pub struct Grid<T: GridItem> {
    /// Map from internal indexes to items, their minimal XY coordinates, and their sizes
    items: FnvHashMap<u64, (T, XY, WH)>,
    /// Map from XY coordinates to internal indexes
    grid: FnvHashMap<XY, u64>,
    next_idx: u64,
}

impl<T: GridItem> Default for Grid<T> {
    fn default() -> Self {
        Grid {
            items: FnvHashMap::default(),
            grid: FnvHashMap::default(),
            next_idx: 0,
        }
    }
}

impl<T: GridItem> Grid<T> {
    /// Creates an empty grid
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Gets the item at a specific position, along with that item's
    /// minimal XY coordinates and size. Width and height cannot be 0.
    pub fn get(&self, position: XY) -> Option<&(T, XY, WH)> {
        Some(&self.items[self.grid.get(&position)?])
    }

    pub fn get_f64(&self, position: Vec2) -> Option<&(T, XY, WH)> {
        self.get(vec2(
            position.x.floor() as isize,
            position.y.floor() as isize,
        ))
    }

    pub fn get_mut(&mut self, position: XY) -> Option<(&mut T, XY, WH)> {
        let (t, xy, wh) = self.items.get_mut(self.grid.get_mut(&position)?).unwrap();
        Some((t, *xy, *wh))
    }

    pub fn iter(&self) -> impl Iterator<Item = &(T, XY, WH)> {
        self.items.values()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (&mut T, XY, WH)> {
        self.items.values_mut().map(|(t, xy, wh)| (t, *xy, *wh))
    }

    pub fn get_in_bounds(
        &self,
        min_x: f64,
        max_x: f64,
        min_y: f64,
        max_y: f64,
    ) -> impl Iterator<Item = &(T, XY, WH)> {
        self.items.values().filter(move |(_t, xy, wh)| {
            let [x, y] = [xy.x as f64, xy.y as f64];
            let [w, h] = [wh.0 as f64, wh.1 as f64];

            x + w >= min_x && x <= max_x && y + h >= min_y && y <= max_y
        })
    }

    /// Gets the xy positions that are empty
    pub fn get_empty_in_bounds(&self, min_x: f64, max_x: f64, min_y: f64, max_y: f64) -> Vec<XY> {
        let min_x = min_x.floor() as isize;
        let max_x = max_x.ceil() as isize;
        let min_y = min_y.floor() as isize;
        let max_y = max_y.ceil() as isize;

        iproduct!(min_x..=max_x, min_y..=max_y)
            .map(|(x, y)| vec2(x, y))
            .filter(|xy| self.grid.get(&xy).is_none())
            .collect::<Vec<_>>()
    }

    /// Gets the item touching an edge centered at double_xy / 2, in the direction `direction`,
    /// and also gets the minimal xy coords, the size, and the index along the perimeter.
    pub fn get_item_touching_edge_mut(
        &mut self,
        double_xy: XY,
        direction: XY,
    ) -> Option<(&mut T, XY, WH, usize)> {
        let x_mis = double_xy.x.rem_euclid(2);
        let y_mis = double_xy.y.rem_euclid(2);

        debug_assert!(x_mis != y_mis, "Not on an edge!");

        let mut xy = vec2(double_xy.x.div_euclid(2), double_xy.y.div_euclid(2));
        if direction.x < 0 {
            xy.x -= 1;
        }
        if direction.y < 0 {
            xy.y -= 1;
        }

        if let Some((t, min_xy, (w, h))) = self.get_mut(xy) {
            if direction.x < 0 {
                xy.x += 1;
            }
            if direction.y < 0 {
                xy.y += 1;
            }

            Some((
                t,
                min_xy,
                (w, h),
                if x_mis != 0 {
                    if xy.y == min_xy.y {
                        // Bottom edge
                        xy.x - min_xy.x
                    } else {
                        // Top edge
                        (w + h + w) as isize - (xy.x - min_xy.x) - 1
                    }
                } else {
                    if xy.x == min_xy.x {
                        // Left edge
                        (w + h + w + h) as isize - (xy.y - min_xy.y) - 1
                    } else {
                        // Right edge
                        w as isize + (xy.y - min_xy.y)
                    }
                } as usize,
            ))
        } else {
            None
        }
    }

    pub fn extend(&mut self, iter: impl IntoIterator<Item = (T, XY, WH)>) {
        for (t, xy, wh) in iter.into_iter() {
            self.insert(t, xy, wh);
        }
    }

    /// Inserts an item at a specific minimal XY position with a specific size.
    /// Removes and returns overlapping items.
    pub fn insert(&mut self, t: T, position: XY, size: WH) -> Vec<(T, XY, WH)> {
        let (x, y) = (position.x, position.y);
        let (w, h) = size;

        let mut overlapping = vec![];

        for y in y..(y + h as isize) {
            for x in x..(x + w as isize) {
                overlapping.extend(self.remove(vec2(x, y)));
            }
        }

        let idx = self.next_idx;
        self.next_idx += 1;

        self.items.insert(idx, (t, position, size));

        for y in y..(y + h as isize) {
            for x in x..(x + w as isize) {
                self.grid.insert(vec2(x, y), idx);
            }
        }

        overlapping
    }

    /// Removes and returns the item at a specific position
    pub fn remove(&mut self, position: XY) -> Option<(T, XY, WH)> {
        if let Some(idx) = self.grid.get(&position) {
            let (t, xy, (w, h)) = self.items.remove(idx).unwrap();

            for y in xy.y..(xy.y + h as isize) {
                for x in xy.x..(xy.x + w as isize) {
                    self.grid.remove(&vec2(x, y));
                }
            }

            Some((t, xy, (w, h)))
        } else {
            None
        }
    }

    /// Moves the grid by some vector
    pub fn translate(self, vec: XY) -> Self {
        self.into_iter()
            .map(|(t, xy, wh)| (t, xy + vec, wh))
            .collect()
    }

    /// Center the bounding box of this grid at the origin
    pub fn center(self) -> Self {
        let first = self
            .iter()
            .next()
            .map(|(_, xy, wh)| (xy.x, xy.x + wh.0 as isize, xy.y, xy.y + wh.1 as isize))
            .unwrap_or((0, 0, 0, 0));

        let (min_x, max_x, min_y, max_y) = self
            .iter()
            .map(|(_, xy, wh)| (xy.x, xy.x + wh.0 as isize, xy.y, xy.y + wh.1 as isize))
            .fold(first, |(l0, r0, b0, t0), (l1, r1, b1, t1)| {
                (l0.min(l1), r0.max(r1), b0.min(b1), t0.max(t1))
            });

        let vec = vec2(
            -(min_x + max_x).div_euclid(2),
            -(min_y + max_y).div_euclid(2),
        );

        self.translate(vec)
    }

    /// Rotates the grid around some point by some number of counterclockwise right turns.
    /// The rotation will effectively be around some half-integer coordinates
    pub fn rotate(self, center: Vec2, turns: isize) -> Self {
        let turns = turns.rem_euclid(4);
        let center = vec2(center.x.floor() as isize, center.y.floor() as isize);

        self.into_iter()
            .map(|(t, mut xy, mut wh)| {
                for _ in 0..turns {
                    xy = (xy - center).right_ccw() + center;
                    wh = (wh.1, wh.0);
                    xy.x = xy.x + 1 - wh.0 as isize;
                }

                (t.rotate_in_grid(turns), xy, wh)
            })
            .collect()
    }

    /// Flips the x coordinates in the grid around some x position.
    /// The axis will effecively be at some half-integer coordinate.
    pub fn flip_x(self, x: f64) -> Self {
        self.into_iter()
            .map(|(t, mut xy, (w, h))| {
                xy.x = 2 * x.floor() as isize + 1 - xy.x - w as isize;
                (t.flip_x_in_grid(), xy, (w, h))
            })
            .collect()
    }

    /// Flips the y coordinates in the grid around some y position.
    /// The axis will effecively be at some half-integer coordinate.
    pub fn flip_y(self, y: f64) -> Self {
        self.into_iter()
            .map(|(t, mut xy, (w, h))| {
                xy.y = 2 * y.floor() as isize + 1 - xy.y - h as isize;
                (t.flip_y_in_grid(), xy, (w, h))
            })
            .collect()
    }
}

impl<T: GridItem> FromIterator<(T, XY, WH)> for Grid<T> {
    fn from_iter<I: IntoIterator<Item = (T, XY, WH)>>(i: I) -> Self {
        let mut g = Self::new();

        for (t, xy, wh) in i {
            g.insert(t, xy, wh);
        }

        g
    }
}

impl<T: GridItem> IntoIterator for Grid<T> {
    type Item = (T, XY, WH);
    // Oof, long type name
    type IntoIter = std::iter::Map<
        <FnvHashMap<u64, (T, XY, WH)> as IntoIterator>::IntoIter,
        fn((u64, (T, XY, WH))) -> (T, XY, WH),
    >;

    fn into_iter(self) -> Self::IntoIter {
        fn value<K, V>(tup: (K, V)) -> V {
            tup.1
        }

        self.items.into_iter().map(value)
    }
}

// To make the test cases work
impl<'a> GridItem for &'a str {}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_from_iter() {
        let grid = [("a", vec2(0, 0), (1, 1)), ("b", vec2(1, 0), (2, 1))]
            .iter()
            .cloned()
            .collect::<Grid<_>>();

        assert_eq!(
            Some(("a", vec2(0, 0), (1, 1))),
            grid.get(vec2(0, 0)).cloned()
        );
        assert_eq!(
            Some(("b", vec2(1, 0), (2, 1))),
            grid.get(vec2(1, 0)).cloned()
        );
        assert_eq!(
            Some(("b", vec2(1, 0), (2, 1))),
            grid.get(vec2(2, 0)).cloned()
        );
    }

    #[test]
    fn test_insert_overlapping() {
        let mut grid = Grid::new();

        grid.insert("a", vec2(0, 0), (2, 2));
        grid.insert("b", vec2(1, 1), (2, 2));

        assert_eq!(None, grid.get(vec2(0, 0)));
    }

    #[test]
    fn test_translate() {
        let mut grid = Grid::new();
        grid.insert("a", vec2(-1, 2), (3, 2));

        let grid = grid.translate(vec2(-1, 5));

        assert_eq!(("a", vec2(-2, 7), (3, 2)), *grid.iter().next().unwrap());
    }

    #[test]
    fn test_rotate_ccw() {
        let mut grid = Grid::new();
        grid.insert("a", vec2(-1, 2), (3, 2));

        let grid = grid.rotate(vec2(-0.5, 2.5), 1);

        assert_eq!(("a", vec2(-2, 2), (2, 3)), *grid.iter().next().unwrap());
    }

    #[test]
    fn test_rotate_upside_down() {
        let mut grid = Grid::new();
        grid.insert("a", vec2(-1, 2), (3, 2));

        let grid = grid.rotate(vec2(1.5, -1.5), 2);

        assert_eq!(("a", vec2(1, -7), (3, 2)), *grid.iter().next().unwrap());
    }

    #[test]
    fn test_rotate_cw() {
        let mut grid = Grid::new();
        grid.insert("a", vec2(-1, 2), (3, 2));

        let grid = grid.rotate(vec2(-0.5, 2.5), -1);

        assert_eq!(("a", vec2(-1, 0), (2, 3)), *grid.iter().next().unwrap());
    }

    #[test]
    fn test_flip_x() {
        let mut grid = Grid::new();
        grid.insert("a", vec2(-1, 2), (3, 2));

        let grid = grid.flip_x(1.5);

        assert_eq!(("a", vec2(1, 2), (3, 2)), *grid.iter().next().unwrap());
    }

    #[test]
    fn test_flip_y() {
        let mut grid = Grid::new();
        grid.insert("a", vec2(-1, 2), (3, 2));

        let grid = grid.flip_y(-0.5);

        assert_eq!(("a", vec2(-1, -5), (3, 2)), *grid.iter().next().unwrap());
    }
}
