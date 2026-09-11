use bevy::math::Vec3;
use bevy_context_steering::SteeringContext;

struct Behaviour;
struct Behaviour2;

trait ContextTesting {
    fn test() -> Self;
    fn check_direction(&mut self, expected: Vec3) -> &mut Self;
}

impl ContextTesting for SteeringContext {
    fn test() -> Self {
        let mut context = SteeringContext::default();
        context.insert::<Behaviour>();
        context.insert::<Behaviour2>();
        context
    }
    fn check_direction(&mut self, expected: Vec3) -> &mut Self {
        let resultant_direction = self.resultant_direction();
        assert_approx_dir(resultant_direction, expected, 0.98);
        self
    }
}

fn assert_approx_dir(actual: Vec3, expected: Vec3, threshold: f32) {
    let dot = actual.normalize_or_zero().dot(expected.normalize_or_zero());
    assert!(
        dot >= threshold,
        "Directions do not match! \nActual:   {:?}\nExpected: {:?}\nDot:      {}",
        actual,
        expected,
        dot
    );
}

#[test]
fn interest_only() {
    let mut context = SteeringContext::test();

    let intended_diretion = context.cache.directions()[0];
    let expected_direction = intended_diretion;

    context.set_interest::<Behaviour>(intended_diretion);

    context.update();
    context.check_direction(expected_direction);
}

#[test]
fn danger_only() {
    let mut context = SteeringContext::test();

    let intended_diretion = context.cache.directions()[0];
    let expected_direction = -intended_diretion;

    context.set_danger::<Behaviour>(intended_diretion);

    context.update();
    context.check_direction(expected_direction);
}

#[test]
fn nullify_singular_direction() {
    let north = Vec3::Y;
    let east = Vec3::X;

    let mut context = SteeringContext::test();
    context.set_interest::<Behaviour>(north);

    // Danger exactly cancels the interest at North
    context.set_danger::<Behaviour>(north);
    context.set_interest::<Behaviour2>(east);

    let expected_direction = east;

    context.update();
    context.check_direction(expected_direction);
}

#[test]
fn interpolation() {
    let mut context = SteeringContext::test();

    let expected_direction = {
        let cache = &context.cache;
        let center_idx = cache.directions().len() / 2;
        let neighbor_idx = cache.direction_neighbours()[center_idx][1];
        let off_grid_target =
            (cache.directions()[center_idx] + cache.directions()[neighbor_idx]).normalize();
        off_grid_target
    };

    context.set_interest::<Behaviour>(expected_direction);

    context.update();
    context.check_direction(expected_direction);
}

#[test]
fn weighted_behaviours() {
    let mut context = SteeringContext::test();

    let north = Vec3::Y;
    let east = Vec3::X;

    let weight_north = 2.0;
    let weight_east = 1.0;

    // Insert and configure first behavior
    context.set_interest::<Behaviour>(north);
    context.set_weight::<Behaviour>(weight_north);

    context.set_interest::<Behaviour2>(east);
    context.set_weight::<Behaviour2>(weight_east);

    // The expected direction is the weighted vector sum:
    // (2.0 * North) + (1.0 * East) normalized.
    let expected = (north * weight_north + east * weight_east).normalize();
    context.update();
    context.check_direction(expected);

    let result = context.resultant_direction();
    // Final sanity check: The result should be closer to North than to East
    assert!(result.dot(north) > result.dot(east));
}

/// Demonstrates `set_velocity` mapping continuous directions to the nearest slot
/// and ensuring the velocity is recovered near the interest peak.
#[test]
fn velocity_by_direction_matches_interest_peak() {
    let mut context = SteeringContext::default();
    context.insert::<Behaviour>();

    let direction = context.cache.directions()[0];
    let target_velocity = direction * 3.5;

    // Set interest and velocity using continuous direction vectors
    context.set_interest::<Behaviour>(direction);
    context.set_velocity::<Behaviour>(direction, target_velocity);
    context.update();

    let resolved = context
        .resultant_velocity()
        .expect("expected a resolved velocity");
    assert!(
        (resolved - target_velocity).length() < 1e-3,
        "expected {target_velocity:?}, got {resolved:?}"
    );
}


/// A slot with interest but no velocity write resolves to None ("no opinion").
#[test]
fn interest_without_velocity_resolves_to_none() {
    let mut context = SteeringContext::default();
    context.insert::<Behaviour>();

    let direction = context.cache.directions()[0];
    context.set_interest::<Behaviour>(direction);
    context.update();

    assert_eq!(
        context.resultant_velocity(),
        None,
        "expected None when no behaviour wrote a velocity preference"
    );
}

/// An explicit zero velocity write ("explicit stop") resolves to Some(Vec3::ZERO).
#[test]
fn explicit_zero_velocity_is_some_zero_not_none() {
    let mut context = SteeringContext::default();
    context.insert::<Behaviour>();

    let direction = context.cache.directions()[0];

    context.set_interest::<Behaviour>(direction);
    context.set_velocity::<Behaviour>(direction, Vec3::ZERO);
    context.update();

    assert_eq!(
        context.resultant_velocity(),
        Some(Vec3::ZERO),
        "explicit zero-velocity write should resolve as Some(ZERO), not None"
    );
}
