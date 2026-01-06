use crate::simulator::math::Vector2D;
use std::collections::HashMap;

#[derive(Debug, PartialEq, Hash, Eq)]
pub struct GridCoord {
    x: i32,
    y: i32
}

impl GridCoord {
    pub fn new(x: i32, y: i32) -> GridCoord {
        GridCoord {x, y}
    }
}

pub struct SpatialGrid {
    cell_size: i32,
    pub planes: HashMap<GridCoord, Vec<String>>
}

impl SpatialGrid {
    pub fn new(cell_size: i32) -> SpatialGrid {
        SpatialGrid { cell_size, planes: HashMap::new(), }
    }

    pub fn to_grid_coord(&self, coords: Vector2D) -> GridCoord {
        let x = (coords.x / self.cell_size as f64).floor() as i32;
        let y = (coords.y / self.cell_size as f64).floor() as i32;
        GridCoord::new(x, y)
    }

    pub fn insert(&mut self, id: String, position: Vector2D) {
        let key = self.to_grid_coord(position);
        self.planes.entry(key).and_modify(|v| v.push(id.clone()))
            .or_insert(vec![id]);
    }

    pub fn clear(&mut self) {
        self.planes.clear();
    }

    pub fn get_nearby_ids(&self, exclude_id: &str, position: Vector2D) -> impl Iterator<Item=&String> {
        let center = self.to_grid_coord(position);
        (-1..=1).flat_map(move |dx| {
            (-1..=1).map(move |dy| {
                GridCoord::new(center.x + dx, center.y + dy)
            })
        })
        .flat_map(|coord| self.planes.get(&coord))
        .flatten()
        .filter(move |id| *id != exclude_id)
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grid_coords_conversion() {
        let cell_size = 8;
        let grid = SpatialGrid::new(cell_size);

        assert_eq!(GridCoord::new(0, 0), grid.to_grid_coord(Vector2D::new(5.0, 7.0)));
        assert_eq!(GridCoord::new(-1, 0), grid.to_grid_coord(Vector2D::new(-5.0, 7.0)));
        assert_eq!(GridCoord::new(0, -1), grid.to_grid_coord(Vector2D::new(5.0, -7.0)));
        assert_eq!(GridCoord::new(-1, -1), grid.to_grid_coord(Vector2D::new(-5.0, -7.0)));
    }

    #[test]
    fn test_insert() {
        let cell_size = 8;
        let mut grid = SpatialGrid::new(cell_size);

        grid.insert("P1".to_string(), Vector2D::new(5.0, 7.0));
        grid.insert("P2".to_string(), Vector2D::new(5.0, 7.0));
        grid.insert("P3".to_string(), Vector2D::new(-9.0, -9.0));

        assert_eq!(2, grid.planes.get(&GridCoord::new(0, 0)).map(|v| v.len()).unwrap_or(100));
        assert_eq!(1, grid.planes.get(&GridCoord::new(-2, -2)).map(|v| v.len()).unwrap_or(100));
    }

    #[test]
    fn test_get_neighbor_ids() {
        let cell_size = 8;
        let mut grid = SpatialGrid::new(cell_size);

        grid.insert("P1".to_string(), Vector2D::new(5.0, 7.0));
        grid.insert("P2".to_string(), Vector2D::new(5.0, 7.0));
        grid.insert("P3".to_string(), Vector2D::new(-9.0, -9.0));
        grid.insert("P4".to_string(), Vector2D::new(9.0, 9.0));

        assert_eq!(
            vec!["P2", "P4"],
            grid.get_nearby_ids("P1", Vector2D::new(1.0, 1.0)).collect::<Vec<&String>>()
        );

    }

}