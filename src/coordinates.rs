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
    pub fn unscaled(connection: &(Coordinates, Coordinates)) -> Self {
        Self {
            x: connection.1.x - connection.0.x,
            y: connection.1.y - connection.0.y,
            z: connection.1.z - connection.0.z,
        }
    }

    pub fn normalized(x: f64, y: f64, z: f64) -> CoordinatesVector {
        Self { x, y, z }.normalize(f64::EPSILON).unwrap()
    }

    pub fn random() -> Self {
        let mut generator = StdRng::seed_from_u64(0);

        Self::normalized(
            generator.gen::<f64>(),
            generator.gen::<f64>(),
            generator.gen::<f64>(),
        )
    }

    pub fn from(connection: &(Coordinates, Coordinates)) -> Self {
        Self::normalized(
            connection.1.x - connection.0.x,
            connection.1.y - connection.0.y,
            connection.1.z - connection.0.z,
        )
    }

    pub fn norm(&self) -> f64 {
        (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }

    pub fn flip(&self) -> Self {
        self.rescale(-1f64)
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

    pub fn normal(&self, other: &Self, epsilon: f64) -> Option<Self> {
        let result = Self {
            x: self.y * other.z - self.z * other.y,
            y: self.z * other.x - self.x * other.z,
            z: self.x * other.y - self.y * other.x,
        };

        if result.norm() <= epsilon {
            None
        } else {
            Some(result.rescale(1f64 / result.norm()))
        }
    }

    pub fn is_parallel_to(&self, other: &Self, epsilon: f64) -> bool {
        self.normal(other, epsilon).is_none()
    }

    fn rescale(&self, multiplier: f64) -> Self {
        CoordinatesVector {
            x: multiplier * self.x,
            y: multiplier * self.y,
            z: multiplier * self.z,
        }
    }

    pub fn normal_direction_to(
        a: &(Coordinates, Coordinates),
        b: &(Coordinates, Coordinates),
        epsilon: f64,
    ) -> Option<Self> {
        let a = Self::unscaled(a);
        let b = Self::unscaled(b);
        let cross = Self {
            x: a.y * b.z - a.z * b.y,
            y: a.z * b.x - a.x * b.z,
            z: a.x * b.y - a.y * b.x,
        };

        if cross.norm() <= epsilon * a.norm() * b.norm() {
            None
        } else {
            Some(cross.rescale(1f64 / cross.norm()))
        }
    }
}
