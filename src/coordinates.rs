use rand::{rngs::StdRng, Rng, SeedableRng};

/// Coordinates in the three-dimensional plane.
#[derive(Clone, Copy, Debug)]
pub struct Coordinates {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

/// Oriented vector in the three-dimensional plane.
#[derive(Clone, Copy, Debug)]
pub struct CoordinatesVector {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl PartialEq for Coordinates {
    /// Compares `self` entry-wise with `other` to verify equality.
    fn eq(&self, other: &Self) -> bool {
        self.x == other.x && self.y == other.y && self.z == other.z
    }
}

impl Eq for Coordinates {}

impl Ord for Coordinates {
    /// Compares `self` entry-wise with `other`.
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
    /// Compares `self` entry-wise with `other`.
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl std::hash::Hash for Coordinates {
    /// Computes the hash of three-dimensional coordinates.
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.x.to_bits().hash(state);
        self.y.to_bits().hash(state);
        self.z.to_bits().hash(state);
    }
}

impl CoordinatesVector {
    /// Constructs a random vector.
    pub fn random() -> Self {
        let mut generator = StdRng::seed_from_u64(0);

        Self {
            x: generator.gen::<f64>(),
            y: generator.gen::<f64>(),
            z: generator.gen::<f64>(),
        }
    }

    /// Constructs a vector from an oriented pair of coordinates.
    pub fn from(connection: &(Coordinates, Coordinates)) -> Self {
        Self {
            x: connection.1.x - connection.0.x,
            y: connection.1.y - connection.0.y,
            z: connection.1.z - connection.0.z,
        }
    }

    /// Computes the euclidean norm of the current vector.
    pub fn norm(&self) -> f64 {
        (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }

    /// Multiplies `self` entry-wise by `multiplier`.
    pub fn rescale(&self, multiplier: f64) -> Self {
        Self {
            x: multiplier * self.x,
            y: multiplier * self.y,
            z: multiplier * self.z,
        }
    }

    /// Returns a normalized version of `self` if it is not the zero vector.
    pub fn normalize(&self, epsilon: f64) -> Option<Self> {
        let norm = self.norm();
        // nothing is returned in case of zero norm meaning less then epsilon
        if norm <= epsilon {
            return None;
        }
        // otherwise the vector is rescaled by its norm
        Some(self.rescale(1f64 / norm))
    }

    /// Computes the dot product with `other`.
    pub fn dot(&self, other: &Self) -> f64 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    /// Computes the three-dimensional cross product with `other`.
    pub fn cross(&self, other: &Self) -> Self {
        Self {
            x: self.y * other.z - self.z * other.y,
            y: self.z * other.x - self.x * other.z,
            z: self.x * other.y - self.y * other.x,
        }
    }

    /// Returns true if parallel to `other` namely their cross product is the zero vector.
    pub fn is_parallel_to(&self, other: &Self, epsilon: f64) -> bool {
        // when the norm of the resulting cross product is less then epsilon
        // then the vectors are considered collinear or parallel
        self.cross(other).norm() <= epsilon
    }
}
