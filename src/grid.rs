use fnv::FnvHashMap;
use std::iter::{FromIterator, IntoIterator};

pub type XY = [i32; 2];
pub type WH = [i32; 2];

/// Sparse grid for storing things that do not necessarily take up
/// a 1x1 slot
#[derive(Debug)]
pub struct Grid<T> {
    /// Map from internal indexes to items, their minimal XY coordinates, and their sizes
    items: FnvHashMap<u64, (T, XY, WH)>,
    /// Map from XY coordinates to internal indexes
    grid: FnvHashMap<XY, u64>,
    next_idx: u64,
}

impl<T> Grid<T> {
    /// Creates an empty grid
    pub fn new() -> Self {
        Grid {
            items: FnvHashMap::default(),
            grid: FnvHashMap::default(),
            next_idx: 0,
        }
    }

    /// Gets the item at a specific position, along with that item's
    /// minimal XY coordinates and size. Width and height cannot be 0.
    pub fn get(&self, position: XY) -> Option<&(T, XY, WH)> {
        Some(&self.items[self.grid.get(&position)?])
    }

    /// Inserts an item at a specific minimal XY position with a specific size.
    /// Removes overlapping items.
    pub fn insert(&mut self, t: T, position: XY, size: WH) {
        let [x, y] = position;
        let [w, h] = size;

        for y in y..(y + h) {
            for x in x..(x + w) {
                self.remove([x, y]);
            }
        }

        let idx = self.next_idx;
        self.next_idx += 1;

        self.items.insert(idx, (t, position, size));

        for y in y..(y + h) {
            for x in x..(x + w) {
                self.grid.insert([x, y], idx);
            }
        }
    }

    /// Removes the item at a specific position
    pub fn remove(&mut self, position: XY) {
        if let Some(idx) = self.grid.get(&position) {
            let (t, [x, y], [w, h]) = self.items.remove(idx).unwrap();

            for y in y..(y + h) {
                for x in x..(x + w) {
                    self.grid.remove(&[x, y]);
                }
            }
        }
    }
}

impl<T> FromIterator<(T, XY, WH)> for Grid<T> {
    fn from_iter<I: IntoIterator<Item=(T, XY, WH)>>(i: I) -> Self {
        let mut g = Self::new();

        for (t, xy, wh) in i {
            g.insert(t, xy, wh);
        }

        g
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_from_iter() {
        let grid = [
            ("a", [0, 0], [1, 1]),
            ("b", [1, 0], [2, 1]),
        ].iter().cloned().collect::<Grid<_>>();

        assert_eq!(Some(("a", [0, 0], [1, 1])), grid.get([0, 0]).cloned());
        assert_eq!(Some(("b", [1, 0], [2, 1])), grid.get([1, 0]).cloned());
        assert_eq!(Some(("b", [1, 0], [2, 1])), grid.get([2, 0]).cloned());
    }

    #[test]
    fn test_insert_overlapping() {
        let mut grid = Grid::new();

        grid.insert("a", [0, 0], [2, 2]);
        grid.insert("b", [1, 1], [2, 2]);

        assert_eq!(None, grid.get([0, 0]));
    }
}