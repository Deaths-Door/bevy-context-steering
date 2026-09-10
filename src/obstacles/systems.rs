use bevy::ecs::entity::EntityHashMap;
use bevy_many_relationships::{ManyRelatedEntityCommands, OutgoingRelationships};

use super::*;

pub(crate) fn update_obstacles(
    agent_query: ActiveAgentsQuery<ObstacleQueryData>,
    spatial_query: SpatialQuery,
    commands: ParallelCommands,
) {
    agent_query.par_iter().for_each(|agent| {
        let shape = &agent.detection.shape;
        let shape_rotation = agent.detection.rotation;
        let config = &agent.detection.config;
        let direction = agent.detection.direction;

        let origin = agent.global_transform.translation();
        let filter =
            SpatialQueryFilter::from_mask(**agent.filter).with_excluded_entities([agent.entity]);

        let mut current_hits = EntityHashMap::new();
        let predicate = |hit: ShapeHitData| {
            current_hits.insert(hit.entity, hit);
            true
        };

        spatial_query.shape_hits_callback(
            shape,
            origin,
            shape_rotation,
            direction,
            config,
            &filter,
            predicate,
        );

        commands.command_scope(|mut commands| {
            let mut commands = commands.entity(agent.entity);

            if let Some(obstacle_iter) = agent.obstacles.map(|obstacle| obstacle.targets()) {
                for current_obstacle in obstacle_iter {
                    if !current_hits.contains_key(&current_obstacle) {
                        commands.remove_outgoing_to::<Obstacle>(current_obstacle);
                    }
                }
            }

            for (entity, shape_hit) in current_hits {
                commands.add_outgoing_to(entity, Obstacle { shape_hit });
            }
        });
    });
}

#[derive(QueryData)]
pub(crate) struct ObstacleQueryData {
    entity: Entity,

    global_transform: &'static GlobalTransform,
    filter: &'static ObstacleFilter,
    detection: &'static ObstacleDetection,

    obstacles: Option<&'static OutgoingRelationships<Obstacle>>,
}
