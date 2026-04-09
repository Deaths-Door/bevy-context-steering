use super::*;

#[derive(Component, Reflect)]
/*TODO:Neighborhood */
#[require(SteeringContext)]

pub struct SteeringAgent {
    pub max_speed: f32,
    pub max_force: Vec3,
    pub acceleration_wn: f32,
    pub neighbour_hood_radius: f32,
}

impl Default for SteeringAgent {
    fn default() -> Self {
        Self {
            max_speed: 100.0,
            max_force: Vec3::splat(100.0),
            acceleration_wn: 8.0,
            neighbour_hood_radius: 10.0,
        }
    }
}
