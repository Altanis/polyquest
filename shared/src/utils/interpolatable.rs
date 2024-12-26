#[derive(Debug, Clone)]
pub struct Interpolatable<T: Default + Clone> {
    pub original: T,
    pub value: T,
    pub target: T,
    pub direction: f32
}

impl<T: Default + Clone> Default for Interpolatable<T> {
    fn default() -> Self {
        Interpolatable {
            original: T::default(),
            value: T::default(),
            target: T::default(),
            direction: 1.0
        }
    }
}

impl<T: Default + Clone> Interpolatable<T> {
    pub fn new(value: T) -> Self {
        Self {
            original: value.clone(),
            target: value.clone(),
            value,
            direction: 1.0
        }
    }
}