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

    pub fn from_cache(cache: &SteeringCache) -> Self {
        Self::new(cache.directions().len())
    }

    /// Set interest at each direction, overwriting any previous value
    pub fn set_interest(&mut self, cache: &SteeringCache, dir: Vec3) {
        self.interest_with(cache, dir, overwrite);
    }

    /// Update interest at each direction, adding any previous value
    pub fn interest(&mut self, cache: &SteeringCache, dir: Vec3) {
        self.interest_with(cache, dir, |a, b| a + b);
    }

    /// Set danger at each direction, overwriting any previous value
    pub fn set_danger(&mut self, cache: &SteeringCache, dir: Vec3) {
        self.danger_with(cache, dir, overwrite);
    }

    /// Update the danger at each direction, using the max of the previous and new value
    pub fn danger(&mut self, cache: &SteeringCache, dir: Vec3) {
        self.danger_with(cache, dir, |a, b| a.max(b));
    }

    ///  Sets velocity at the given direction, overwriting any previous value
    pub fn set_velocity(&mut self, cache: &SteeringCache, direction: Vec3, target_velocity: Vec3) {
        self.velocity_with(cache, direction, target_velocity, overwrite);
    }

    /// Update velocity at the given direction, averaging the two values
    pub fn velocity(&mut self, cache: &SteeringCache, direction: Vec3, target_velocity: Vec3) {
        self.velocity_with(cache, direction, target_velocity, |a, b| (a + b) * 0.5);
    }

    /// Update interest while taking into account the previous value
    pub fn interest_with(
        &mut self,
        cache: &SteeringCache,
        dir: Vec3,
        combine: impl Fn(f32, f32) -> f32,
    ) {
        assert_cache_len!(self.field, cache);

        let dir = dir.normalize_or_zero();

        for (Weight { interest, .. }, direction) in
            self.field.iter_mut().zip(cache.directions().iter())
        {
            let new_interest = direction.dot(dir).max(0.0);
            *interest = (combine)(*interest, new_interest)
        }
    }

    /// Update danger with taking into account the previous value
    pub fn danger_with(
        &mut self,
        cache: &SteeringCache,
        dir: Vec3,
        combine: impl Fn(f32, f32) -> f32,
    ) {
        assert_cache_len!(self.field, cache);
        let dir = dir.normalize_or_zero();

        for (Weight { danger, .. }, direction) in
            self.field.iter_mut().zip(cache.directions().iter())
        {
            let new_danger = direction.dot(dir).max(0.0);
            *danger = (combine)(*danger, new_danger);
        }
    }

    pub fn velocity_with(
        &mut self,
        cache: &SteeringCache,
        direction: Vec3,
        target_velocity: Vec3,
        combine: impl Fn(Vec3, Vec3) -> Vec3,
    ) {
        let direction_slot = cache.nearest_direction_slot(direction);
        self.velocity_at_with(cache, direction_slot, target_velocity, combine);
    }

    /// Update velocity with taking into considersation the previous value
    fn velocity_at_with(
        &mut self,
        cache: &SteeringCache,
        direction_slot: usize,
        target_velocity: Vec3,
        combine: impl Fn(Vec3, Vec3) -> Vec3,
    ) {
        let directions = cache.directions();
        let neighbours = cache.direction_neighbours();
        let target_direction = directions[direction_slot];

        for &slot in &neighbours[direction_slot] {
            let direction = directions[slot];
            let wk = target_direction.dot(direction).max(0.0);
            let velocity = target_velocity * wk;

            match &mut self.field[slot].velocity {
                Some(v_old) => {
                    *v_old = (combine)(*v_old, velocity);
                }
                vel @ None => {
                    *vel = Some(velocity);
                }
            }
        }
    }

    /// Resets all interest values across the field to `0.0`.
    pub fn clear_interest(&mut self) {
        for Weight { interest, .. } in &mut self.field {
            *interest = 0.0;
        }
    }

    /// Resets all danger values across the field to `0.0`.
    pub fn clear_danger(&mut self) {
        for Weight { danger, .. } in &mut self.field {
            *danger = 0.0;
        }
    }

    /// Clears all velocity entries written to this behaviour.
    pub fn clear_velocity(&mut self) {
        for slot in &mut self.field {
            slot.velocity = None;
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
    pub const fn field_mut(&mut self) -> &mut SteeringField {
        &mut self.field
    }
}

fn overwrite<T>(_: T, b: T) -> T {
    b
}
