use super::coordinates::CoordinatesVector;

/// Three-dimensional plane.
#[derive(Clone, Copy, Debug)]
struct Plane {
    /// Normal vector of the plane.
    normal: CoordinatesVector,
    /// Two basis vectors spanning the plane.
    basis: (CoordinatesVector, CoordinatesVector),
}

/// A plane matcher can represent a three-dimensional plane or an undefined one.
#[derive(Clone, Copy, Debug)]
pub struct PlaneMatcher {
    /// Well-defined or undefined plane.
    plane: Option<Plane>,
    /// Threshold controlling collinearity.
    epsilon: f64,
}

impl PlaneMatcher {
    /// Constructs a plane if any on which both `current` and `successor` connections lie.
    pub fn between(
        current: &CoordinatesVector,
        successor: &CoordinatesVector,
        epsilon: f64,
    ) -> Self {
        // first checks whether the two vectors have a defined cross product direction
        match current.cross(successor).normalize(epsilon) {
            // if the cross product is not the zero vector then a plane exists
            Some(normal) => {
                // computes the normal having a positive `z` entry
                let normal = if normal.z < 0f64 {
                    normal.rescale(-1f64)
                } else {
                    normal
                };
                // computes the reference vector
                let r = CoordinatesVector::random().normalize(f64::EPSILON).unwrap();
                // computes the first vector of the basis
                let u = normal.cross(&r).normalize(f64::EPSILON).unwrap();
                // and the second
                let v = normal.cross(&u).normalize(f64::EPSILON).unwrap();
                // constructs plane matcher
                PlaneMatcher {
                    // plane
                    plane: Some(Plane {
                        normal,
                        basis: (u, v),
                    }),
                    // collinearity threshold
                    epsilon,
                }
            }
            // if they have the zero vector as cross product then they are collinear and no plane is defined
            None => PlaneMatcher::undefined(epsilon),
        }
    }

    /// Constructs an undefined plane using `epsilon` as collinearity threshold.
    pub fn undefined(epsilon: f64) -> Self {
        PlaneMatcher {
            plane: None,
            epsilon,
        }
    }

    /// Returns true if the plane has no defined direction.
    pub fn is_undefined(&self) -> bool {
        self.plane.is_none()
    }

    /// Returns true if it shares the same plane with `other` meaning they have parallel normal vectors.
    pub fn is_same_as(&self, other: &Self) -> bool {
        match (self.plane, other.plane) {
            (Some(p), Some(q)) => p.normal.is_parallel_to(&q.normal, self.epsilon),
            _ => false,
        }
    }

    /// Returns a well-defined plane matcher only if it shares the same plane with `other`.
    pub fn match_against(&self, other: &Self) -> Option<PlaneMatcher> {
        match (self.plane, other.plane) {
            // in case both planes are defined
            (Some(p), Some(q)) => {
                // returns the current plane if both are parallel
                if p.normal.is_parallel_to(&q.normal, self.epsilon) {
                    Some(*self)
                } else {
                    None
                }
            }
            (Some(_), None) => Some(*self),
            (None, Some(_)) => Some(*other),
            _ => None,
        }
    }

    /// Projects `vector` on the plane if defined as unit vector.
    pub fn project(&self, vector: &CoordinatesVector) -> Option<CoordinatesVector> {
        match self.plane {
            Some(plane) => CoordinatesVector {
                x: plane.basis.0.dot(vector),
                y: plane.basis.1.dot(vector),
                z: plane.normal.dot(vector),
            }
            .normalize(self.epsilon),
            None => None,
        }
    }

    /// Projects `current` and `successor` vectors on the plane and computes the clockwise
    /// angle on the plane between the two.
    pub fn project_angle_between(
        &self,
        current: &CoordinatesVector,
        successor: &CoordinatesVector,
    ) -> Option<f64> {
        // only if the vectors can be projected on the current plane
        match (self.project(current), self.project(successor)) {
            (Some(u), Some(v)) => {
                // computes the clockwise angle projected on the plane
                Some(std::f64::consts::PI + (v.y * u.x - v.x * u.y).atan2(u.x * v.x + u.y * v.y))
            }
            _ => None,
        }
    }
}

impl PartialEq for PlaneMatcher {
    /// Same as [PlaneMatcher::is_same_as]
    fn eq(&self, other: &Self) -> bool {
        self.is_same_as(other)
    }
}
