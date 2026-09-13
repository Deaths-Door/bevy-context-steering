mod sets;
mod systems;

pub use sets::*;

use super::*;
use crate::{behaviours::*, obstacles::update_obstacles};
use bevy_many_relationships::{
    IncomingRelationships, ManyRelationshipsPlugin, OutgoingRelationships,
};
use systems::*;

pub struct SteeringPlugin;

impl Plugin for SteeringPlugin {
    fn build(&self, app: &mut App) {
        // Add many relations plugin
        app.add_plugins(ManyRelationshipsPlugin);

        // Add the cluster map, and observers to keep map in sync with clusters in world
        app.init_resource::<ClusterMap>();
        app.add_observer(on_insert_cluster);
        app.add_observer(on_discard_cluster);

        // Config the sets
        app.configure_sets(FixedPreUpdate, SteeringSpatialSet);
        app.configure_sets(FixedUpdate, SteeringBehaviorSet);
        app.configure_sets(FixedPostUpdate, SteeringPhysicsSet);

        type DefaultGroupProperties = (
            SteeringGroupCentre,
            SteeringGroupMeanHeading,
            SteeringGroupMeanVelocity,
        );

        // when clsuter is spawned add its group properties
        app.add_observer(on_add_component_insert::<Cluster, DefaultGroupProperties>);

        // when cluster has no members, despawn the cluster, which will also remove the properties
        app.add_observer(
            on_remove_component_remove::<IncomingRelationships<ClusterMember>, Cluster>,
        );

        // when an agent has neighbours found, then add its group properties
        app.add_observer(
            on_add_component_insert::<OutgoingRelationships<Neighbour>, DefaultGroupProperties>,
        );

        // when agent has no neighbours, then remove its group properties as well
        app.add_observer(
            on_remove_component_remove::<OutgoingRelationships<Neighbour>, DefaultGroupProperties>,
        );

        app.add_systems(
            FixedPreUpdate,
            (
                update_neighbour_properties,
                update_cluster_properties,
                update_neighbours,
                update_obstacles,
            )
                .chain()
                .in_set(SteeringSpatialSet),
        );

        // set of all of the behaviours of the lib
        let behaviour_update = (
            Seek::steering_behaviour_update,
            Flee::steering_behaviour_update,
            Pursuit::steering_behaviour_update,
            Evade::steering_behaviour_update,
            Brake::steering_behaviour_update,
            Throttle::steering_behaviour_update,
            Cohere::steering_behaviour_update,
            Scatter::steering_behaviour_update,
            CohereCluster::steering_behaviour_update,
            ScatterCluster::steering_behaviour_update,
            AvoidObstacles::steering_behaviour_update,
        );

        // apply behaviours
        app.add_systems(FixedUpdate, behaviour_update.in_set(SteeringBehaviorSet));

        // instead the 2 weights,
        // ideally one would do it on spawn of coherecluster THEN on the cluster, but im not in the mood
        app.add_observer(
            on_add_component_insert::<Cluster, (CohereClusterWeight, ScatterClusterWeight)>,
        );

        // set of all motion types in the lib
        let motion_update = (
            motion::MotionKinematic::steering_motion_update,
            motion::MotionOmnidirectional::steering_motion_update,
            motion::MotionDirectional::steering_motion_update,
        );

        // apply the resultant field, and then the motion in sequence
        app.add_systems(
            FixedPostUpdate,
            (update_resultant_field, motion_update)
                .chain()
                .in_set(SteeringPhysicsSet),
        );
    }
}
