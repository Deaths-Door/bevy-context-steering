use super::*;

/// directional locomotion: forces may be applied along any axis to
/// reach the resultant velocity.
#[derive(Component, Clone, Copy, PartialEq, Debug)]
pub struct MotionDirectional {
    /// Bounds acceleration directly
    pub max_acceleration: f32,
}

impl MotionDirectional {
    /// Creates a new directional motion config with the given
    /// acceleration cap
    pub const fn new(max_acceleration: f32) -> Self {
        Self { max_acceleration }
    }

    /// Returns a copy with the acceleration cap replaced.
    pub const fn with_max_acceleration(mut self, max_acceleration: f32) -> Self {
        self.max_acceleration = max_acceleration;
        self
    }
}

impl MotionDirectional {
    pub(crate) fn apply(
        time: Res<Time>,
        mut query: ActiveAgentsQuery<MotionQueryData, With<Self>>,
    ) {
        let dt = time.delta_secs();

        query.par_iter_mut().for_each(|mut agent| {
            let target_direction = agent.context.resultant_direction();

            // 1. Turn the agent toward where it wants to go
            if let Some(flat_target_dir) = target_direction.try_normalize() {
                let current_rot = agent.transform.rotation;

                let up = agent.transform.up();
                let target_rot = agent.transform.looking_to(flat_target_dir, up).rotation;

                let rotation_diff = target_rot * current_rot.inverse();
                let (axis, angle) = rotation_diff.to_axis_angle();

                let mut angle_error = angle;
                if angle_error > std::f32::consts::PI {
                    angle_error -= std::f32::consts::TAU;
                }

                let max_angular_speed = **agent.max_angular_speed;
                let target_ang_velocity =
                    (angle_error / dt).clamp(-max_angular_speed, max_angular_speed);
                let current_ang_velocity = agent.angular_velocity.dot(axis);

                let angular_acceleration = (target_ang_velocity - current_ang_velocity) / dt;
                let inertia = agent.computed_angular_inertia.value();

                let torque = inertia * axis * angular_acceleration;
                agent.forces.apply_torque(torque);
            }

            // 2. Movement Split along the forward axis
            let forward = agent.transform.forward().as_vec3();
            let max_speed = **agent.max_linear_speed;

            // Desired velocity vector from steering context
            let target_velocity = agent
                .context
                .resultant_velocity()
                .unwrap_or_else(|| target_direction * max_speed);

            // PROJECT onto the forward axis (this splits it so we only care about forward/reverse intent)
            let target_forward_speed = target_velocity.dot(forward);
            let current_forward_speed = agent.velocity.dot(forward);

            let delta_v = target_forward_speed - current_forward_speed;
            let desired_accel = delta_v / dt;

            let max_acceleration = agent.motion.max_acceleration;
            let clamped_accel = desired_accel.clamp(-max_acceleration, max_acceleration);

            // Force = mass * acceleration, pushed entirely along the local/world forward axis
            let force_magnitude = agent.computed_mass.value() * clamped_accel;
            let force = forward * force_magnitude;

            agent.forces.apply_force(force);
        });
    }
}

#[derive(QueryData)]
#[query_data(mutable)]
pub(crate) struct MotionQueryData {
    motion: &'static MotionDirectional,
    context: &'static SteeringContext,

    computed_mass: &'static ComputedMass,
    computed_angular_inertia: &'static ComputedAngularInertia,

    max_linear_speed: &'static MaxLinearSpeed,
    max_angular_speed: &'static MaxAngularSpeed,

    velocity: &'static LinearVelocity,
    angular_velocity: &'static AngularVelocity,

    transform: &'static mut Transform,

    forces: Forces,
}
