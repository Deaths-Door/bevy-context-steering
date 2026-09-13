use super::*;
use crate::neighbours::{Neighbour, NeighbourhoodExtents, NeighbourhoodFilter};
use avian3d::collision::collider::contact_query::{ClosestPoints, closest_points, contact};
use bevy::ecs::entity::EntityHashSet;
use bevy_many_relationships::{
    IncomingRelationships, ManyRelatedEntityCommands, OutgoingRelationships,
};

#[derive(QueryData)]
pub(crate) struct MemberClusterQueryData {
    transform: &'static GlobalTransform,
    velocity: &'static LinearVelocity,
}
#[derive(QueryData)]
#[query_data(mutable)]
pub(crate) struct NeighbourGroupQueryData {
    relationships: &'static IncomingRelationships<Neighbour>,
    centre: &'static mut SteeringGroupCentre,
    average_velocity: &'static mut SteeringGroupMeanVelocity,
    average_heading: &'static mut SteeringGroupMeanHeading,
}

pub(crate) fn update_neighbour_properties(
    mut query_group: ActiveAgentsQuery<NeighbourGroupQueryData>,
    query_members: Query<MemberClusterQueryData, With<IncomingRelationships<Neighbour>>>,
) {
    query_group.par_iter_mut().for_each(|mut item| {
        let mut total_centre = Vec3::ZERO;
        let mut total_velocity = Vec3::ZERO;
        let mut total_heading = Vec3::ZERO;
        let mut count = 0u32;

        query_members
            .iter_many(item.relationships.sources())
            .for_each(|member| {
                total_centre += member.transform.translation();
                total_velocity += **member.velocity;
                total_heading += member.transform.forward().as_vec3();
                count += 1
            });

        let centre = total_centre / count as f32;
        let average_velocity = total_velocity / count as f32;
        let average_heading = total_heading / count as f32;

        item.centre.0 = centre;
        item.average_velocity.0 = average_velocity;
        item.average_heading.0 = average_heading;
    });
}

// -------------------------

pub(crate) fn update_neighbours(
    agent_query: ActiveAgentsQuery<NeighbourhoodQueryData>,
    hit_query: Query<HitQueryData>,
    spatial_query: SpatialQuery,
    commands: ParallelCommands,
) {
    agent_query.par_iter().for_each(|agent| {
        let mut current_hits = EntityHashSet::new();
        let aabb = ColliderAabb::new(agent.global_transform.translation(), **agent.bounds);

        spatial_query.aabb_intersections_with_aabb_callback(aabb, |potential_hit| {
            if let Ok(hit_item) = hit_query.get(potential_hit)
                && hit_item
                    .collision_layers
                    .interacts_with(*agent.collision_layers)
            {
                current_hits.insert(potential_hit);
            }

            true
        });

        commands.command_scope(|mut commands| {
            let mut commands = commands.entity(agent.entity);

            if let Some(neighbour_iter) = agent.neighbours.map(|neighbour| neighbour.targets()) {
                for current_neighbour in neighbour_iter {
                    if !current_hits.contains(&current_neighbour) {
                        commands.remove_outgoing_to::<Neighbour>(current_neighbour);
                    }
                }
            }

            for &entity in &current_hits {
                let Ok(hit) = hit_query.get(entity) else {
                    unreachable!()
                };

                if let Some(neighbour) = neighbour(&agent, hit) {
                    commands.add_outgoing_to(entity, neighbour);
                }
            }
        });
    });
}

fn neighbour(
    agent: &NeighbourhoodQueryDataItem<'_, '_>,
    hit_item: HitQueryDataItem<'_, '_>,
) -> Option<Neighbour> {
    let collider1 = agent.collider;
    let collider2 = hit_item.collider;

    let position1 = agent.global_transform.translation();
    let position2 = hit_item.global_transform.translation();

    let rotation1 = agent.global_transform.rotation();
    let rotation2 = hit_item.global_transform.rotation();

    let max_distance = agent.bounds.min_element();

    let closest_points = closest_points(
        collider1,
        position1,
        rotation1,
        collider2,
        position2,
        rotation2,
        max_distance,
    );

    if let Ok(closest_points) = closest_points {
        let points = match closest_points {
            ClosestPoints::WithinMargin(a, b) => Some((a, b)),
            ClosestPoints::Intersecting
                if let Ok(Some(contact)) = {
                    // TODO: allow tuning this??
                    let prediction_distance = 0.01;
                    let contact = contact(
                        collider1,
                        position1,
                        rotation1,
                        collider2,
                        position2,
                        rotation2,
                        prediction_distance,
                    );
                    contact
                } =>
            {
                Some((
                    agent.global_transform.transform_point(contact.local_point1),
                    hit_item
                        .global_transform
                        .transform_point(contact.local_point2),
                ))
            }
            // Treate centres as closest points
            _ => None,
        };

        if let Some((agent_point_world, hit_point_world)) = points {
            let agent_point = agent_point_world - agent.global_transform.translation();
            let hit_point = hit_point_world - agent.global_transform.translation();
            let distance = (hit_point - agent_point).length();

            return Some(Neighbour {
                agent_point,
                hit_point,
                distance,
            });
        }
    }

    None
}

#[derive(QueryData)]
pub(crate) struct NeighbourhoodQueryData {
    entity: Entity,
    filter: &'static NeighbourhoodFilter,
    bounds: &'static NeighbourhoodExtents,
    neighbours: Option<&'static OutgoingRelationships<Neighbour>>,

    collision_layers: &'static CollisionLayers,
    collider: &'static Collider,

    global_transform: &'static GlobalTransform,
}

#[derive(QueryData)]
pub(crate) struct HitQueryData {
    collider: &'static Collider,
    collision_layers: &'static CollisionLayers,

    global_transform: &'static GlobalTransform,
}
