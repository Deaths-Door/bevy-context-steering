use super::*;

/// A singular steering logic unit (e.g., Seek, Flee, Obstacle Avoidance).
pub struct SteeringBehaviour {
    field: SteeringField,
    /// Used to prioritize this behavior during the blending process.
    weight: f32,
}

macro_rules! assert_cache_len {
    ($field:expr, $cache:expr) => {
        debug_assert_eq!(
            $field.len(),
            $cache.directions().len(),
            "Steering cache size mismatch: field length ({}) != cache directions length ({})",
            $field.len(),
            $cache.directions().len()
        );
    };
}

impl SteeringBehaviour {
    pub fn new(count: usize) -> Self {
        Self {
            field: SteeringField::new(count),
            weight: 1.0,
        }
    }

    pub fn from_cache(cache: &SteeringDirectionsCache) -> Self {
        Self::new(cache.directions().len())
    }

    /// Assigns interest to each direction based on the given input  vector.
    pub fn set_interest(&mut self, cache: &SteeringDirectionsCache, dir: Vec3) {
        assert_cache_len!(self.field, cache);

        let dir = dir.normalize_or_zero();

        for (Weight { interest, .. }, direction) in
            self.field.iter_mut().zip(cache.directions().iter())
        {
            let new_interest = direction.dot(dir).max(0.0);
            *interest = new_interest
        }
    }

    ///  Assigns danger to each direction based on the given input vector.
    pub fn set_danger(&mut self, cache: &SteeringDirectionsCache, dir: Vec3) {
        assert_cache_len!(self.field, cache);
        let dir = dir.normalize_or_zero();

        for (Weight { danger, .. }, direction) in
            self.field.iter_mut().zip(cache.directions().iter())
        {
            let new_danger = direction.dot(dir).max(0.0);
            *danger = new_danger
        }
    }

    /// Resets all interest values across the field to `0.0`.
    pub fn clear_interest(&mut self) {
        for Weight { interest, .. } in self.field.iter_mut() {
            *interest = 0.0;
        }
    }

    /// Resets all danger values across the field to `0.0`.
    pub fn clear_danger(&mut self) {
        for Weight { danger, .. } in self.field.iter_mut() {
            *danger = 0.0;
        }
    }

    /// Returns the overall weight multiplier of this steering behaviour.
    pub const fn weight(&self) -> f32 {
        self.weight
    }

    /// Sets the weight multiplier for this steering behaviour.
    pub fn set_weight(&mut self, weight: f32) {
        self.weight = weight;
    }

    /// Returns a reference to the underlying [`SteeringField`].
    pub const fn field(&self) -> &SteeringField {
        &self.field
    }

    /// Returns a mutable reference to the underlying [`SteeringField`].
    pub fn field_mut(&mut self) -> &mut SteeringField {
        &mut self.field
    }
}
