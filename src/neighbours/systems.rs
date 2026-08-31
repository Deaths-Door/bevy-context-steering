use bevy_many_relationships::ManyRelatedEntityCommands;

use crate::neighbours::{Neighbour, NeighbourhoodBounds, NeighbourhoodFilter};

use super::*;

// REMOVE PREVIOUS FRAME NEIGHBOURS 
pub(crate) fn update_neighbours(
    mut agent_query: ActiveAgentsQuery<NeighbourhoodQueryData>,
    layer_query: Query<&CollisionLayers>,
    spatial_query: SpatialQuery,
    commands: ParallelCommands,
) {
    agent_query.par_iter_mut().for_each(|agent| {
        let filter =
            SpatialQueryFilter::from_mask(**agent.filter).with_excluded_entities([agent.entity]);

        let mut hits = Vec::new();

        // TODO: clear previous list.. eg using entityhashmap.interesect?? does that handle clear case, no right? 
        spatial_query.aabb_intersections_with_aabb_callback(**agent.bounds, |potential_hit| {
            if let Ok(layers) = layer_query.get(potential_hit)
                && filter.test(potential_hit, *layers)
            {
                hits.push(potential_hit);
            }

            true
        });

        commands.command_scope(|mut commands| {
            commands
                .entity(agent.entity)
                .add_many_related::<Neighbour>(hits.as_slice());
        });
    });
}

#[derive(QueryData)]
pub(crate) struct NeighbourhoodQueryData {
    entity: Entity,
    filter: &'static NeighbourhoodFilter,
    bounds: &'static NeighbourhoodBounds,
}
