use super::*;
use std::ops::Deref;

pub(super) fn debug_steering_context(
    mut gizmos: Gizmos,
    query: ActiveAgentsQuery<
        (
            &SteeringContext,
            &SteeringDebugOptions,
            &Transform,
            &LinearVelocity,
        ),
        With<EnableSteeringDebug>,
    >,
) {
    for (context, options, transform, velocity) in query.iter() {
        let start = transform.translation;
        let mut draw_arrow = |end: Vec3, color: Color| {
            gizmos.arrow(start, end, color);
        };

        if let Some((color, length)) = options.resultant_direction.using_weight(1.0) {
            let dir = context.resultant_direction();
            let end = start + dir * length;
            draw_arrow(end, color);
        }

        if let Some((color, length)) = options.resultant_velocity.using_weight(1.0) {
            let target_velocity = context.resultant_velocity().unwrap_or(**velocity);
            let dir = target_velocity.normalize_or_zero();
            let end = start + dir * length;
            draw_arrow(end, color)
        }

        for (type_id, style) in options.behaviors.iter().filter(|(_, style)| style.enabled) {
            let Some(behaviour) = context.deref().get(type_id) else {
                continue;
            };

            for (weight, direction) in behaviour.field().iter().zip(context.cache.directions()) {
                // Transform local direction vector to world orientation (ignoring entity scale/translation)
                let world_dir = transform.rotation * *direction;

                // Interest Channel
                if let Some((color, length)) = style.interest.using_weight(weight.interest()) {
                    let end = start + world_dir * length;
                    draw_arrow(end, color)
                }

                // Danger Channel
                if let Some((color, length)) = style.danger.using_weight(weight.danger()) {
                    let end = start + world_dir * length;
                    draw_arrow(end, color)
                }

                // Velcoity Channel
                if let Some((velocity, (color, length))) =
                    weight.velocity().zip(style.velocity.using_weight(1.0))
                {
                    let dir = (transform.rotation * velocity).normalize_or_zero();
                    let end = start + dir * length;
                    draw_arrow(end, color)
                }
            }
        }
    }
}
