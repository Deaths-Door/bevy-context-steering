use super::*;



pub(crate) type ActiveAgentsQuery<'w, 's, D, F = ()> = Query<'w, 's, D, (With<SteeringAgent>, F)>;

pub(super) fn update_resultant_field(mut query: ActiveAgentsQuery<&mut SteeringContext>) {
    query.par_iter_mut().for_each(|mut ctx| ctx.update());
}

#[derive(QueryData)]
#[query_data(mutable)]
pub(super) struct BehaviourForceQueryData {
    data: &'static SteeringAgent,
    context: &'static SteeringContext,
    mass: &'static ComputedMass,
    forces: Forces,
}

pub(super) fn apply_forces(mut query: ActiveAgentsQuery<BehaviourForceQueryData>) {
    query.par_iter_mut().for_each(|mut agent| {
        let target_heading = agent.context.resultant_direction();
        // TODO: figure how how to combien this...?
        let target_velocity = agent
            .context
            .resultant_velocity()
            .unwrap_or_else(|| target_heading * agent.data.max_speed);

        let current_velocity = agent.forces.linear_velocity();
        let desired_velocity = target_velocity - current_velocity;

        let mass = agent.mass.value();
        let k = agent.data.acceleration_wn * agent.data.acceleration_wn * mass;
        let mut force = desired_velocity * k;

        force = force.clamp(-agent.data.max_force, agent.data.max_force);
        agent.forces.apply_force(force);
    });
}
