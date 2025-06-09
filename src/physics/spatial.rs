// Spatial Hashing - Efficient broad-phase collision detection
use super::math::*;
use std::collections::HashMap;

/// Types of objects that can be stored in spatial hash
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum SpatialObject {
    RigidBody(usize),
    SoftBodyParticle(usize, usize), // (soft_body_index, particle_index)
    FluidParticle(usize),
    Constraint(usize),
    Entity(usize), // ECS entity index
}

/// Spatial hash grid for efficient neighbor queries
#[derive(Debug, Clone)]
pub struct SpatialHash {
    pub cell_size: f64,
    pub grid: HashMap<(i32, i32, i32), Vec<(SpatialObject, AABB)>>,
    pub inv_cell_size: f64,
}

impl SpatialHash {
    pub fn new(cell_size: f64) -> Self {
        Self {
            cell_size,
            grid: HashMap::new(),
            inv_cell_size: 1.0 / cell_size,
        }
    }

    /// Clear all objects from the spatial hash
    pub fn clear(&mut self) {
        self.grid.clear();
    }

    /// Get the grid cell coordinates for a point
    fn get_cell(&self, point: Vec3) -> (i32, i32, i32) {
        (
            (point.x * self.inv_cell_size).floor() as i32,
            (point.y * self.inv_cell_size).floor() as i32,
            (point.z * self.inv_cell_size).floor() as i32,
        )
    }

    /// Get all grid cells that an AABB overlaps
    fn get_overlapping_cells(&self, aabb: AABB) -> Vec<(i32, i32, i32)> {
        let min_cell = self.get_cell(aabb.min);
        let max_cell = self.get_cell(aabb.max);

        let mut cells = Vec::new();

        for x in min_cell.0..=max_cell.0 {
            for y in min_cell.1..=max_cell.1 {
                for z in min_cell.2..=max_cell.2 {
                    cells.push((x, y, z));
                }
            }
        }

        cells
    }

    /// Insert an object into the spatial hash
    pub fn insert(&mut self, object: SpatialObject, aabb: AABB) {
        let cells = self.get_overlapping_cells(aabb);

        for cell in cells {
            self.grid
                .entry(cell)
                .or_insert_with(Vec::new)
                .push((object, aabb));
        }
    }

    /// Query objects that potentially overlap with the given AABB
    pub fn query(&self, query_aabb: AABB) -> Vec<SpatialObject> {
        let cells = self.get_overlapping_cells(query_aabb);
        let mut results = Vec::new();
        let mut seen = std::collections::HashSet::new();

        for cell in cells {
            if let Some(objects) = self.grid.get(&cell) {
                for (object, aabb) in objects {
                    // Avoid duplicates and check for actual AABB overlap
                    if !seen.contains(object) && query_aabb.intersects(*aabb) {
                        results.push(*object);
                        seen.insert(*object);
                    }
                }
            }
        }

        results
    }

    /// Query objects within a sphere
    pub fn query_sphere(&self, center: Vec3, radius: f64) -> Vec<SpatialObject> {
        let aabb = AABB::from_point(center, radius);
        let candidates = self.query(aabb);
        let mut results = Vec::new();

        for object in candidates {
            // For sphere queries, we need to check actual distance
            // This is a simplified version - in practice you'd store object positions
            results.push(object);
        }

        results
    }

    /// Get statistics about the spatial hash
    pub fn stats(&self) -> SpatialHashStats {
        let total_cells = self.grid.len();
        let total_objects = self.grid.values().map(|v| v.len()).sum();
        let max_objects_per_cell = self.grid.values().map(|v| v.len()).max().unwrap_or(0);
        let avg_objects_per_cell = if total_cells > 0 {
            total_objects as f64 / total_cells as f64
        } else {
            0.0
        };

        SpatialHashStats {
            total_cells,
            total_objects,
            max_objects_per_cell,
            avg_objects_per_cell,
            cell_size: self.cell_size,
        }
    }

    /// Resize the spatial hash based on current object distribution
    pub fn optimize(&mut self, objects: &[(SpatialObject, AABB)]) {
        if objects.is_empty() {
            return;
        }

        // Calculate optimal cell size based on average object size
        let avg_size = objects
            .iter()
            .map(|(_, aabb)| (aabb.max - aabb.min).magnitude())
            .sum::<f64>()
            / objects.len() as f64;

        let optimal_cell_size = avg_size * 2.0; // Heuristic: 2x average object size

        if (optimal_cell_size - self.cell_size).abs() > self.cell_size * 0.5 {
            // Significant change, rebuild with new cell size
            self.cell_size = optimal_cell_size;
            self.inv_cell_size = 1.0 / optimal_cell_size;
            self.clear();

            for (object, aabb) in objects {
                self.insert(*object, *aabb);
            }
        }
    }

    /// Get potential collision pairs from objects in the same cells
    pub fn get_potential_pairs(&self) -> Vec<(SpatialObject, SpatialObject)> {
        let mut pairs = Vec::new();

        for objects in self.grid.values() {
            // Check all pairs within each cell
            for i in 0..objects.len() {
                for j in i + 1..objects.len() {
                    let (obj_a, aabb_a) = objects[i];
                    let (obj_b, aabb_b) = objects[j];

                    // Basic AABB intersection test
                    if aabb_a.intersects(aabb_b) {
                        // Ensure consistent ordering for pairs
                        let pair = if obj_a < obj_b {
                            (obj_a, obj_b)
                        } else {
                            (obj_b, obj_a)
                        };
                        pairs.push(pair);
                    }
                }
            }
        }

        // Remove duplicates (objects can be in multiple cells)
        pairs.sort_unstable();
        pairs.dedup();

        pairs
    }
}

#[derive(Debug, Clone)]
pub struct SpatialHashStats {
    pub total_cells: usize,
    pub total_objects: usize,
    pub max_objects_per_cell: usize,
    pub avg_objects_per_cell: f64,
    pub cell_size: f64,
}

/// Hierarchical spatial hash for multi-scale queries
#[derive(Debug, Clone)]
pub struct HierarchicalSpatialHash {
    pub levels: Vec<SpatialHash>,
    pub base_cell_size: f64,
}

impl HierarchicalSpatialHash {
    pub fn new(base_cell_size: f64, num_levels: usize) -> Self {
        let mut levels = Vec::new();

        for i in 0..num_levels {
            let cell_size = base_cell_size * (2.0_f64).powi(i as i32);
            levels.push(SpatialHash::new(cell_size));
        }

        Self {
            levels,
            base_cell_size,
        }
    }

    /// Clear all levels
    pub fn clear(&mut self) {
        for level in &mut self.levels {
            level.clear();
        }
    }

    /// Insert object into appropriate levels based on size
    pub fn insert(&mut self, object: SpatialObject, aabb: AABB) {
        let object_size = (aabb.max - aabb.min).magnitude();

        // Insert into levels where cell size is appropriate for object size
        for level in &mut self.levels {
            if level.cell_size >= object_size * 0.5 && level.cell_size <= object_size * 4.0 {
                level.insert(object, aabb);
            }
        }
    }

    /// Query using the most appropriate level
    pub fn query(&self, query_aabb: AABB) -> Vec<SpatialObject> {
        let query_size = (query_aabb.max - query_aabb.min).magnitude();

        // Find the best level for this query size
        let mut best_level = 0;
        let mut best_ratio = f64::MAX;

        for (i, level) in self.levels.iter().enumerate() {
            let ratio = (level.cell_size / query_size - 1.0).abs();
            if ratio < best_ratio {
                best_ratio = ratio;
                best_level = i;
            }
        }

        self.levels[best_level].query(query_aabb)
    }
}

/// Broad-phase collision detection using spatial hashing
#[derive(Debug, Clone)]
pub struct BroadPhase {
    pub spatial_hash: SpatialHash,
    pub collision_pairs: Vec<(SpatialObject, SpatialObject)>,
}

impl BroadPhase {
    pub fn new(cell_size: f64) -> Self {
        Self {
            spatial_hash: SpatialHash::new(cell_size),
            collision_pairs: Vec::new(),
        }
    }

    /// Update the broad-phase with new object positions
    pub fn update(&mut self, objects: &[(SpatialObject, AABB)]) {
        self.spatial_hash.clear();
        self.collision_pairs.clear();

        // Insert all objects
        for (object, aabb) in objects {
            self.spatial_hash.insert(*object, *aabb);
        }

        // Find potential collision pairs
        self.find_collision_pairs(objects);
    }

    fn find_collision_pairs(&mut self, objects: &[(SpatialObject, AABB)]) {
        let mut checked_pairs = std::collections::HashSet::new();

        for (object_a, aabb_a) in objects {
            let candidates = self.spatial_hash.query(*aabb_a);

            for object_b in candidates {
                if object_a == &object_b {
                    continue;
                }

                // Create a consistent pair ordering to avoid duplicates
                let pair = if object_a < &object_b {
                    (*object_a, object_b)
                } else {
                    (object_b, *object_a)
                };

                if !checked_pairs.contains(&pair) {
                    self.collision_pairs.push(pair);
                    checked_pairs.insert(pair);
                }
            }
        }
    }

    /// Get all potential collision pairs
    pub fn get_collision_pairs(&self) -> &[(SpatialObject, SpatialObject)] {
        &self.collision_pairs
    }

    /// Query for objects near a point
    pub fn query_point(&self, point: Vec3, radius: f64) -> Vec<SpatialObject> {
        let query_aabb = AABB::from_point(point, radius);
        self.spatial_hash.query(query_aabb)
    }

    /// Ray casting through the spatial hash
    pub fn raycast(
        &self,
        ray_origin: Vec3,
        ray_direction: Vec3,
        max_distance: f64,
    ) -> Vec<SpatialObject> {
        let ray_end = ray_origin + ray_direction.normalized() * max_distance;

        // Create AABB that encompasses the ray
        let min = ray_origin.min_component_wise(ray_end);
        let max = ray_origin.max_component_wise(ray_end);
        let ray_aabb = AABB::new(min, max);

        self.spatial_hash.query(ray_aabb)
    }
}

/// Sweep and prune algorithm for 1D spatial sorting
#[derive(Debug, Clone)]
pub struct SweepAndPrune {
    pub objects: Vec<(SpatialObject, f64, f64)>, // (object, min, max)
    pub axis: usize,                             // 0=x, 1=y, 2=z
}

impl SweepAndPrune {
    pub fn new(axis: usize) -> Self {
        Self {
            objects: Vec::new(),
            axis,
        }
    }

    /// Update with new object positions
    pub fn update(&mut self, objects: &[(SpatialObject, AABB)]) {
        self.objects.clear();

        for (object, aabb) in objects {
            let (min, max) = match self.axis {
                0 => (aabb.min.x, aabb.max.x),
                1 => (aabb.min.y, aabb.max.y),
                2 => (aabb.min.z, aabb.max.z),
                _ => (aabb.min.x, aabb.max.x),
            };

            self.objects.push((*object, min, max));
        }

        // Sort by minimum value along the axis
        self.objects.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
    }

    /// Find overlapping pairs along this axis
    pub fn find_overlaps(&self) -> Vec<(SpatialObject, SpatialObject)> {
        let mut pairs = Vec::new();

        for i in 0..self.objects.len() {
            let (obj_a, min_a, max_a) = self.objects[i];

            for j in (i + 1)..self.objects.len() {
                let (obj_b, min_b, _max_b) = self.objects[j];

                // If min_b > max_a, no more overlaps for obj_a
                if min_b > max_a {
                    break;
                }

                pairs.push((obj_a, obj_b));
            }
        }

        pairs
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spatial_hash_creation() {
        let spatial_hash = SpatialHash::new(1.0);

        assert_eq!(spatial_hash.cell_size, 1.0);
        assert_eq!(spatial_hash.inv_cell_size, 1.0);
        assert!(spatial_hash.grid.is_empty());
    }

    #[test]
    fn test_get_cell() {
        let spatial_hash = SpatialHash::new(1.0);

        assert_eq!(spatial_hash.get_cell(Vec3::new(0.5, 0.5, 0.5)), (0, 0, 0));
        assert_eq!(spatial_hash.get_cell(Vec3::new(1.5, 1.5, 1.5)), (1, 1, 1));
        assert_eq!(
            spatial_hash.get_cell(Vec3::new(-0.5, -0.5, -0.5)),
            (-1, -1, -1)
        );
    }

    #[test]
    fn test_insert_and_query() {
        let mut spatial_hash = SpatialHash::new(1.0);

        let obj1 = SpatialObject::RigidBody(0);
        let aabb1 = AABB::new(Vec3::new(0.0, 0.0, 0.0), Vec3::new(0.5, 0.5, 0.5));

        let obj2 = SpatialObject::RigidBody(1);
        let aabb2 = AABB::new(Vec3::new(1.0, 1.0, 1.0), Vec3::new(1.5, 1.5, 1.5));

        spatial_hash.insert(obj1, aabb1);
        spatial_hash.insert(obj2, aabb2);

        // Query overlapping with obj1
        let query_aabb = AABB::new(Vec3::new(-0.1, -0.1, -0.1), Vec3::new(0.6, 0.6, 0.6));
        let results = spatial_hash.query(query_aabb);

        assert!(results.contains(&obj1));
        assert!(!results.contains(&obj2));
    }

    #[test]
    fn test_overlapping_cells() {
        let spatial_hash = SpatialHash::new(1.0);

        let aabb = AABB::new(Vec3::new(0.5, 0.5, 0.5), Vec3::new(1.5, 1.5, 1.5));
        let cells = spatial_hash.get_overlapping_cells(aabb);

        // Should overlap 8 cells (2x2x2)
        assert_eq!(cells.len(), 8);
        assert!(cells.contains(&(0, 0, 0)));
        assert!(cells.contains(&(1, 1, 1)));
    }

    #[test]
    fn test_spatial_hash_clear() {
        let mut spatial_hash = SpatialHash::new(1.0);

        let obj = SpatialObject::RigidBody(0);
        let aabb = AABB::new(Vec3::zero(), Vec3::new(1.0, 1.0, 1.0));

        spatial_hash.insert(obj, aabb);
        assert!(!spatial_hash.grid.is_empty());

        spatial_hash.clear();
        assert!(spatial_hash.grid.is_empty());
    }

    #[test]
    fn test_spatial_hash_stats() {
        let mut spatial_hash = SpatialHash::new(1.0);

        for i in 0..10 {
            let obj = SpatialObject::RigidBody(i);
            let aabb = AABB::new(
                Vec3::new(i as f64, i as f64, i as f64),
                Vec3::new(i as f64 + 0.5, i as f64 + 0.5, i as f64 + 0.5),
            );
            spatial_hash.insert(obj, aabb);
        }

        let stats = spatial_hash.stats();
        assert!(stats.total_cells > 0);
        assert!(stats.total_objects > 0);
        assert_eq!(stats.cell_size, 1.0);
    }

    #[test]
    fn test_hierarchical_spatial_hash() {
        let mut hspatial = HierarchicalSpatialHash::new(1.0, 3);

        let obj = SpatialObject::RigidBody(0);
        let aabb = AABB::new(Vec3::zero(), Vec3::new(2.0, 2.0, 2.0));

        hspatial.insert(obj, aabb);

        let query_aabb = AABB::new(Vec3::new(-0.5, -0.5, -0.5), Vec3::new(0.5, 0.5, 0.5));
        let results = hspatial.query(query_aabb);

        assert!(results.contains(&obj));
    }

    #[test]
    fn test_broad_phase() {
        let mut broad_phase = BroadPhase::new(1.0);

        let objects = vec![
            (
                SpatialObject::RigidBody(0),
                AABB::new(Vec3::new(0.0, 0.0, 0.0), Vec3::new(1.0, 1.0, 1.0)),
            ),
            (
                SpatialObject::RigidBody(1),
                AABB::new(Vec3::new(0.5, 0.5, 0.5), Vec3::new(1.5, 1.5, 1.5)),
            ),
            (
                SpatialObject::RigidBody(2),
                AABB::new(Vec3::new(5.0, 5.0, 5.0), Vec3::new(6.0, 6.0, 6.0)),
            ),
        ];

        broad_phase.update(&objects);

        let pairs = broad_phase.get_collision_pairs();

        // Should find overlap between objects 0 and 1
        assert!(pairs.len() > 0);
        assert!(
            pairs.contains(&(SpatialObject::RigidBody(0), SpatialObject::RigidBody(1)))
                || pairs.contains(&(SpatialObject::RigidBody(1), SpatialObject::RigidBody(0)))
        );
    }

    #[test]
    fn test_query_point() {
        let mut broad_phase = BroadPhase::new(1.0);

        let objects = vec![
            (
                SpatialObject::RigidBody(0),
                AABB::new(Vec3::new(0.0, 0.0, 0.0), Vec3::new(1.0, 1.0, 1.0)),
            ),
            (
                SpatialObject::RigidBody(1),
                AABB::new(Vec3::new(5.0, 5.0, 5.0), Vec3::new(6.0, 6.0, 6.0)),
            ),
        ];

        broad_phase.update(&objects);

        let results = broad_phase.query_point(Vec3::new(0.5, 0.5, 0.5), 1.0);
        assert!(results.contains(&SpatialObject::RigidBody(0)));
        assert!(!results.contains(&SpatialObject::RigidBody(1)));
    }

    #[test]
    fn test_sweep_and_prune() {
        let mut sap = SweepAndPrune::new(0); // X-axis

        let objects = vec![
            (
                SpatialObject::RigidBody(0),
                AABB::new(Vec3::new(0.0, 0.0, 0.0), Vec3::new(2.0, 1.0, 1.0)),
            ),
            (
                SpatialObject::RigidBody(1),
                AABB::new(Vec3::new(1.0, 0.0, 0.0), Vec3::new(3.0, 1.0, 1.0)),
            ),
            (
                SpatialObject::RigidBody(2),
                AABB::new(Vec3::new(5.0, 0.0, 0.0), Vec3::new(6.0, 1.0, 1.0)),
            ),
        ];

        sap.update(&objects);

        let overlaps = sap.find_overlaps();

        // Should find overlap between objects 0 and 1 along X-axis
        assert!(overlaps.len() > 0);
        assert!(overlaps.contains(&(SpatialObject::RigidBody(0), SpatialObject::RigidBody(1))));
        assert!(!overlaps.contains(&(SpatialObject::RigidBody(0), SpatialObject::RigidBody(2))));
    }

    #[test]
    fn test_raycast() {
        let mut broad_phase = BroadPhase::new(1.0);

        let objects = vec![
            (
                SpatialObject::RigidBody(0),
                AABB::new(Vec3::new(5.0, -1.0, -1.0), Vec3::new(6.0, 1.0, 1.0)),
            ),
            (
                SpatialObject::RigidBody(1),
                AABB::new(Vec3::new(10.0, 10.0, 10.0), Vec3::new(11.0, 11.0, 11.0)),
            ),
        ];

        broad_phase.update(&objects);

        let ray_origin = Vec3::new(0.0, 0.0, 0.0);
        let ray_direction = Vec3::new(1.0, 0.0, 0.0);
        let max_distance = 10.0;

        let results = broad_phase.raycast(ray_origin, ray_direction, max_distance);

        // Should hit object 0, but not object 1
        assert!(results.contains(&SpatialObject::RigidBody(0)));
    }
}
