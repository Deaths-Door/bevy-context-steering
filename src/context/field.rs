use super::*;

/// Length shoould always be SAMPLE_SIZE
#[derive(Clone, Deref, DerefMut)]
pub struct SteeringField(Box<[Weight]>);

impl SteeringField {
    pub fn new(count: usize) -> Self {
        Self((0usize..count).map(|_| Weight::default()).collect())
    }

    pub fn from_cache(cache: &SteeringCache) -> Self {
        Self::new(cache.directions().len())
    }
}
