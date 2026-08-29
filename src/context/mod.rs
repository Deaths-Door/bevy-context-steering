mod behaviour;
mod cache;
mod field;
mod weight;

pub use behaviour::*;
pub use cache::*;
pub use field::*;
pub use weight::*;

use std::{
    any::TypeId,
    f32::consts::PI,
    sync::{Arc, LazyLock, RwLock},
};

use super::*;
use bevy::{
    math::ops::{cos, sin},
    platform::collections::HashMap,
};

/// This component maintains a map of active behaviors and the resulting combined
/// spatial field used for decision making.
#[derive(Component, Deref, DerefMut)]
pub struct SteeringContext {
    pub cache: Arc<SteeringCache>,

    /// Active behaviors indexed by their unique type ID.
    #[deref]
    behaviours: HashMap<TypeId, SteeringBehaviour>,

    /// The scratchpad to calculate the final, weighted combination of all interest and danger samples.
    resultant_field: SteeringField,

    resultant_direction: Vec3,
    resultant_velocity: Option<Vec3>,
}

impl Default for SteeringContext {
    fn default() -> Self {
        let cache = SteeringCache::default_shared();
        Self::new(cache)
    }
}

impl SteeringContext {
    pub fn new(cache: Arc<SteeringCache>) -> Self {
        let resultant_field = SteeringField::from_cache(&cache);

        Self {
            cache,
            resultant_field,
            behaviours: Default::default(),
            resultant_direction: Default::default(),
            resultant_velocity: Default::default(),
        }
    }

    /// Returns the calculated resultant direction of all active steering behaviours.
    pub const fn resultant_direction(&self) -> Vec3 {
        self.resultant_direction
    }

    /// Returns the calculated resultant velocity of all active steering behaviours.
    pub const fn resultant_velocity(&self) -> Option<Vec3> {
        self.resultant_velocity
    }

    /// Removes the steering behaviour of type `K` from the context.
    pub fn remove<K: 'static>(&mut self) {
        self.behaviours.remove(&TypeId::of::<K>());
    }

    /// Inserts a new default steering behaviour associated with type `K`.
    pub fn insert<K: 'static>(&mut self) {
        self.behaviours.insert(
            TypeId::of::<K>(),
            SteeringBehaviour::from_cache(&self.cache),
        );
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
        // Ideally use self.get_mut::<K>(), but partial borrows dont work
        self.behaviours
            .get_mut(&TypeId::of::<K>())
            .map(|behaviour| behaviour.set_interest(&self.cache, dir))
            .is_some()
    }

    /// Sets the danger direction vector for behaviour `K`. Returns `true` if updated, `false` otherwise.
    pub fn set_danger<K: 'static>(&mut self, dir: Vec3) -> bool {
        // Ideally use self.get_mut::<K>(), but partial borrows dont work

        self.behaviours
            .get_mut(&TypeId::of::<K>())
            .map(|v| v.set_danger(&self.cache, dir))
            .is_some()
    }

    /// Clears the interest direction vector for behaviour `K`. Returns `true` if updated, `false` otherwise.
    pub fn clear_interest<K: 'static>(&mut self) -> bool {
        // Ideally use self.get_mut::<K>(), but partial borrows dont work

        self.behaviours
            .get_mut(&TypeId::of::<K>())
            .map(|v| v.clear_interest())
            .is_some()
    }

    /// Clears the danger direction vector for behaviour `K`. Returns `true` if updated, `false` otherwise.
    pub fn clear_danger<K: 'static>(&mut self) -> bool {
        // Ideally use self.get_mut::<K>(), but partial borrows dont work
        self.behaviours
            .get_mut(&TypeId::of::<K>())
            .map(|v| v.clear_danger())
            .is_some()
    }

    /// Sets velocity for behaviour `K` mapped to the nearest direction slot.
    /// Returns `true` if the behaviour exists, `false` otherwise.
    pub fn set_velocity<K: 'static>(&mut self, direction: Vec3, target_velocity: Vec3) -> bool {
        // Ideally use self.get_mut::<K>(), but partial borrows dont work
        self.behaviours
            .get_mut(&TypeId::of::<K>())
            .map(|b| b.set_velocity(&self.cache, direction, target_velocity))
            .is_some()
    }

    /// Sets velocity for behaviour `K` at a specific direction slot index (overwrites).
    /// Returns `true` if the behaviour exists, `false` otherwise.
    pub fn set_velocity_at<K: 'static>(
        &mut self,
        direction_slot: usize,
        target_velocity: Vec3,
    ) -> bool {
        // Ideally use self.get_mut::<K>(), but partial borrows dont work

        self.behaviours
            .get_mut(&TypeId::of::<K>())
            .map(|b| b.set_velocity_at(&self.cache, direction_slot, target_velocity))
            .is_some()
    }

    /// Clears all stored velocity fields for behaviour `K`.
    /// Returns `true` if the behaviour exists, `false` otherwise.
    pub fn clear_velocity<K: 'static>(&mut self) -> bool {
        // Ideally use self.get_mut::<K>(), but partial borrows dont work

        self.behaviours
            .get_mut(&TypeId::of::<K>())
            .map(|b| b.clear_velocity())
            .is_some()
    }
}

impl SteeringContext {
    pub fn update(&mut self) {
        self.update_resultant_field();
        let resultant_slot = self.update_resultant_direction();
        self.update_resultant_velocity(resultant_slot)
    }

    pub(crate) fn update_resultant_velocity(&mut self, slot: Option<usize>) {
        let velocity = match slot {
            Some(slot) => self.interpolate_velocity(slot),
            None => Some(Vec3::ZERO),
        };

        self.resultant_velocity = velocity;
    }

    /// Smooths velocity using true weighted average to preserve speed magnitude given the winning slot.
    fn interpolate_velocity(&self, slot: usize) -> Option<Vec3> {
        let directions = self.cache.directions();
        let neighbours = &*self.cache.direction_neighbours()[slot];

        let mut total_velocity = None;
        let mut total_weight = 0.0;

        for &index in neighbours.iter() {
            if let Some(v) = self.resultant_field[index].velocity() {
                // Weight based on alignment with the continuous resultant direction
                let wk = self.resultant_direction.dot(directions[index]).max(0.0);
                if wk > f32::EPSILON {
                    *total_velocity.get_or_insert_default() += *v * wk;
                    total_weight += wk;
                }
            }
        }

        match total_weight > f32::EPSILON {
            true => total_velocity.map(|v: Vec3| v / total_weight),
            false => None,
        }
    }

    pub(crate) fn update_resultant_direction(&mut self) -> Option<usize> {
        // Find interest considering danger
        let field_iter = self.resultant_field.iter();

        // Nothing to react to at all — stay put, don't hand off to interpolate,
        // which always returns a unit vector even when weights sum to ~0.
        let is_clean = field_iter
            .clone()
            .all(|w| w.interest <= f32::EPSILON && w.danger <= f32::EPSILON);

        if is_clean {
            self.resultant_direction = Vec3::ZERO;
            return None;
        }

        let masks = into_masked_interest(field_iter);

        // Find the slot index with the highest remaining interest.
        let Some((resultant_slot, _)) = masks.enumerate().max_by(|(_, a), (_, b)| a.total_cmp(b))
        else {
            unreachable!()
        };

        self.resultant_direction = self.interpolate_direction(resultant_slot);

        Some(resultant_slot)
    }

    /// Allows for more 'natural-ish' movement
    fn interpolate_direction(&self, slot: usize) -> Vec3 {
        let directions = self.cache.directions();
        let neighbours = &*self.cache.direction_neighbours()[slot];
        let neighbouring_weights = neighbours.iter().map(|index| &self.resultant_field[*index]);
        let masks = into_masked_interest(neighbouring_weights);

        let mut interpolated_direction = Vec3::ZERO;
        let mut weights = 0.0;

        for (interest, index) in masks.zip(neighbours.iter()) {
            interpolated_direction += interest * directions[*index];
            weights += interest;
        }

        match weights > f32::EPSILON {
            true => interpolated_direction.normalize_or_zero(),
            false => directions[slot],
        }
    }

    pub(crate) fn update_resultant_field(&mut self) {
        // Reset the resultant field to a clean state (0.0 interest, 0.0 danger).
        self.resultant_field.fill(Weight::default());

        for behaviour in self.behaviours.values() {
            let field = behaviour.field();
            let weight = behaviour.weight();

            if weight <= f32::EPSILON {
                continue;
            }

            for (
                resultant,
                Weight {
                    interest,
                    danger,
                    velocity,
                },
            ) in self.resultant_field.iter_mut().zip(field.iter())
            {
                // Accumulate Interests: Add weighted interest to the total.
                // Weigh the added interest to be able to priotise certain directtions
                resultant.interest += interest * weight;
                resultant.danger = resultant.danger.max(*danger);

                if let Some(incoming_vel) = velocity {
                    *resultant.velocity.get_or_insert(Vec3::ZERO) += *incoming_vel * weight;
                }
            }
        }
    }
}

fn into_masked_interest<'a>(iter: impl Iterator<Item = &'a Weight>) -> impl Iterator<Item = f32> {
    iter.map(|slot| slot.interest * (1.0 - slot.danger))
}
