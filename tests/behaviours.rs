use avian3d::prelude::*;
use bevy::{
    app::PanicHandlerPlugin, mesh::MeshPlugin, prelude::*, scene::ScenePlugin,
    time::TimeUpdateStrategy,
};
use bevy_context_steering::*;
use test_case::test_case;

fn setup_app<const N: usize>(apply: impl FnOnce(&mut App)) -> App {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        PanicHandlerPlugin,
        AssetPlugin::default(),
        TransformPlugin,
        MeshPlugin,
        ScenePlugin,
    ));

    app.add_plugins((PhysicsPlugins::default(), SteeringPlugin));

    app.insert_resource(Gravity::ZERO);
    app.insert_resource(TimeUpdateStrategy::ManualDuration(
        std::time::Duration::from_secs_f32(1.0 / 60.0),
    ));

    app.finish();
    app.cleanup();

    apply(&mut app);

    for _ in 0..N {
        app.update();
    }

    app
}

fn spawn_agent<'a>(commands: &'a mut Commands) -> EntityCommands<'a> {
    commands.spawn((
        RigidBody::Dynamic,
        Mass(1.0),
        Collider::sphere(1.0),
        SteeringAgent {
            max_speed: 5.0,
            max_force: Vec3::splat(50.0),
            acceleration_wn: 15.0,
            ..default()
        },
        // Crucial: Avian needs damping to stop the "wobble"
        LinearDamping(1.0),
        AngularDamping(1.0),
    ))
}

const ALIGNMENT_THRESHOLD: f32 = 0.97;

#[test_case(vec3(10.0, 0.0, 0.0);"Seek 1")] // Pure X
#[test_case(vec3(0.0, 15.0, 0.0);"Seek 2")] // Pure Y
#[test_case(vec3(0.0, 0.0, 12.0);"Seek 3")] // Pure Z
#[test_case(vec3(-10.0, 2.0, -5.0);"Seek 4")] // Off-axis Negative
#[test_case(vec3(0.1, 10.0, 0.1);"Seek 5")] // Near-Pole
#[test_case(vec3(-15.0, -15.0, 0.0);"Seek 6")] // Lower Quadrant
#[test_case(vec3(1.0, 0.0, 100.0);"Seek 7")] // Extreme Z-Tilt
#[test_case(vec3(-5.0, 0.001, 0.0);"Seek 8")] // Near-Axis
#[test_case(vec3(7.32, -4.15, 9.88);"Seek 9")] // Randomized "Noise"
fn test_seek(target_pos: Vec3) {
    let mut agent_id = Entity::PLACEHOLDER;
    let app = setup_app::<30>(|app| {
        let mut commands = app.world_mut().commands();

        let behaviour = Seek::new(target_pos.clone());
        agent_id = spawn_agent(&mut commands).insert(behaviour).id();
    });

    let target_dir = target_pos.normalize();
    let transform = app.world().get::<Transform>(agent_id).unwrap();
    let velocity = app.world().get::<LinearVelocity>(agent_id).unwrap().0;

    // 1. Check Velocity Alignment (The "Intent" check)
    let velocity_dir = velocity.normalize_or_zero();
    let alignment = velocity_dir.dot(target_dir);

    assert!(
        alignment > ALIGNMENT_THRESHOLD,
        "Velocity misaligned! Dot: {}",
        alignment
    );

    // 2. Check Angular Drift (The "Result" check)
    let current_pos = transform.translation;
    let progress = current_pos.dot(target_dir);
    let projected_point = target_dir * progress;
    let drift = current_pos.distance(projected_point);
    let drift_angle = (drift / progress).atan().to_degrees();

    assert!(
        drift_angle < 7.5,
        "Agent veered off course by {} degrees",
        drift_angle
    );
}

#[test_case(vec3(10.0, 0.0, 0.0);"Flee 1")] // Pure X
#[test_case(vec3(0.0, 15.0, 0.0);"Flee 2")] // Pure Y
#[test_case(vec3(0.0, 0.0, 12.0);"Flee 3")] // Pure Z
#[test_case(vec3(-10.0, 2.0, -5.0);"Flee 4")] // Off-axis Negative
#[test_case(vec3(0.1, 10.0, 0.1);"Flee 5")] // Near-Pole
#[test_case(vec3(-15.0, -15.0, 0.0);"Flee 6")] // Lower Quadrant
#[test_case(vec3(1.0, 0.0, 100.0);"Flee 7")] // Extreme Z-Tilt
#[test_case(vec3(-5.0, 0.001, 0.0);"Flee 8")] // Near-Axis
#[test_case(vec3(7.32, -4.15, 9.88);"Flee 9")] // Randomized "Noise"
fn test_flee(target_pos: Vec3) {
    let mut agent_id = Entity::PLACEHOLDER;
    let app = setup_app::<30>(|app| {
        let mut commands = app.world_mut().commands();

        let behaviour = Flee::new(target_pos.clone());
        agent_id = spawn_agent(&mut commands).insert(behaviour).id();
    });

    let velocity = app.world().get::<LinearVelocity>(agent_id).unwrap().0;

    let target_dir = -target_pos.normalize();

    // 1. Check Velocity Alignment (The "Intent" check)
    let velocity_dir = velocity.normalize_or_zero();
    let alignment = velocity_dir.dot(target_dir);

    assert!(
        alignment > ALIGNMENT_THRESHOLD,
        "Agent is not fleeing in the correct direction! Alignment: {}, Velocity: {:?}",
        alignment,
        velocity
    );
    assert!(
        alignment > ALIGNMENT_THRESHOLD,
        "Velocity misaligned! Dot: {}",
        alignment
    );
}
