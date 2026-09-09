pub use avian3d::prelude::*;
pub use bevy::{
    app::PanicHandlerPlugin, mesh::MeshPlugin, prelude::*, scene::ScenePlugin,
    time::TimeUpdateStrategy,
};
pub use bevy_context_steering::{behaviours::*, motion::MotionKinematic, *};

pub const COLLIDER_RADIUS: f32 = 1.0;
pub const MOVEMENT_TOLERANCE: f32 = 2.0 * COLLIDER_RADIUS + 0.05;

pub fn agent<'a, 'b>(commands: &'a mut EntityCommands<'b>) -> &'a mut EntityCommands<'b> {
    commands.insert((
        RigidBody::Dynamic,
        Mass(1.0),
        Collider::sphere(COLLIDER_RADIUS),
        SteeringAgent::default(),
        MotionKinematic::default(),
        // Crucial: Avian needs damping to stop the "wobble"
        LinearDamping(1.0),
        AngularDamping(1.0),
    ))
}

pub fn neighbour<'a, 'b>(
    mask: LayerMask,
    half_extent: Vec3,
    commands: &'a mut EntityCommands<'b>,
) -> &'a mut EntityCommands<'b> {
    commands.insert((
        CollisionLayers::new(mask, mask),
        NeighbourhoodFilter::from(mask),
        NeighbourhoodExtents::from(half_extent),
    ))
}

pub trait SteeringScenarioExt {
    fn test() -> Self;
    fn step_frames(&mut self, frames: usize);

    fn step(&mut self) {
        self.step_frames(30);
    }

    fn step_frame(&mut self) {
        self.step_frames(1)
    }

    #[deprecated]
    fn spawn_agent(&mut self, with: impl FnOnce(EntityCommands<'_>)) -> Entity;
    fn agent(
        &mut self,
        with: impl for<'a, 'b> FnOnce(&'a mut EntityCommands<'b>) -> &'a mut EntityCommands<'b>,
    ) -> Entity;

    fn get<T: Component>(&self, entity: Entity) -> &T;
}

pub trait SteeringScenarioCommandsExt {
    fn spawn_agent(&mut self) -> &mut Self;
    fn neighbour(&mut self, mask: LayerMask, half_extent: Vec3) -> &mut Self;
}

impl SteeringScenarioExt for App {
    fn test() -> Self {
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

        app
    }

    fn step_frames(&mut self, count: usize) {
        for _ in 0..count {
            self.update();
        }
    }

    fn spawn_agent(&mut self, with: impl FnOnce(EntityCommands<'_>)) -> Entity {
        let mut commands = self.world_mut().commands();
        let mut commands = commands.spawn_empty();
        let commands = agent(&mut commands);
        let id = commands.id();
        (with)(commands.reborrow());
        self.world_mut().flush();
        id
    }

    fn agent(
        &mut self,
        with: impl for<'a, 'b> FnOnce(&'a mut EntityCommands<'b>) -> &'a mut EntityCommands<'b>,
    ) -> Entity {
        let mut commands = self.world_mut().commands();
        let mut commands = commands.spawn_empty();
        let commands = agent(&mut commands);
        let id = commands.id();
        (with)(commands);
        self.world_mut().flush();
        id
    }

    #[track_caller]
    fn get<T: Component>(&self, entity: Entity) -> &T {
        self.world().get(entity).expect("Failed to get component")
    }
}

impl SteeringScenarioCommandsExt for EntityCommands<'_> {
    fn spawn_agent(&mut self) -> &mut Self {
        agent(self)
    }

    fn neighbour(&mut self, mask: LayerMask, half_extent: Vec3) -> &mut Self {
        neighbour(mask, half_extent, self)
    }
}
