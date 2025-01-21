use core::f64;
use rstar::{Envelope, RTree, RTreeObject, AABB};

use super::{coordinates::Coordinates, path::Path};

#[derive(PartialEq, Clone)]
pub struct Polygon<'a> {
    pub path: &'a Path,
    pub boundary: (Coordinates, Coordinates),
}

impl RTreeObject for Polygon<'_> {
    type Envelope = AABB<[f64; 2]>;

    fn envelope(&self) -> Self::Envelope {
        AABB::from_corners(
            [self.boundary.0.x, self.boundary.0.y],
            [self.boundary.1.x, self.boundary.1.y],
        )
    }
}

impl<'a> Polygon<'a> {
    pub fn from(path: &'a Path) -> Self {
        Self {
            path,
            boundary: Polygon::boundary(&path.sequence),
        }
    }

    fn boundary(path: &Vec<Coordinates>) -> (Coordinates, Coordinates) {
        let mut min = Coordinates {
            x: f64::INFINITY,
            y: f64::INFINITY,
            z: f64::NAN,
        };
        let mut max = Coordinates {
            x: f64::NEG_INFINITY,
            y: f64::NEG_INFINITY,
            z: f64::NAN,
        };

        for Coordinates { x, y, .. } in path {
            if *x < min.x {
                min.x = *x;
            }

            if *x > max.x {
                max.x = *x;
            }

            if *y < min.y {
                min.y = *y;
            }

            if *y > max.y {
                max.y = *y;
            }
        }

        (min, max)
    }

    fn contains_boundary_of(&self, other: &Self) -> bool {
        self.boundary.0.x <= other.boundary.0.x
            && self.boundary.1.x >= other.boundary.1.x
            && self.boundary.0.y <= other.boundary.0.y
            && self.boundary.1.y >= other.boundary.1.y
    }

    fn contains_point(&self, point: &Coordinates) -> bool {
        if self.path.contains(point) {
            return true;
        }

        let n = self.path.sequence.len() - 1;
        let mut inside = false;

        for i in 0..n {
            let a = self.path.sequence[i];
            let b = self.path.sequence[(i + 1) % n];

            if (a.y > point.y) != (b.y > point.y)
                && point.x < a.x + ((point.y - a.y) * (b.x - a.x) / (b.y - a.y))
            {
                inside = !inside;
            }
        }

        inside
    }

    fn shares_sides_with(&self, other: &Self) -> bool {
        for i in 0..(self.path.sequence.len() - 1) {
            for j in 0..(other.path.sequence.len() - 1) {
                if (self.path.sequence[i], self.path.sequence[i + 1])
                    == (other.path.sequence[j], other.path.sequence[j + 1])
                {
                    return true;
                }
            }
        }

        false
    }

    fn contains(&self, other: &Self) -> bool {
        self.contains_boundary_of(other)
            && other
                .path
                .sequence
                .iter()
                .all(|point| self.contains_point(point))
    }

    pub fn filter_fundamental_polygons_inefficient(polygons: Vec<Polygon<'a>>) -> Vec<Polygon<'a>> {
        let mask = polygons
            .iter()
            .map(|polygon| {
                !polygons
                    .iter()
                    .filter(|other| other.path != polygon.path)
                    .any(|other| polygon.contains(other) && polygon.shares_sides_with(other))
            })
            .collect::<Vec<_>>();

        polygons
            .into_iter()
            .zip(mask.iter())
            .filter(|(_, selected)| **selected)
            .map(|(polygon, _)| polygon)
            .collect()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn polygon_test() {
        let inner = Path::from(&vec![
            Coordinates {
                x: 0f64,
                y: 0f64,
                z: 0f64,
            },
            Coordinates {
                x: 0f64,
                y: 1f64,
                z: 0f64,
            },
            Coordinates {
                x: 1f64,
                y: 0f64,
                z: 0f64,
            },
            Coordinates {
                x: 0f64,
                y: 0f64,
                z: 0f64,
            },
            // Coordinates {
            //     x: 0.25f64,
            //     y: 0.25f64,
            //     z: 0f64,
            // },
            // Coordinates {
            //     x: 0.25f64,
            //     y: 0.75f64,
            //     z: 0f64,
            // },
            // Coordinates {
            //     x: 0.75f64,
            //     y: 0.25f64,
            //     z: 0f64,
            // },
            // Coordinates {
            //     x: 0.25f64,
            //     y: 0.25f64,
            //     z: 0f64,
            // },
        ]);
        let outer = Path::from(&vec![
            Coordinates {
                x: 0f64,
                y: 0f64,
                z: 0f64,
            },
            Coordinates {
                x: 0f64,
                y: 1f64,
                z: 0f64,
            },
            Coordinates {
                x: 1f64,
                y: 1f64,
                z: 0f64,
            },
            Coordinates {
                x: 1f64,
                y: 0f64,
                z: 0f64,
            },
            Coordinates {
                x: 0f64,
                y: 0f64,
                z: 0f64,
            },
        ]);

        let filtered = Polygon::filter_fundamental_polygons_inefficient(vec![
            Polygon::from(&outer),
            Polygon::from(&inner),
        ]);

        println!("number of filtered {}", filtered.len());

        // filtered.iter().for_each(|p| println!("{:#?}", p.sequence));
    }
}
