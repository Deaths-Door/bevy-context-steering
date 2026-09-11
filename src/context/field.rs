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

impl FromIterator<Weight> for SteeringField {
    fn from_iter<T: IntoIterator<Item = Weight>>(iter: T) -> Self {
        Self(FromIterator::from_iter(iter))
    }
}

impl IntoIterator for SteeringField {
    type Item = Weight;
    type IntoIter = std::vec::IntoIter<Weight>;
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a> IntoIterator for &'a SteeringField {
    type Item = &'a Weight;
    type IntoIter = std::slice::Iter<'a, Weight>;
    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl<'a> IntoIterator for &'a mut SteeringField {
    type Item = &'a mut Weight;
    type IntoIter = std::slice::IterMut<'a, Weight>;
    fn into_iter(self) -> Self::IntoIter {
        self.0.iter_mut()
    }
}
