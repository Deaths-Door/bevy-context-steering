mod shared;

use bevy_many_relationships::OutgoingRelationships;
use shared::*;

type ONeighbours = OutgoingRelationships<Neighbour>;

// -----------------------------------------------------------------------------
// 1. Basic Multi-Agent Inclusion & Out-of-Bounds Exclusions
// -----------------------------------------------------------------------------
#[test]
fn test_multiple_agents_spatial_bounds() {
    let mut app = App::test();

    // Observer at origin (extent 5.0 -> AABB [-5, 5])
    let observer = app.agent(|c| c.neighbour(LayerMask::DEFAULT, Vec3::splat(5.0)));
    // Target 1 inside bounds
    let target_inside = app.agent(|c| {
        c.neighbour(LayerMask::DEFAULT, Vec3::splat(5.0))
            .insert(Transform::from_translation(Vec3::new(3.0, 0.0, 0.0)))
    });
    // Target 2 outside bounds
    let target_outside = app.agent(|c| {
        c.neighbour(LayerMask::DEFAULT, Vec3::splat(5.0))
            .insert(Transform::from_translation(Vec3::new(12.0, 0.0, 0.0)))
    });

    app.step_frame();
    app.step_frame();

    let neighbours = app
        .get::<ONeighbours>(observer)
        .targets()
        .collect::<Vec<_>>();

    // TODO: check why does this shit fail with stepframes but not with step (aka 30frames)
    assert_eq!(neighbours.len(), 1);
    assert!(neighbours.contains(&target_inside));
    assert!(!neighbours.contains(&target_outside));
}

// -----------------------------------------------------------------------------
// 2. Layer Filtering Across Multiple Agents
// -----------------------------------------------------------------------------
#[test]
fn test_layer_mask_filtering() {
    let mut app = App::test();
    let layer_a = LayerMask::DEFAULT;
    let layer_b = LayerMask(2);

    let observer = app.agent(|c| c.neighbour(layer_a, Vec3::splat(5.0)));
    let matching_agent = app.agent(|c| {
        c.neighbour(layer_a, Vec3::splat(5.0))
            .insert(Transform::from_translation(Vec3::new(2.0, 0.0, 0.0)))
    });
    let filtered_agent = app.agent(|c| {
        c.neighbour(layer_b, Vec3::splat(5.0))
            .insert(Transform::from_translation(Vec3::new(2.0, 0.0, 0.0)))
    });

    app.step_frame();
    app.step_frame();
    app.step();
    let neighbours = app
        .get::<ONeighbours>(observer)
        .targets()
        .collect::<Vec<_>>();

    assert_eq!(neighbours, vec![matching_agent]);
    assert!(!neighbours.contains(&filtered_agent));
}

// -----------------------------------------------------------------------------
// 3. Dynamic Adding & Removing Between Frames
// -----------------------------------------------------------------------------
#[test]
fn test_spawn_and_despawn_between_frames() {
    let mut app = App::test();

    let observer = app.agent(|c| c.neighbour(LayerMask::DEFAULT, Vec3::splat(5.0)));
    app.step_frame();

    // Frame 1: Initial state (empty)
    assert!(app.world().get::<ONeighbours>(observer).is_none());

    // Frame 2: Spawn new agent in range
    let target = app.agent(|c| {
        c.neighbour(LayerMask::DEFAULT, Vec3::splat(5.0))
            .insert(Transform::from_translation(Vec3::new(2.0, 0.0, 0.0)))
    });

    app.step_frame();

    let neighbours = app
        .get::<ONeighbours>(observer)
        .targets()
        .collect::<Vec<_>>();

    assert_eq!(neighbours, vec![target]);

    // Frame 3: Despawn target
    app.world_mut().despawn(target);
    app.step_frame();
    assert!(app.world().get::<ONeighbours>(observer).is_none());
}

// -----------------------------------------------------------------------------
// 4. Dynamic Bounds Changes (Expanding & Contracting Extent)
// -----------------------------------------------------------------------------
#[test]
fn test_bounds_change_updates_relationships() {
    let mut app = App::test();

    let observer = app.agent(|c| c.neighbour(LayerMask::DEFAULT, Vec3::splat(5.0)));
    let target = app.agent(|c| {
        c.neighbour(LayerMask::DEFAULT, Vec3::splat(5.0))
            .insert(Transform::from_translation(Vec3::new(8.0, 0.0, 0.0)))
    });

    app.step_frame();
    // Out of initial 5.0 extent
    assert!(app.world().get::<ONeighbours>(observer).is_none());

    // Expand observer bounds to reach target
    app.world_mut()
        .entity_mut(observer)
        .insert(NeighbourhoodExtents::from(Vec3::splat(10.0)));
    app.step_frame();

    let neighbours = app
        .get::<ONeighbours>(observer)
        .targets()
        .collect::<Vec<_>>();

    assert_eq!(neighbours, vec![target]);

    // Contract observer bounds again
    app.world_mut()
        .entity_mut(observer)
        .insert(NeighbourhoodExtents::from(Vec3::splat(2.0)));
    app.step_frame();
    assert!(app.world().get::<ONeighbours>(observer).is_none());
}

// -----------------------------------------------------------------------------
// 5. Dynamic Layer Mask Changes Between Frames
// -----------------------------------------------------------------------------
#[test]
fn test_dynamic_layer_mask_toggle() {
    let mut app = App::test();
    let layer_a = LayerMask::DEFAULT;
    let layer_b = LayerMask(2);

    let observer = app.agent(|c| c.neighbour(layer_a, Vec3::splat(5.0)));
    let target = app.agent(|c| {
        c.neighbour(layer_a, Vec3::splat(5.0))
            .insert(Transform::from_translation(Vec3::new(2.0, 0.0, 0.0)))
    });

    app.step_frame();
    app.step_frame();

    let neighbours = app
        .get::<ONeighbours>(observer)
        .targets()
        .collect::<Vec<_>>();

    assert_eq!(neighbours, vec![target]);

    // Switch target to non-matching layer
    app.world_mut()
        .entity_mut(target)
        .insert(CollisionLayers::new(layer_b, LayerMask::ALL));
    app.step_frame();
    assert!(app.world().get::<ONeighbours>(observer).is_none());

    // Switch back to matching layer
    app.world_mut()
        .entity_mut(target)
        .insert(CollisionLayers::new(layer_a, LayerMask::ALL));
    app.step_frame();

    let neighbours = app
        .get::<ONeighbours>(observer)
        .targets()
        .collect::<Vec<_>>();

    assert_eq!(neighbours, vec![target]);
}
