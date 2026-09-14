mod shared;

use bevy_many_relationships::OutgoingRelationships;
use bevy_context_steering::{Obstacle, ObstacleDetection, ObstacleFilter};
use shared::*;
use test_case::test_case;

type DetectedObstacles = OutgoingRelationships<Obstacle>;

fn detector(app: &mut App, mask: LayerMask, max_distance: f32) -> Entity {
    app.agent(|commands| {
        commands.insert((
            ObstacleFilter(mask),
            ObstacleDetection::new(
                Collider::sphere(0.5),
                ShapeCastConfig {
                    max_distance,
                    ..default()
                },
            )
            .with_direction(Dir3::X),
        ))
    })
}

fn obstacle(app: &mut App, position: Vec3, layers: CollisionLayers) -> Entity {
    let mut commands = app.world_mut().commands();
    let entity = commands
        .spawn_empty()
        .obstacle()
        .insert((Transform::from_translation(position), layers))
        .id();
    app.world_mut().flush();
    entity
}

fn detected(app: &App, entity: Entity) -> Vec<Entity> {
    app.get::<DetectedObstacles>(entity)
        .targets()
        .collect()
}

#[test_case(2.0, 5.0; "near obstacle")]
#[test_case(4.0, 5.0; "far obstacle within range")]
fn detects_obstacle_in_cast_direction(position: f32, max_distance: f32) {
    let mut app = App::test();
    let detector = detector(&mut app, LayerMask::DEFAULT, max_distance);
    let obstacle = obstacle(
        &mut app,
        Vec3::new(position, 0.0, 0.0),
        CollisionLayers::new(LayerMask::DEFAULT, LayerMask::ALL),
    );

    app.step_frame();
    app.step_frame();

    assert_eq!(detected(&app, detector), vec![obstacle]);
}

#[test_case(3.0, 1.0; "just beyond range")]
#[test_case(10.0, 5.0; "well beyond range")]
fn ignores_obstacle_beyond_cast_distance(position: f32, max_distance: f32) {
    let mut app = App::test();
    let detector = detector(&mut app, LayerMask::DEFAULT, max_distance);
    let _obstacle = obstacle(
        &mut app,
        Vec3::new(position, 0.0, 0.0),
        CollisionLayers::new(LayerMask::DEFAULT, LayerMask::ALL),
    );

    app.step_frame();
    app.step_frame();
    app.step_frame();

    assert!(app.world().get::<DetectedObstacles>(detector).is_none());
}

#[test]
fn ignores_obstacle_on_filtered_layer() {
    let mut app = App::test();
    let detector = detector(&mut app, LayerMask::DEFAULT, 5.0);
    let filtered_layer = LayerMask(2);
    let _obstacle = obstacle(
        &mut app,
        Vec3::new(3.0, 0.0, 0.0),
        CollisionLayers::new(filtered_layer, LayerMask::ALL),
    );

    app.step_frame();
    app.step_frame();

    assert!(app.world().get::<DetectedObstacles>(detector).is_none());
}

#[test]
fn removes_obstacle_relationship_when_obstacle_leaves_range() {
    let mut app = App::test();
    let detector = detector(&mut app, LayerMask::DEFAULT, 5.0);
    let obstacle = obstacle(
        &mut app,
        Vec3::new(3.0, 0.0, 0.0),
        CollisionLayers::new(LayerMask::DEFAULT, LayerMask::ALL),
    );

    app.step_frame();
    app.step_frame();
    assert_eq!(detected(&app, detector), vec![obstacle]);

    app.world_mut()
        .entity_mut(obstacle)
        .insert(Transform::from_translation(Vec3::new(20.0, 0.0, 0.0)));
    app.step_frame();
    app.step_frame();
    app.step_frame();

    assert!(app.world().get::<DetectedObstacles>(detector).is_none());
}
