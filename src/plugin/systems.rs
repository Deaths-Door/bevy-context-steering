use super::*;

pub(super) fn update_resultant_field(mut query: ActiveAgentsQuery<&mut SteeringContext>) {
    query.par_iter_mut().for_each(|mut ctx| ctx.update());
}