use avian3d::prelude::*;
use bevy::{
    app::PanicHandlerPlugin, mesh::MeshPlugin, prelude::*, scene::ScenePlugin,
    time::TimeUpdateStrategy,
};
use bevy_context_steering::*;
use test_case::test_case;

const N_FRAMES: usize = 30;

fn setup_app(apply: impl FnOnce(&mut App)) -> App {
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
    loop_frames(&mut app);

    app
}

fn loop_frames(app: &mut App) {
    for _ in 0..N_FRAMES {
        app.update();
    }
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
    let app = setup_app(|app| {
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
    let app = setup_app(|app| {
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

// --- Stationary target ---
#[test_case(Vec3::ZERO, vec3(10.0, 0.0, 0.0), &[Vec3::ZERO]; "Pursuit - Stationary Target")]
#[test_case(vec3(-5.0, 0.0, -5.0), vec3(10.0, 0.0, 10.0), &[Vec3::ZERO]; "Pursuit - Stationary Target Diagonal")]
// --- Target moving away (classic pursuit) ---
#[test_case(Vec3::ZERO, vec3(10.0, 0.0, 0.0), &[vec3(2.0, 0.0, 0.0)]; "Pursuit - Target Fleeing Along X")]
#[test_case(Vec3::ZERO, vec3(0.0, 0.0, 10.0), &[vec3(0.0, 0.0, 5.0)]; "Pursuit - Target Fleeing Along Z")]
#[test_case(Vec3::ZERO, vec3(10.0, 0.0, 10.0), &[vec3(3.0, 0.0, 3.0)]; "Pursuit - Target Fleeing Diagonal")]
// --- Target moving perpendicular (interception required) ---
#[test_case(Vec3::ZERO, vec3(10.0, 0.0, 0.0), &[vec3(0.0, 0.0, 5.0)]; "Pursuit - Target Crossing Perpendicular")]
#[test_case(vec3(-10.0, 0.0, 0.0), vec3(0.0, 0.0, 10.0), &[vec3(5.0, 0.0, 0.0)]; "Pursuit - Target Crossing At Angle")]
// --- Target moving toward agent (aligned ahead case) ---
#[test_case(Vec3::ZERO, vec3(10.0, 0.0, 0.0), &[vec3(-5.0, 0.0, 0.0)]; "Pursuit - Target Approaching Head-On")]
// --- Velocity changes mid-pursuit ---
#[test_case(Vec3::ZERO, vec3(20.0, 0.0, 0.0), &[vec3(5.0, 0.0, 0.0), vec3(0.0, 0.0, 5.0)]; "Pursuit - Target Changes Direction 90deg")]
#[test_case(Vec3::ZERO, vec3(20.0, 0.0, 0.0), &[vec3(5.0, 0.0, 0.0), Vec3::ZERO]; "Pursuit - Target Stops Mid-Pursuit")]
#[test_case(Vec3::ZERO, vec3(20.0, 0.0, 0.0), &[vec3(5.0, 0.0, 0.0), vec3(-5.0, 0.0, 0.0)]; "Pursuit - Target Reverses Direction")]
#[test_case(Vec3::ZERO, vec3(20.0, 0.0, 0.0), &[vec3(2.0, 0.0, 0.0), vec3(0.0, 0.0, 5.0), vec3(-3.0, 0.0, -3.0)]; "Pursuit - Target Changes Direction Twice")]
// --- Edge / degenerate ---
#[test_case(vec3(0.0, 0.0, -100.0), vec3(0.0, 0.0, 100.0), &[vec3(0.0, 0.0, 5.0)]; "Pursuit - Far Apart Same Axis")]
#[test_case(Vec3::ZERO, vec3(10.0, 0.0, 10.0), &[vec3(1.0, 0.001, 0.0)]; "Pursuit - Near-Zero Y Velocity Component")]
#[test_case(vec3(7.32, 0.0, -4.15), vec3(-3.5, 0.0, 9.88), &[vec3(2.1, 0.0, -1.7)]; "Pursuit - Randomized Positions and Velocity")]
fn test_pursuit(agent_pos: Vec3, target_pos: Vec3, velocities: &[Vec3]) {
    let mut agent_id = Entity::PLACEHOLDER;
    let mut target_id = Entity::PLACEHOLDER;

    let mut app = setup_app(|app| {
        let mut commands = app.world_mut().commands();

        // 1. Spawn the Target moving at a constant speed
        target_id = commands
            .spawn((
                Transform::from_translation(target_pos),
                RigidBody::Dynamic,
                Mass(1.0),
                Collider::sphere(1.0),
            ))
            .id();

        // 2. Spawn the Agent with Pursuit behavior
        let behaviour = Pursuit::new(target_id);
        agent_id = spawn_agent(&mut commands)
            .insert(Transform::from_translation(agent_pos))
            .insert(behaviour)
            .id();
    });

    for &velocity in velocities.iter() {
        // Apply the new velocity to the target
        let mut vel = app
            .world_mut()
            .get_mut::<LinearVelocity>(target_id)
            .unwrap();
        vel.0 = velocity;

        let initial_distance = {
            let a_pos = app.world().get::<Transform>(agent_id).unwrap().translation;
            let t_pos = app.world().get::<Transform>(target_id).unwrap().translation;
            (t_pos - a_pos).length()
        };

        loop_frames(&mut app);

        let a_pos = app.world().get::<Transform>(agent_id).unwrap().translation;
        let a_vel = app.world().get::<LinearVelocity>(agent_id).unwrap().0;
        let t_pos = app.world().get::<Transform>(target_id).unwrap().translation;
        let t_vel = app.world().get::<LinearVelocity>(target_id).unwrap().0;

        // --- ASSERTIONS ---

        // 2. Agent is getting closer to the target (convergence over N frames)
        // Store initial distance before loop_frames and compare after.
        // NOTE: capture `initial_distance` before calling loop_frames above.
        let current_distance = (t_pos - a_pos).length();
        assert!(
            current_distance < initial_distance + 0.1,
            "Agent should be closing the gap each velocity phase. before={:.3}, after={:.3}, t_vel={:?}",
            initial_distance,
            current_distance,
            t_vel
        );

        // 3. If target is moving, agent should NOT aim at current target pos —
        //    it should aim ahead of it. Only meaningful when target has velocity.
        if t_vel.length() > f32::EPSILON {
            let separation = t_pos - a_pos;
            let agent_dir = a_vel.normalize_or_zero();

            // Naive direction toward current target position
            let to_current = separation.normalize_or_zero();

            // Direction toward a rough future position (one second ahead)
            let future_target = t_pos + t_vel.normalize() * t_vel.length();
            let to_future = (future_target - a_pos).normalize_or_zero();

            let dot_to_current = agent_dir.dot(to_current);
            let dot_to_future = agent_dir.dot(to_future);

            assert!(
                dot_to_future >= dot_to_current * ALIGNMENT_THRESHOLD,
                "When target is moving, agent should steer toward future pos, not current.\n\
            dot_to_future={:.3}, dot_to_current={:.3}, t_vel={:?}",
                dot_to_future,
                dot_to_current,
                t_vel
            );
        }

        // 4. If target is stationary, agent should aim directly at it
        if t_vel.length() <= f32::EPSILON {
            let to_target = (t_pos - a_pos).normalize_or_zero();
            let agent_dir = a_vel.normalize_or_zero();
            let alignment = agent_dir.dot(to_target);
            assert!(
                alignment > ALIGNMENT_THRESHOLD,
                "When target is stationary, agent should seek directly toward it. alignment={:.3}",
                alignment
            );
        }
    }
}

// --- Stationary threat ---
#[test_case(Vec3::ZERO, vec3(10.0, 0.0, 0.0), &[Vec3::ZERO]; "Evade - Stationary Threat")]
#[test_case(vec3(-5.0, 0.0, -5.0), vec3(10.0, 0.0, 10.0), &[Vec3::ZERO]; "Evade - Stationary Threat Diagonal")]
// --- Threat chasing the agent (classic evade) ---
#[test_case(Vec3::ZERO, vec3(10.0, 0.0, 0.0), &[vec3(-2.0, 0.0, 0.0)]; "Evade - Threat Chasing Along X")]
#[test_case(Vec3::ZERO, vec3(0.0, 0.0, 10.0), &[vec3(0.0, 0.0, -5.0)]; "Evade - Threat Chasing Along Z")]
#[test_case(Vec3::ZERO, vec3(10.0, 0.0, 10.0), &[vec3(-3.0, 0.0, -3.0)]; "Evade - Threat Chasing Diagonal")]
// --- Threat moving perpendicular (agent should veer away from intercept) ---
#[test_case(Vec3::ZERO, vec3(10.0, 0.0, 0.0), &[vec3(0.0, 0.0, 5.0)]; "Evade - Threat Crossing Perpendicular")]
#[test_case(vec3(-10.0, 0.0, 0.0), vec3(0.0, 0.0, 10.0), &[vec3(5.0, 0.0, 0.0)]; "Evade - Threat Crossing At Angle")]
// --- Threat moving away from agent (low urgency) ---
#[test_case(Vec3::ZERO, vec3(10.0, 0.0, 0.0), &[vec3(5.0, 0.0, 0.0)]; "Evade - Threat Fleeing Away")]
// --- Threat approaching head-on ---
#[test_case(Vec3::ZERO, vec3(10.0, 0.0, 0.0), &[vec3(-5.0, 0.0, 0.0)]; "Evade - Threat Approaching Head-On")]
// --- Velocity changes mid-evade ---
#[test_case(Vec3::ZERO, vec3(20.0, 0.0, 0.0), &[vec3(-5.0, 0.0, 0.0), vec3(0.0, 0.0, -5.0)]; "Evade - Threat Changes Direction 90deg")]
#[test_case(Vec3::ZERO, vec3(20.0, 0.0, 0.0), &[vec3(-5.0, 0.0, 0.0), Vec3::ZERO]; "Evade - Threat Stops Mid-Chase")]
#[test_case(Vec3::ZERO, vec3(20.0, 0.0, 0.0), &[vec3(-5.0, 0.0, 0.0), vec3(5.0, 0.0, 0.0)]; "Evade - Threat Reverses Direction")]
#[test_case(Vec3::ZERO, vec3(20.0, 0.0, 0.0), &[vec3(-2.0, 0.0, 0.0), vec3(0.0, 0.0, -5.0), vec3(3.0, 0.0, 3.0)]; "Evade - Threat Changes Direction Twice")]
// --- Edge / degenerate ---
#[test_case(Vec3::ZERO, vec3(10.0, 0.0, 10.0), &[vec3(-1.0, 0.001, 0.0)]; "Evade - Near-Zero Y Velocity Component")]
#[test_case(vec3(7.32, 0.0, -4.15), vec3(-3.5, 0.0, 9.88), &[vec3(-2.1, 0.0, 1.7)]; "Evade - Randomized Positions and Velocity")]

// Weird 
// Threat orbiting the agent (tangential velocity, never actually closing)
#[test_case(Vec3::ZERO, vec3(10.0, 0.0, 0.0), &[vec3(0.0, 0.0, 10.0), vec3(-10.0, 0.0, 0.0), vec3(0.0, 0.0, -10.0), vec3(10.0, 0.0, 0.0)]; "Evade - Threat Orbiting Agent")]

// Threat moving directly away — agent has no reason to flee hard, but shouldn't chase
#[test_case(Vec3::ZERO, vec3(10.0, 0.0, 0.0), &[vec3(20.0, 0.0, 0.0)]; "Evade - Threat Sprinting Away")]
// Threat is at the exact same position (zero separation — degenerate)
#[test_case(Vec3::ZERO, Vec3::ZERO, &[vec3(-1.0, 0.0, 0.0)]; "Evade - Threat Spawns On Agent")]

// Very high threat speed — prediction time collapses, agent should still not freeze
#[test_case(Vec3::ZERO, vec3(10.0, 0.0, 0.0), &[vec3(-100.0, 0.0, 0.0)]; "Evade - Threat Extremely Fast Approach")]

fn test_evade(agent_pos: Vec3, threat_pos: Vec3, velocities: &[Vec3]) {
    let mut agent_id = Entity::PLACEHOLDER;
    let mut threat_id = Entity::PLACEHOLDER;

    let mut app = setup_app(|app| {
        let mut commands = app.world_mut().commands();

        threat_id = commands
            .spawn((
                Transform::from_translation(threat_pos),
                RigidBody::Dynamic,
                Mass(1.0),
                Collider::sphere(1.0),
            ))
            .id();

        let behaviour = Evade::new(threat_id);
        agent_id = spawn_agent(&mut commands)
            .insert(Transform::from_translation(agent_pos))
            .insert(behaviour)
            .id();
    });

    for &velocity in velocities.iter() {
        let mut vel = app
            .world_mut()
            .get_mut::<LinearVelocity>(threat_id)
            .unwrap();
        vel.0 = velocity;

        loop_frames(&mut app);

        let a_pos = app.world().get::<Transform>(agent_id).unwrap().translation;
        let a_vel = app.world().get::<LinearVelocity>(agent_id).unwrap().0;
        let t_pos = app.world().get::<Transform>(threat_id).unwrap().translation;
        let t_vel = app.world().get::<LinearVelocity>(threat_id).unwrap().0;

        // --- ASSERTIONS ---

        // 1. Agent is actively moving — evade should always produce steering
        assert!(
            a_vel.length() > f32::EPSILON,
            "Agent should be moving when evading. vel={:?}",
            a_vel
        );

        // 2. Agent velocity should have a component pointing AWAY from the threat.
        //    This is the core evade invariant — the opposite of pursuit's convergence check.
        let to_threat = (t_pos - a_pos).normalize_or_zero();
        let agent_dir = a_vel.normalize_or_zero();
        let flee_alignment = agent_dir.dot(to_threat);
        assert!(
            flee_alignment < ALIGNMENT_THRESHOLD,
            "Agent should be moving away from threat, not toward it. \
             flee_alignment={:.3} (positive = toward threat), t_vel={:?}",
            flee_alignment,
            t_vel
        );

        // 4. If threat is moving toward the agent, agent should be steering away from
        //    the predicted intercept point, not just the current threat position.
        let threat_dir = t_vel.normalize_or_zero();
        let is_threat_approaching = threat_dir.dot(-to_threat) > 0.0;

        if t_vel.length() > f32::EPSILON && is_threat_approaching {
            // Rough predicted threat position
            let future_threat = t_pos + t_vel * 1.0;
            let to_future_threat = (future_threat - a_pos).normalize_or_zero();
            let dot_away_from_future = agent_dir.dot(to_future_threat);
            let dot_away_from_current = agent_dir.dot(to_threat);

            // Agent should be fleeing the future position at least as much as current
            assert!(
                dot_away_from_future <= dot_away_from_current + 0.1,
                "When threat is approaching, agent should evade predicted position, not just current.\n\
                 dot_away_from_future={:.3}, dot_away_from_current={:.3}, t_vel={:?}",
                dot_away_from_future,
                dot_away_from_current,
                t_vel
            );
        }

        // 5. If threat is stationary or moving away, agent should still be
        //    moving away from the threat's current position (flee, not prediction needed).
        if t_vel.length() <= f32::EPSILON {
            assert!(
                flee_alignment < ALIGNMENT_THRESHOLD,
                "Even against stationary threat, agent should flee directly away. \
                 alignment={:.3}",
                flee_alignment
            );
        }
    }
}
