use super::*;

/// Steering behavior that accelerates an agent toward a target position, line, or plane.
///
/// `Seek` calculates an attractive steering vector toward a spatial target point or line.
/// By configuring [`Self::axis_direction`], `Seek` unifies traditional **Point Seeking** 
/// (with falloff aka **Arrive**) and **Line/Rail Constraints**.
///
/// - **Point Seek** (`axis_direction == Vec3::ZERO`): Pulls the agent toward `target + offset` in 3D space.
/// - **Line / Rail Seek** (`axis_direction == unit vector`): Constrains the agent to an arbitrary 3D line. 
///   Pull forces act purely perpendicular to `axis_direction`, leaving motion along `axis_direction` completely unconstrained.
///
/// Use [`Self::offset`] to define formation offsets or relative target points.
///
/// The magnitude of the resulting force attenuates over distance according to the configured [`Falloff`].
///
/// # Examples
///
/// ```rust
/// use bevy::prelude::*;
/// use bevy_context_steering::{Seek, Falloff};
///
/// // 1. Traditional 3D Point Seek with offset
/// let point_seek = Seek::new(Vec3::new(10.0, 0.0, 0.0))
///     .with_offset(Vec3::new(0.0, 0.0, -2.0)) // 2 units behind target
///     .with_falloff(Falloff::SmoothStep { threshold: 20.0 });
///
/// // 2. Rail / Axis Lock along the X-axis (holds Y=0, Z=0)
/// let rail_seek = Seek::rail(Vec3::X, Vec3::ZERO);
/// ```
#[derive(Component, Debug, Clone, Reflect)]
#[reflect(Component, Debug)]
#[component(
    on_add = on_add_into_steering_context::<Self>, 
    on_remove = on_remove_from_steering_context::<Self>
)]
#[require(SteeringContext)]
pub struct Seek {
    /// Global spatial position the agent is attempting to seek.
    pub target: Vec3,
    /// Spatial offset applied relative to the target's position.
    ///
    /// - `Vec3::ZERO`: Seeks the target directly.
    /// - Non-zero `Vec3`: Seeks a relative point shifted from `target` (e.g. formation slots, offsets).
    pub offset: Vec3,
    /// Unconstrained free-motion axis.
    ///
    /// - `Vec3::ZERO`: Standard 3D Point Seek.
    /// - Normalized unit vector (e.g. `Vec3::Y`): Infinite Line / Rail Seek.
    pub axis_direction : Vec3, 
    /// Distance-based attenuation profile controlling force magnitude drop-off.
    pub falloff: Falloff,
}

impl Seek {
  /// Creates a new `Seek` behavior pointing toward `target` with no distance falloff ([`Falloff::None`]).
    pub const fn new(target: Vec3) -> Self {
        Self {
            target,
            offset : Vec3::ZERO,
            axis_direction : Vec3::ZERO,
            falloff: Falloff::None,
        }
    }

    /// **Rail / Line Seek.** Agent is free to move along `axis_direction`;
    /// pulled only in the plane perpendicular to it, toward the line through `position`.
    pub fn rail(axis_direction: Vec3, position: Vec3) -> Self {
        Self::new(position).with_axis_direction(axis_direction)
    }

    /// Sets the falloff behavior 
    pub const fn with_falloff(mut self, falloff: Falloff) -> Self {
        self.falloff = falloff;
        self
    }

    /// Sets the relative spatial offset from the target.
    pub const fn with_offset(mut self, offset: Vec3) -> Self {
        self.offset = offset;
        self
    }

    /// Sets the unconstrained direction axis 
    pub fn with_axis_direction(mut self, axis_direction: Vec3) -> Self {
        self.axis_direction = axis_direction.normalize_or_zero();
        self
    }
}

impl Seek {
    pub(crate) fn steering_behaviour_update(mut query: ActiveAgentsQuery<BehaviourQueryData<Self>>) {
        query
            .par_iter_mut()
            .for_each(|mut agent| {  
                let v = Self::v_direction(&agent);
             
                let distance = agent.transform.translation().distance(v);

                if distance <= f32::EPSILON {
                    agent.context.clear_interest::<Self>();
                } else {
                    let agent_position = agent.transform.translation();
                    let steering_dir = (v - agent_position).normalize_or_zero();

                    let factor = agent.behaviour.falloff.inwards_factor(distance);
                    let steering_dir = steering_dir * factor;
                    agent.context.set_interest::<Self>(steering_dir);
                }
            });
    }

    pub(super) fn v_direction<T:BehaviourData>(agent: &BehaviourQueryDataItem<'_, '_, T>)-> Vec3 {
        let agent_position = agent.transform.translation();
        let target = agent.behaviour.target() + agent.behaviour.offset();

        let target_direction =  agent_position - target;
        let agent_axis =  agent.behaviour.axis_direction();
        let t = target_direction.dot(agent_axis);
        let v = target + t * agent_axis;
        v
    }
}

#[derive(QueryData)]
#[query_data(mutable)]
pub(crate) struct BehaviourQueryData<T: Component> {
    pub(crate) behaviour: &'static T,
    pub(crate) transform: &'static GlobalTransform,
    pub(crate) context: &'static mut SteeringContext,
}

pub(crate) trait BehaviourData : Component {
    fn target(&self) -> Vec3;  
    fn offset(&self) -> Vec3;
    fn axis_direction(&self) -> Vec3;
}

impl BehaviourData for Seek {
    fn target(&self) -> Vec3 {
        self.target
    }

    fn offset(&self) -> Vec3 {
        self.offset   
    }
    fn axis_direction(&self) -> Vec3{
        self.axis_direction
    }
}

