use rand::{rngs::StdRng, Rng, SeedableRng};

/// Coordinates in the three-dimensional plane.
#[derive(Clone, Copy, Debug)]
pub struct Coordinates {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

#[derive(Clone, Copy, Debug)]
pub struct CoordinatesVector {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl PartialEq for Coordinates {
    fn eq(&self, other: &Self) -> bool {
        self.x == other.x && self.y == other.y && self.z == other.z
    }
}

impl Eq for Coordinates {}

impl Ord for Coordinates {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        if self.x < other.x {
            std::cmp::Ordering::Less
        } else if self.x > other.x {
            std::cmp::Ordering::Greater
        } else if self.y < other.y {
            std::cmp::Ordering::Less
        } else if self.y > other.y {
            std::cmp::Ordering::Greater
        } else if self.z < other.z {
            std::cmp::Ordering::Less
        } else if self.z > other.z {
            std::cmp::Ordering::Greater
        } else {
            std::cmp::Ordering::Equal
        }
    }
}

impl PartialOrd for Coordinates {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl std::hash::Hash for Coordinates {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.x.to_bits().hash(state);
        self.y.to_bits().hash(state);
        self.z.to_bits().hash(state);
    }
}

impl CoordinatesVector {
    pub fn random() -> Self {
        let mut generator = StdRng::seed_from_u64(0);

        Self {
            x: generator.gen::<f64>(),
            y: generator.gen::<f64>(),
            z: generator.gen::<f64>(),
        }
    }

    pub fn from(connection: &(Coordinates, Coordinates)) -> Self {
        Self {
            x: connection.1.x - connection.0.x,
            y: connection.1.y - connection.0.y,
            z: connection.1.z - connection.0.z,
        }
    }

    pub fn norm(&self) -> f64 {
        (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }

    pub fn rescale(&self, multiplier: f64) -> Self {
        Self {
            x: multiplier * self.x,
            y: multiplier * self.y,
            z: multiplier * self.z,
        }
    }

    pub fn normalize(&self, epsilon: f64) -> Option<Self> {
        let norm = self.norm();

        if norm <= epsilon {
            return None;
        }

        Some(self.rescale(1f64 / norm))
    }

    pub fn dot(&self, other: &Self) -> f64 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    pub fn cross(&self, other: &Self) -> Self {
        Self {
            x: self.y * other.z - self.z * other.y,
            y: self.z * other.x - self.x * other.z,
            z: self.x * other.y - self.y * other.x,
        }
    }

    pub fn is_parallel_to(&self, other: &Self, epsilon: f64) -> bool {
        self.cross(other).norm() <= epsilon
    }
}
