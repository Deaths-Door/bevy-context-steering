use std::{any::TypeId, f32::consts::PI, sync::LazyLock};

use super::*;
use bevy::{
    math::ops::{cos, sin},
    platform::collections::HashMap,
};

const SAMPLE_SIZE: usize = 128;
pub(crate) static DIRECTIONS: LazyLock<[Vec3; SAMPLE_SIZE]> = LazyLock::new(|| {
    let golden_angle = PI * (5.0f32.sqrt() - 1.0);

    std::array::from_fn(|i| {
        let y = 1.0 - ((i as f32) / (SAMPLE_SIZE as f32 - 1.0)) * 2.0;
        let r = (1.0 - y * y).sqrt();
        let theta = golden_angle * (i as f32);
        let x = r * cos(theta);
        let z = r * sin(theta);
        vec3(x, y, z)
    })
});

/// A lookup table mapping each direction index to its immediate neighbors (including self) .
static NEIGHBOURS: LazyLock<[Box<[usize]>; SAMPLE_SIZE]> = LazyLock::new(|| {
    // Angular distance between points roughly equal to sqrt ( 4pi / N )
    const TUNING: f32 = 1.25;
    let distance = ((4.0 * PI) / SAMPLE_SIZE as f32).sqrt();
    let radius = TUNING * distance;
    let threshold = cos(radius);

    std::array::from_fn(|index| {
        let target = DIRECTIONS[index];
        DIRECTIONS
            .iter()
            .enumerate()
            // 1. Only include points within the dot product threshold
            .filter(|(_, dir)| dir.dot(target) > threshold)
            .map(|(i, _)| i)
            .collect()
    })
});

/// This component maintains a map of active behaviors and the resulting combined
/// spatial field used for decision making.
#[derive(Component, Default, Deref, DerefMut)]
pub struct SteeringContext {
    /// Active behaviors indexed by their unique type ID.
    #[deref]
    behaviours: HashMap<TypeId, SteeringBehaviour>,
    /// The final, weighted combination of all interest and danger samples.
    resultant_field: SteeringField,

    resultant_direction: Vec3,
}

/// A singular steering logic unit (e.g., Seek, Flee, Obstacle Avoidance).
pub struct SteeringBehaviour {
    field: SteeringField,
    /// Used to prioritize this behavior during the blending process.
    weight: f32,
}

/// Length shoould always be SAMPLE_SIZE
#[derive(Clone, Deref, DerefMut)]
pub struct SteeringField(Box<[Weight; SAMPLE_SIZE]>);

/// A directional weight pair representing the desirability and risk of a specific vector.
#[derive(Default, Clone)]
pub struct Weight {
    /// How much the agent wants to move in this direction.
    interest: f32,
    /// How much the agent wants to avoid moving in this direction.
    danger: f32,
}

impl Weight {
    /// Creates a new `Weight` instance with specified interest and danger values.
    pub const fn new(interest: f32, danger: f32) -> Self {
        Self { interest, danger }
    }

    /// Returns the interest value.
    pub const fn interest(&self) -> f32 {
        self.interest
    }

    /// Returns the danger value.
    pub const fn danger(&self) -> f32 {
        self.danger
    }

    /// Sets the interest value.
    pub const fn set_interest(&mut self, interest: f32) {
        self.interest = interest;
    }

    /// Sets the danger value.
    pub const fn set_danger(&mut self, danger: f32) {
        self.danger = danger;
    }
}

impl Default for SteeringField {
    fn default() -> Self {
        Self(Box::new(std::array::from_fn(|_| Weight::default())))
    }
}

impl Default for SteeringBehaviour {
    fn default() -> Self {
        Self {
            field: Default::default(),
            weight: 1.0,
        }
    }
}

impl SteeringBehaviour {
    /// Assigns interest to each direction based on the given input  vector.
    pub fn set_interest(&mut self, dir: Vec3) {
        let dir = dir.normalize_or_zero();

        for (Weight { interest, .. }, direction) in self.field.iter_mut().zip(DIRECTIONS.iter()) {
            let new_interest = direction.dot(dir).max(0.0);
            *interest = new_interest
        }
    }

    ///  Assigns danger to each direction based on the given input vector.
    pub fn set_danger(&mut self, dir: Vec3) {
        let dir = dir.normalize_or_zero();

        for (Weight { danger, .. }, direction) in self.field.iter_mut().zip(DIRECTIONS.iter()) {
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

impl SteeringContext {
    /// Returns the calculated resultant direction of all active steering behaviours.
    pub const fn resultant_direction(&self) -> Vec3 {
        self.resultant_direction
    }

    /// Removes the steering behaviour of type `K` from the context.
    pub fn remove<K: 'static>(&mut self) {
        self.behaviours.remove(&TypeId::of::<K>());
    }

    /// Inserts a new default steering behaviour associated with type `K`.
    pub fn insert<K: 'static>(&mut self) {
        self.behaviours
            .insert(TypeId::of::<K>(), SteeringBehaviour::default());
    }

    /// Returns an immutable reference to the steering behaviour of type `K`, if present.
    pub fn get<K: 'static>(&self) -> Option<&SteeringBehaviour> {
        self.behaviours.get(&TypeId::of::<K>())
    }

    /// Returns a mutable reference to the steering behaviour of type `K`, if present.
    pub fn get_mut<K: 'static>(&mut self) -> Option<&mut SteeringBehaviour> {
        self.behaviours.get_mut(&TypeId::of::<K>())
    }

    /// Returns `true` if a steering behaviour of type `K` exists in the context.
    pub fn contains<K: 'static>(&self) -> bool {
        self.behaviours.contains_key(&TypeId::of::<K>())
    }

    /// Sets the weight of the behaviour `K`. Returns `true` if updated, or `false` if `K` was not found.
    pub fn set_weight<K: 'static>(&mut self, weight: f32) -> bool {
        self.get_mut::<K>().map(|v| v.set_weight(weight)).is_some()
    }

    /// Sets the interest direction vector for behaviour `K`. Returns `true` if updated, `false` otherwise.
    pub fn set_interest<K: 'static>(&mut self, dir: Vec3) -> bool {
        self.get_mut::<K>().map(|v| v.set_interest(dir)).is_some()
    }

    /// Sets the danger direction vector for behaviour `K`. Returns `true` if updated, `false` otherwise.
    pub fn set_danger<K: 'static>(&mut self, dir: Vec3) -> bool {
        self.get_mut::<K>().map(|v| v.set_danger(dir)).is_some()
    }

    /// Clears the interest direction vector for behaviour `K`. Returns `true` if updated, `false` otherwise.
    pub fn clear_interest<K: 'static>(&mut self) -> bool {
        self.get_mut::<K>().map(|v| v.clear_interest()).is_some()
    }

    /// Clears the danger direction vector for behaviour `K`. Returns `true` if updated, `false` otherwise.
    pub fn clear_danger<K: 'static>(&mut self) -> bool {
        self.get_mut::<K>().map(|v| v.clear_danger()).is_some()
    }
}

impl SteeringContext {
    pub(crate) fn update(&mut self) {
        self.update_resultant_field();
        self.update_resultant_direction();
    }

    pub(crate) fn update_resultant_direction(&mut self) {
        // Find interest considering danger
        let field_iter = self.resultant_field.iter();

        // Nothing to react to at all — stay put, don't hand off to interpolate,
        // which always returns a unit vector even when weights sum to ~0.
        let is_clean = field_iter
            .clone()
            .all(|w| w.interest <= f32::EPSILON && w.danger <= f32::EPSILON);

        if is_clean {
            self.resultant_direction = Vec3::ZERO;
            return;
        }

        let masks = into_masked_interest(field_iter);

        // Find the slot index with the highest remaining interest.
        let Some((resultant_slot, _)) = masks.enumerate().max_by(|(_, a), (_, b)| a.total_cmp(b))
        else {
            unreachable!()
        };

        self.resultant_direction = self.interpolate(resultant_slot);
    }

    /// Allows for more 'natural-ish' movement
    fn interpolate(&self, slot: usize) -> Vec3 {
        let neighbours = &NEIGHBOURS[slot];
        let neighbouring_weights = neighbours.iter().map(|index| &self.resultant_field[*index]);
        let masks = into_masked_interest(neighbouring_weights);

        let mut direction = Vec3::ZERO;
        let mut weights = 0.0;

        for (interest, index) in masks.zip(neighbours.iter()) {
            direction += interest * DIRECTIONS[*index];
            weights += interest;
        }

        match weights > f32::EPSILON {
            true => direction.normalize_or_zero(),
            false => DIRECTIONS[slot],
        }
    }

    pub(crate) fn update_resultant_field(&mut self) {
        // Reset the resultant field to a clean state (0.0 interest, 0.0 danger).
        self.resultant_field.fill(Weight::default());

        for SteeringBehaviour { field, weight } in self.behaviours.values() {
            for (resultant, Weight { interest, danger }) in
                self.resultant_field.iter_mut().zip(field.iter())
            {
                // Accumulate Interests: Add weighted interest to the total.
                // Weigh the added interest to be able to priotise certain directtions
                resultant.interest += interest * weight;
                resultant.danger = resultant.danger.max(*danger);
            }
        }
    }
}

fn into_masked_interest<'a>(iter: impl Iterator<Item = &'a Weight>) -> impl Iterator<Item = f32> {
    iter.map(|slot| slot.interest * (1.0 - slot.danger))
}

// sanity checks
#[cfg(test)]
mod tests {
    use super::*;

    struct Behaviour;
    struct Behaviour2;

    fn apply_test(expected: Vec3, apply: impl FnOnce(&mut SteeringContext)) -> SteeringContext {
        let mut context = SteeringContext::default();
        context.insert::<Behaviour>();
        context.insert::<Behaviour2>();

        apply(&mut context);
        context.update();

        let resultant_direction = context.resultant_direction();
        assert_approx_dir(resultant_direction, expected, 0.98);
        context
    }

    fn assert_approx_dir(actual: Vec3, expected: Vec3, threshold: f32) {
        let dot = actual.normalize_or_zero().dot(expected.normalize_or_zero());
        assert!(
            dot >= threshold,
            "Directions do not match! \nActual:   {:?}\nExpected: {:?}\nDot:      {}",
            actual,
            expected,
            dot
        );
    }

    #[test]
    fn interest_only() {
        let intended_diretion = DIRECTIONS[0];
        let expected_direction = intended_diretion;

        apply_test(expected_direction, |context| {
            context.set_interest::<Behaviour>(intended_diretion);
        });
    }

    #[test]
    fn danger_only() {
        let mut context = SteeringContext::default();
        context.insert::<Behaviour>();

        let intended_diretion = DIRECTIONS[0];
        let expected_direction = -intended_diretion;

        apply_test(expected_direction, |context| {
            context.set_danger::<Behaviour>(intended_diretion);
        });
    }

    #[test]
    fn nullify_singular_direction() {
        let north = Vec3::Y;
        let east = Vec3::X;

        apply_test(east, |context| {
            context.set_interest::<Behaviour>(north);
            // Danger exactly cancels the interest at North
            context.set_danger::<Behaviour>(north);
            context.set_interest::<Behaviour2>(east);
        });
    }

    #[test]
    fn interpolation() {
        let center_idx = SAMPLE_SIZE / 2;
        let neighbor_idx = NEIGHBOURS[center_idx][1];
        let off_grid_target = (DIRECTIONS[center_idx] + DIRECTIONS[neighbor_idx]).normalize();
        apply_test(off_grid_target, |context| {
            context.set_interest::<Behaviour>(off_grid_target);
        });
    }

    #[test]
    fn weighted_behaviours() {
        let north = Vec3::Y;
        let east = Vec3::X;

        let weight_north = 2.0;
        let weight_east = 1.0;

        // The expected direction is the weighted vector sum:
        // (2.0 * North) + (1.0 * East) normalized.
        let expected = (north * weight_north + east * weight_east).normalize();

        let context = apply_test(expected, |context| {
            // Insert and configure first behavior
            context.set_interest::<Behaviour>(north);
            context.set_weight::<Behaviour>(weight_north);

            context.set_interest::<Behaviour2>(east);
            context.set_weight::<Behaviour2>(weight_east);
        });

        let result = context.resultant_direction();
        // Final sanity check: The result should be closer to North than to East
        assert!(result.dot(north) > result.dot(east));
    }
}
