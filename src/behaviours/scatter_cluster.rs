use bevy::{ecs::system::SystemParam, platform::collections::HashSet};

use super::*;

/// Steering behavior that pulls the agent away from the center of mass of the cluster
#[derive(Component, Debug, Default, Deref, DerefMut)]
#[component(on_add = on_add_into_steering_context::<Self>, on_remove = on_remove_from_steering_context::<Self>)]
#[require(SteeringContext)]
pub struct ScatterCluster(HashSet<ClusterId>);

/// Inserted on the cluster entity itself
#[derive(Component, Debug, Reflect, Default)]
pub struct ScatterClusterWeight {
    pub weight: ClusterWeight,
}

impl ScatterCluster {
    pub(crate) fn steering_behaviour_update(
        system_parm: cohere_cluster::BehaviourSystemParm<Self>,
    ) {
        cohere_cluster::update_internal::<Self>(system_parm)
    }
}

impl cohere_cluster::Behaviour for ScatterCluster {
    fn clusters(&self) -> impl Iterator<Item = &ClusterId> {
        self.iter()
    }
    fn update_context(&self, context: &mut SteeringContext, target_direction: Vec3) {
        context.set_danger::<Self>(target_direction);
        // Needed forcases, where there is no other behaviours, since one expects a movement, but only danger shouldnt do anything
        context.set_interest::<Self>(-target_direction);
    }
}

impl From<ClusterId> for ScatterCluster {
    fn from(id: ClusterId) -> Self {
        Self::from_iter([id])
    }
}

impl FromIterator<ClusterId> for ScatterCluster {
    fn from_iter<T: IntoIterator<Item = ClusterId>>(iter: T) -> Self {
        Self(HashSet::from_iter(iter))
    }
}
