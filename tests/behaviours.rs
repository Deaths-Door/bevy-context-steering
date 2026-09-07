mod shared;

use shared::*;
use test_case::test_case;

const ALIGNMENT_THRESHOLD: f32 = 0.97;

fn assert_alignment(alignment: f32) {
    assert!(
        alignment > ALIGNMENT_THRESHOLD,
        "Agent velocity misaligned! Dot: {}, expected > {}",
        alignment,
        ALIGNMENT_THRESHOLD
    )
}

#[test_case(Vec3::X * 10.0, Falloff::None; "Seek 1 - Pure X (None)")]
#[test_case(Vec3::Y * 15.0, Falloff::Linear { threshold: 20.0 }; "Seek 2 - Pure Y (Linear)")]
#[test_case(Vec3::Z * 12.0, Falloff::Quadratic { threshold: 15.0 }; "Seek 3 - Pure Z (Quadratic)")]
#[test_case(vec3(-10.0, 2.0, -5.0), Falloff::Cubic { threshold: 12.0 }; "Seek 4 - Off-axis Negative (Cubic)")]
#[test_case(vec3(0.1, 10.0, 0.1), Falloff::SmoothStep { threshold: 10.0 }; "Seek 5 - Near-Pole (SmoothStep)")]
#[test_case(vec3(-15.0, -15.0, 0.0), Falloff::SmootherStep { threshold: 25.0 }; "Seek 6 - Lower Quadrant (SmootherStep)")]
#[test_case(vec3(1.0, 0.0, 100.0), Falloff::InverseSquare { threshold: 50.0 }; "Seek 7 - Extreme Z-Tilt (InverseSquare)")]
#[test_case(vec3(-5.0, 0.001, 0.0), Falloff::Exponential { threshold: 10.0, exponent: 2.0 }; "Seek 8 - Near-Axis (Exponential)")]
#[test_case(vec3(0.0, 0.0, 0.0), Falloff::Stop { threshold: 5.0 }; "Seek 9 - Already At Target (Stay In Place)")]
#[test_case(vec3(7.32, -4.15, 9.88), Falloff::None; "Seek 10 - Randomized Noise (None)")]
fn test_seek(target_pos: Vec3, falloff: Falloff) {
    let mut app = App::test();

    let agent_id = app.spawn_agent(|mut commands| {
        commands.insert(Seek::new(target_pos).with_falloff(falloff.clone()));
    });

    // 1. Capture Pre-Step State
    let initial_pos = app.get::<Transform>(agent_id).translation;
    let initial_dist = initial_pos.distance(target_pos);

    let should_seek = {
        // 1. Check if agent is already at the target location (distance ~ 0)
        let is_already_at_target = initial_dist < f32::EPSILON;

        // 2. Check if falloff is specifically a Stop variant that halts steering outside threshold
        let is_stopped_by_falloff = match falloff {
            Falloff::Stop { threshold } => initial_dist > threshold,
            _ => false, // All other falloff curves (SmoothStep, Linear, InverseSquare, etc.) still seek!
        };

        // 3. Agent should seek ONLY if not already at target AND not hard-stopped by falloff
        !is_already_at_target && !is_stopped_by_falloff
    };

    // 2. Step the simulation
    app.step();

    let current_pos = app.get::<Transform>(agent_id).translation;
    let current_vel = **app.get::<LinearVelocity>(agent_id);

    match should_seek {
        true => {
            let target_dir = (target_pos - initial_pos).normalize_or_zero();

            // A. Alignment Check
            let vel_dir = current_vel.normalize_or_zero();
            let alignment = vel_dir.dot(target_dir);
            assert_alignment(alignment);

            // B. Distance Decreased
            let new_dist = current_pos.distance(target_pos);
            assert!(
                new_dist < initial_dist,
                "Agent failed to seek! Initial dist: {}, New dist: {}",
                initial_dist,
                new_dist
            );
        }
        false => {
            let pos_delta = current_pos.distance(initial_pos);
            let vel_mag = current_vel.length();

            assert!(
                pos_delta < MOVEMENT_TOLERANCE,
                "Agent moved when it should have stayed in place! Delta: {}",
                pos_delta
            );

            assert!(
                vel_mag < f32::EPSILON,
                "Agent gained velocity when it should have stayed in place! Velocity: {}",
                vel_mag
            );
        }
    }
}

#[test_case(vec3(10.0, 0.0, 0.0), Falloff::None; "Flee 1 - Basic Flee (No Falloff)")]
#[test_case(vec3(0.0, 0.0, 10.0), Falloff::None; "Flee 2 - Basic Flee Z Axis")]
#[test_case(vec3(5.0, 0.0, 5.0), Falloff::Linear { threshold: 10.0 }; "Flee 3 - Linear Falloff Within Radius")]
#[test_case(vec3(5.0, 0.0, 5.0), Falloff::SmoothStep { threshold: 10.0 }; "Flee 4 - SmoothStep Falloff Within Radius")]
#[test_case(vec3(5.0, 0.0, 5.0), Falloff::InverseSquare { threshold: 10.0 }; "Flee 5 - InverseSquare Falloff Within Radius")]
#[test_case(vec3(100.0, 0.0, 0.0), Falloff::Stop { threshold: 5.0 }; "Flee 6 - Outside Stop Threshold (No Reaction)")]
#[test_case(vec3(2.0, 0.0, 0.0), Falloff::Stop { threshold: 5.0 }; "Flee 7 - Inside Stop Threshold (Flee)")]
#[test_case(vec3(3.0, 4.0, 0.0), Falloff::Linear { threshold: 10.0 }; "Flee 8 - Diagonal Direction")]
#[test_case(vec3(0.0, 0.0, 0.0), Falloff::Stop { threshold: 5.0 }; "Flee 9 - Already At Target (Stay In Place)")]
fn test_flee(target_pos: Vec3, falloff: Falloff) {
    let mut app = App::test();

    let agent_id = app.spawn_agent(|mut commands| {
        commands.insert(Flee::new(target_pos).with_falloff(falloff.clone()));
    });

    // 1. Capture Pre-Step State
    let initial_pos = app.get::<Transform>(agent_id).translation;
    let initial_dist = initial_pos.distance(target_pos);

    let should_flee = {
        // 1. Check if agent is already at the target location (distance ~ 0).
        //    Direction away from target is undefined at zero distance.
        let is_already_at_target = initial_dist < f32::EPSILON;

        // 2. Check if falloff is specifically a Stop variant that halts steering
        //    outside the threshold (i.e. threat too far away to react to).
        let is_stopped_by_falloff = match falloff {
            Falloff::Stop { threshold } => initial_dist > threshold,
            _ => false, // All other falloff curves still flee!
        };

        // 3. Agent should flee ONLY if not already at target AND not hard-stopped by falloff
        !is_already_at_target && !is_stopped_by_falloff
    };

    // 2. Step the simulation
    app.step();

    let current_pos = app.get::<Transform>(agent_id).translation;
    let current_vel = **app.get::<LinearVelocity>(agent_id);

    match should_flee {
        true => {
            // Flee direction is AWAY from target — opposite of seek.
            let target_dir = (initial_pos - target_pos).normalize_or_zero();

            // A. Alignment Check
            let vel_dir = current_vel.normalize_or_zero();
            let alignment = vel_dir.dot(target_dir);
            assert_alignment(alignment);

            // B. Distance Increased (fleeing moves AWAY, so distance grows)
            let new_dist = current_pos.distance(target_pos);
            assert!(
                new_dist > initial_dist,
                "Agent failed to flee! Initial dist: {}, New dist: {}",
                initial_dist,
                new_dist
            );
        }
        false => {
            let pos_delta = current_pos.distance(initial_pos);
            let vel_mag = current_vel.length();

            assert!(
                pos_delta < MOVEMENT_TOLERANCE,
                "Agent moved when it should have stayed in place! Delta: {}",
                pos_delta
            );

            assert!(
                vel_mag < f32::EPSILON,
                "Agent gained velocity when it should have stayed in place! Velocity: {}",
                vel_mag
            );
        }
    }
}

use bevy::math::vec3;
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
    let mut app = App::test();

    let target_id = app.spawn_agent(|mut commands| {
        commands.insert(Transform::from_translation(target_pos));
    });

    let agent_id = app.spawn_agent(|mut commands| {
        commands.insert((
            Transform::from_translation(agent_pos),
            Pursuit::new(target_id),
        ));
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

        app.step();

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

fn test_evade(agent_pos: Vec3, threat_pos: Vec3, velocities: &[Vec3]) {
    let mut app = App::test();

    let threat_id = app.spawn_agent(|mut commands| {
        commands.insert(Transform::from_translation(threat_pos));
    });

    let agent_id = app.spawn_agent(|mut commands| {
        commands.insert((
            Transform::from_translation(agent_pos),
            Evade::new(threat_id),
        ));
    });

    for &velocity in velocities.iter() {
        let mut vel = app
            .world_mut()
            .get_mut::<LinearVelocity>(threat_id)
            .unwrap();
        vel.0 = velocity;

        app.step();

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

#[test_case(vec3(10.0, 0.0, 0.0); "Brake - Pure X Velocity")]
#[test_case(vec3(-5.0, 2.5, 12.0); "Brake - Arbitrary 3D Velocity")]
fn test_brake(initial_velocity: Vec3) {
    let mut app = App::test();

    let agent_id = app.spawn_agent(|mut commands| {
        commands.insert((LinearVelocity(initial_velocity), Brake));
    });

    app.step();
    app.step();
    app.step();

    let agent_velocity = app.world().get::<LinearVelocity>(agent_id).unwrap();

    // Asserts that braking actively reduces/opposes initial linear velocity
    assert!(
        agent_velocity.length() < initial_velocity.length(),
        "Brake should reduce agent's linear velocity magnitude"
    );
}

#[test_case(vec3(15.0, 0.0, 0.0), vec3(0.0, 0.0, 0.0); "Throttle - Accelerate from Stationary")]
#[test_case(vec3(5.0, 0.0, 5.0), vec3(10.0, 0.0, 0.0); "Throttle - Match Moving Target Velocity")]
fn test_throttle(target_velocity: Vec3, agent_initial_velocity: Vec3) {
    let mut app = App::test();

    let target_id = app.spawn_agent(|mut commands| {
        commands.insert(LinearVelocity(target_velocity));
    });

    let agent_id = app.spawn_agent(|mut commands| {
        commands.insert((
            LinearVelocity(agent_initial_velocity),
            Throttle::new(target_id),
        ));
    });

    app.step();
    app.step();
    app.step();
    app.step();

    let agent_velocity = **app.world().get::<LinearVelocity>(agent_id).unwrap();

    // 1. Verify direction matches (dot product close to 1.0)
    if target_velocity.length_squared() > 0.001 {
        let dir_similarity = agent_velocity.normalize().dot(target_velocity.normalize());
        assert!(
            dir_similarity > 0.9,
            "Expected agent velocity direction to align with target velocity. Dot product: {dir_similarity}"
        );
    }

    // 2. Verify agent accelerated significantly toward target speed
    let speed_diff = (agent_velocity.length() - target_velocity.length()).abs();
    assert!(
        speed_diff < 1.0,
        "Expected agent speed to approach target speed ({}), but got {}",
        target_velocity.length(),
        agent_velocity.length()
    );
}
#[test_case(
    Vec3::ZERO, 
    Vec3::splat(5.0),
    &[], 
    false 
    ; "no_neighbors_should_not_move"
)]
#[test_case(
    Vec3::ZERO, 
    Vec3::splat(5.0),
    &[Vec3::new(3.0, 0.0, 0.0)], 
    true 
    ; "single_neighbor_inside_range"
)]
#[test_case(
    Vec3::ZERO, 
    Vec3::splat(5.0),
    &[Vec3::new(10.0, 0.0, 0.0)], 
    false 
    ; "single_neighbor_outside_range_ignored"
)]
#[test_case(
    Vec3::ZERO, 
    Vec3::splat(5.0),
    &[Vec3::new(3.0, 2.0, 0.0), Vec3::new(3.0, -2.0, 0.0)], 
    true 
    ; "asymmetric_cluster_inside_range"
)]
#[test_case(
    Vec3::ZERO, 
    Vec3::splat(5.0),
    &[
        Vec3::new(3.0, 0.0, 0.0), 
        Vec3::new(-3.0, 0.0, 0.0),
        Vec3::new(0.0, 3.0, 0.0), 
        Vec3::new(0.0, -3.0, 0.0)
    ], 
    false 
    ; "symmetric_surrounding_neighbors_equilibrium"
)]
#[test_case(
    Vec3::ZERO, 
    Vec3::splat(15.0),
    &[Vec3::new(10.0, 0.0, 0.0)], 
    true 
    ; "neighbor_detected_with_larger_custom_half_extent"
)]
fn test_cohere(
    agent_translation: Vec3, 
    half_extent: Vec3,
    neighbours: &[Vec3], 
    should_move: bool
) {
    let mut app = App::test();

    let agent = app.agent(|commands| {
        commands
            .neighbour(LayerMask::DEFAULT,half_extent)
            .insert((
                Cohere::new(),
                Transform::from_translation(agent_translation),
            ))
    });

    neighbours.iter().for_each(|translation| {
        app.agent(|commands| commands.insert(Transform::from_translation(*translation)));
    });

    app.step();

    let new_translation = app.get::<Transform>(agent).translation;

    if should_move {
        // Guard against empty neighbours to prevent panic/division by zero
        assert!(
            !neighbours.is_empty(),
            "should_move was true, but neighbours slice was empty!"
        );

        // Invariant 1: Agent MUST move
        assert_ne!(
            new_translation, agent_translation,
            "Agent should steer when neighbors exist"
        );

        let displacement = new_translation - agent_translation;

        // Calculate general target direction using centroid half-space
        let centre = neighbours.iter().sum::<Vec3>() / neighbours.len() as f32;
        let target_dir = (centre - agent_translation).normalize_or_zero();

        // Invariant 2: Directional alignment threshold
        let dot = displacement.normalize_or_zero().dot(target_dir);

        assert!(
            dot > 0.7,
            "Agent steering misaligned with flock center. Expected dot > 0.7, got {dot:.2}"
        );
    } else {
        // Invariant 3: Agent should stay stationary (e.g. no neighbors or balanced forces)
        assert_eq!(
            new_translation, agent_translation,
            "Agent moved when it should have stayed stationary"
        );
    }
}
