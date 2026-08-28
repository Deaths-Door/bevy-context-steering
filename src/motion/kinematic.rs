use super::*;

/// Applies steering output directly to `LinearVelocity` without physics forces.
#[derive(Component, Clone, Copy, Hash, PartialEq, Eq, Debug)]
pub struct MotionKinematic {
    /// Whether the agent automatically rotates to face its movement direction.
    pub face_movement_direction: bool,
}

impl MotionKinematic {
    /// Creates a new kinematic motion config with the given facing behavior.
    pub const fn new(face_movement_direction: bool) -> Self {
        Self {
            face_movement_direction,
        }
    }

    /// Returns a copy with the `face_movement_direction` flag replaced.
    pub const fn with_face_movement_direction(mut self, face_movement_direction: bool) -> Self {
        self.face_movement_direction = face_movement_direction;
        self
    }
}

impl Default for MotionKinematic {
    fn default() -> Self {
        Self {
            face_movement_direction: true,
        }
    }
}

impl MotionKinematic {
    pub(crate) fn apply(
        mut query: ActiveAgentsQuery<DirectKinematicQueryData, With<MotionKinematic>>,
    ) {
        query.par_iter_mut().for_each(|mut agent| {
            let velocity = agent
                .context
                .resultant_velocity()
                .unwrap_or(agent.context.resultant_direction());
            **agent.velocity = velocity;

            if agent.motion.face_movement_direction
                && let Some(facing_dir) = velocity.try_normalize()
            {
                let up = agent.transform.up();
                let target_rot = agent.transform.looking_to(facing_dir, up).rotation;
                agent.transform.rotation = target_rot;
            }
        });
    }
}

#[derive(QueryData)]
#[query_data(mutable)]
pub(crate) struct DirectKinematicQueryData {
    velocity: &'static mut LinearVelocity,
    transform: &'static mut Transform,

    context: &'static SteeringContext,

    motion: &'static MotionKinematic,
}
