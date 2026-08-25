use std::ops::{Deref, DerefMut};

use super::*;

/// A precomputed, immutable set of sample directions used by [`SteeringContext`]
/// for context-steering interest/danger/velocity maps.
pub struct SteeringDirectionsCache {
    /// Unit-length sample directions, indexed by slot.
    directions: Box<[Vec3]>,
    /// For each slot, the indices of its neighbouring slots including self (used for
    /// interpolating between slots when resolving a resultant direction/velocity).
    direction_neighbours: Box<[Box<[usize]>]>,
}

/// Identifies a particular shape/resolution of direction set, used as the
/// cache key in [`DirectionsRegistry`]. Two requests for the same kind will
/// always resolve to the same shared [`SteeringDirectionsCache`] instance.
#[derive(Hash, Eq, PartialEq, Clone)]
enum DirectionSetKind {
    /// Directions sampled over a 3D sphere (via a Fibonacci sphere).
    Spherical { count: usize },
    /// Directions sampled over a 2D plane ( evenly spaced around a circle).
    Plane { count: usize },
}

/// Process-wide cache of built [`SteeringDirectionsCache`] instances, keyed by
/// [`DirectionSetKind`]. Built lazily on first access; entries are built once
/// and shared (via `Arc`) for the lifetime of the process.
static DIRECTIONS_CACHE_REGISTRY: LazyLock<RwLock<DirectionsRegistry>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));

type DirectionsRegistry = HashMap<DirectionSetKind, Arc<SteeringDirectionsCache>>;

/// Acquires the registry for writing and runs `block` against it.
fn registry_mut<O>(block: impl FnOnce(&mut DirectionsRegistry) -> O) -> O {
    let mut registry = DIRECTIONS_CACHE_REGISTRY
        .write()
        .expect("Failed to acquire write lock on directions cache registry");

    (block)(registry.deref_mut())
}

/// Acquires the registry for reading and runs `block` against it.
fn registry<O>(block: impl FnOnce(&DirectionsRegistry) -> O) -> O {
    let registry = DIRECTIONS_CACHE_REGISTRY
        .read()
        .expect("Failed to acquire read lock on directions cache registry");

    (block)(registry.deref())
}

impl SteeringDirectionsCache {
    /// Returns a slice of unit-length sample directions indexed by slot.
    pub fn directions(&self) -> &[Vec3] {
        &self.directions
    }

    /// Returns a slice containing the neighbor slot indices (including self)
    /// for each slot, used for interpolating between slots.
    pub fn direction_neighbours(&self) -> &[Box<[usize]>] {
        &self.direction_neighbours
    }

    /// Finds the slot nearest `direction`
    pub fn nearest_direction_slot(&self, direction: Vec3) -> usize {
        let slot = self
            .directions
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.dot(direction).total_cmp(&b.dot(direction)))
            .map(|(i, _)| i)
            .expect("direction set is non-empty");

        slot
    }
}

impl SteeringDirectionsCache {
    /// Returns the shared spherical direction set with `count` sample
    /// directions, building and caching it on first request.
    ///
    /// Subsequent calls with the same `count` return a clone of the same
    /// `Arc`, not a fresh build.
    pub fn fibonacci_sphere(count: usize) -> Arc<Self> {
        let key = DirectionSetKind::Spherical { count };
        let cache = registry(|registry| registry.get(&key).cloned());

        match cache {
            Some(cache) => cache,
            None => registry_mut(|registry| {
                registry
                    .entry(DirectionSetKind::Spherical { count })
                    .or_insert_with(|| Arc::new(Self::build_fibonacci_sphere(count)))
                    .clone()
            }),
        }
    }

    pub(crate) const DEFAULT_DIRECTIONS_COUNT: usize = 128;

    /// Returns the shared default direction set (128-sample Fibonacci sphere),
    /// used by [`SteeringContext::default`] so agents that don't need a custom
    /// resolution all share one instance instead of each building their own.
    pub fn default_shared() -> Arc<Self> {
        Self::fibonacci_sphere(Self::DEFAULT_DIRECTIONS_COUNT)
    }
}

impl SteeringDirectionsCache {
    /// Builds a new Fibonacci-sphere-sampled direction set with `count`
    /// directions and precomputes per-slot neighbour adjacency.
    ///
    /// This is the actual (uncached) construction path; callers should go
    /// through [`SteeringDirectionsCache::fibonacci_sphere`] or
    /// [`SteeringDirectionsCache::default_shared`] instead of calling this
    /// directly, so the result is shared via the registry.
    fn build_fibonacci_sphere(count: usize) -> Self {
        let directions = Self::fib_directions(count);
        let direction_neighbours = Self::fib_neighbours(count, &directions);

        Self {
            direction_neighbours,
            directions,
        }
    }

    fn fib_directions(count: usize) -> Box<[Vec3]> {
        let golden_angle = PI * (5.0f32.sqrt() - 1.0);

        (0..count)
            .map(|i| {
                let y = 1.0 - ((i as f32) / (count as f32 - 1.0)) * 2.0;
                let r = (1.0 - y * y).sqrt();
                let theta = golden_angle * (i as f32);
                let x = r * cos(theta);
                let z = r * sin(theta);
                vec3(x, y, z)
            })
            .collect()
    }

    fn fib_neighbours(count: usize, directions: &[Vec3]) -> Box<[Box<[usize]>]> {
        // Angular distance between points roughly equal to sqrt ( 4pi / N )
        const TUNING: f32 = 1.25;
        let distance = ((4.0 * PI) / count as f32).sqrt();
        let radius = TUNING * distance;
        let threshold = cos(radius);

        directions
            .iter()
            .map(|target| {
                directions
                    .iter()
                    .enumerate()
                    // 1. Only include points within the dot product threshold
                    .filter(|(_, dir)| dir.dot(*target) > threshold)
                    .map(|(i, _)| i)
                    .collect()
            })
            .collect()
    }
}
