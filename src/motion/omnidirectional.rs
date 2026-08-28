use super::*;

/// Omnidirectional locomotion: forces may be applied along any axis to
/// reach the resultant velocity.
#[derive(Component, Clone, Copy, PartialEq, Debug)]
pub struct MotionOmnidirectional {
    /// Bounds acceleration directly
    pub max_acceleration: f32,

    /// Whether the agent automatically rotates to face its movement direction.
    pub face_movement_direction: bool,
}

impl MotionOmnidirectional {
    /// Creates a new omnidirectional motion config with the given
    /// acceleration cap
    pub const fn new(max_acceleration: f32) -> Self {
        Self {
            max_acceleration,
            face_movement_direction: true,
        }
    }

    /// Returns a copy with the acceleration cap replaced.
    pub const fn with_max_acceleration(mut self, max_acceleration: f32) -> Self {
        self.max_acceleration = max_acceleration;
        self
    }

    /// Returns a copy with the `face_movement_direction` flag replaced.
    pub const fn with_face_movement_direction(mut self, face_movement_direction: bool) -> Self {
        self.face_movement_direction = face_movement_direction;
        self
    }
}

impl MotionOmnidirectional {
    pub(crate) fn apply(
        time: Res<Time>,
        mut query: ActiveAgentsQuery<MotionQueryData, With<Self>>,
    ) {
        let dt = time.delta_secs();

        query.par_iter_mut().for_each(|mut agent| {
            let target_direction = agent.context.resultant_direction();

            let target_velocity = agent
                .context
                .resultant_velocity()
                .unwrap_or(target_direction * **agent.max_linear_speed);

            let delta_velocity = target_velocity - **agent.velocity;

            let max_acceleration = agent.motion.max_acceleration;
            let acceleration =
                (delta_velocity / dt).clamp_length(-max_acceleration, max_acceleration);

            let mass = agent.computed_mass.value();
            let force = mass * acceleration;
            agent.forces.apply_local_force(force);

            if agent.motion.face_movement_direction
                && let Some(facing_direction) = agent.velocity.try_normalize()
            {
                let up = agent.transform.up();
                let target_rot = agent.transform.looking_to(facing_direction, up).rotation;
                let max_angle = **agent.max_angular_speed * dt; // rad/s * s = rad this frame

                agent.transform.rotation = agent
                    .transform
                    .rotation
                    .rotate_towards(target_rot, max_angle);
            }
        });
    }
}

#[derive(QueryData)]
#[query_data(mutable)]
pub(crate) struct MotionQueryData {
    motion: &'static MotionOmnidirectional,
    context: &'static SteeringContext,

    computed_mass: &'static ComputedMass,

    max_linear_speed: &'static MaxLinearSpeed,
    max_angular_speed: &'static MaxAngularSpeed,

    velocity: &'static LinearVelocity,

    transform: &'static mut Transform,

    forces: Forces,
}
