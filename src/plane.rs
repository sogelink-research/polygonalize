use super::coordinates::CoordinatesVector;

#[derive(Clone, Copy, Debug)]
struct Plane {
    normal: CoordinatesVector,
    basis: (CoordinatesVector, CoordinatesVector),
}

#[derive(Clone, Copy, Debug)]
pub struct PlaneMatcher {
    plane: Option<Plane>,
    epsilon: f64,
}

impl PlaneMatcher {
    pub fn between(
        current: &CoordinatesVector,
        successor: &CoordinatesVector,
        epsilon: f64,
    ) -> Self {
        match current.cross(successor).normalize(epsilon) {
            Some(normal) => {
                let normal = if normal.z < 0f64 {
                    normal.rescale(-1f64)
                } else {
                    normal
                };
                let u = CoordinatesVector::random().normalize(f64::EPSILON).unwrap();
                let v = normal.cross(&u).normalize(f64::EPSILON).unwrap();

                PlaneMatcher {
                    plane: Some(Plane {
                        normal,
                        basis: (u, v),
                    }),
                    epsilon,
                }
            }
            None => PlaneMatcher::undefined(epsilon),
        }
    }

    pub fn undefined(epsilon: f64) -> Self {
        PlaneMatcher {
            plane: None,
            epsilon,
        }
    }

    pub fn is_undefined(&self) -> bool {
        self.plane.is_none()
    }

    pub fn is_same_as(&self, other: &Self) -> bool {
        match (self.plane, other.plane) {
            (Some(p), Some(q)) => p.normal.is_parallel_to(&q.normal, self.epsilon),
            _ => false,
        }
    }

    pub fn match_against(&self, other: &Self) -> Option<PlaneMatcher> {
        match (self.plane, other.plane) {
            (Some(p), Some(q)) => {
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

    pub fn project_angle_between(
        &self,
        current: &CoordinatesVector,
        successor: &CoordinatesVector,
    ) -> Option<f64> {
        match (self.project(current), self.project(successor)) {
            (Some(u), Some(v)) => {
                Some(std::f64::consts::PI + (v.y * u.x - v.x * u.y).atan2(u.x * v.x + u.y * v.y))
            }
            _ => None,
        }
    }
}

impl PartialEq for PlaneMatcher {
    fn eq(&self, other: &Self) -> bool {
        self.is_same_as(other)
    }
}
