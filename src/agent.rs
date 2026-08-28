use super::*;

#[derive(Component, Reflect)]
/*TODO:Neighborhood */
#[require(SteeringContext)]

pub struct SteeringAgent {
    pub neighbour_hood_radius: f32,
}

impl Default for SteeringAgent {
    fn default() -> Self {
        Self {
            neighbour_hood_radius: 10.0,
        }
    }
}
