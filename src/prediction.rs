use super::*;

#[derive(PartialEq, Debug, Clone, Reflect)]
pub struct EntityPrediction {
    pub max_prediction_time: f32,
    pub align_threshold: f32,
    pub ahead_threshold: f32,
}

impl Default for EntityPrediction {
    fn default() -> Self {
        Self {
            align_threshold: 0.95,
            ahead_threshold: 0.0,
            max_prediction_time: 2.0,
        }
    }
}

impl EntityPrediction {
    pub fn predict_position(
        &self,
        agent_translation: Vec3,
        target_translation: Vec3,
        agent_velocity: Vec3,
        target_velocity: Vec3,
    ) -> Vec3 {
        let seperation = target_translation - agent_translation;
        let relative_velocity = target_velocity - agent_velocity;

        let distance = seperation.length();
        let relative_speed = relative_velocity.length();
        let mut time = match relative_speed > f32::EPSILON {
            true => distance / relative_speed,
            false => 0.0,
        };

        let agent_direction = agent_velocity.normalize_or_zero();
        let target_direction = target_velocity.normalize_or_zero();

        let is_aligned = agent_direction.dot(target_direction) > self.align_threshold;
        let is_ahead = target_direction.dot(seperation.normalize_or_zero()) < self.ahead_threshold;

        if is_aligned && is_ahead {
            time = 0.0;
        }

        time = time.min(self.max_prediction_time);

        let ghost_target = target_translation + (target_velocity * time);
        ghost_target
    }
}
