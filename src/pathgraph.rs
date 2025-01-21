use indexmap::IndexMap;
use indexmap::IndexSet;

use super::coordinates::Coordinates;
use super::coordinates::CoordinatesVector;
use super::plane::PlaneMatcher;

type ProjectedSuccessors = Vec<(PlaneMatcher, (Coordinates, Coordinates))>;

#[derive(Debug)]
struct ProjectedIntersection {
    successor: (Coordinates, Coordinates),
    angle: Option<f64>,
}

#[derive(Debug)]
pub struct PathGraphBuilder {
    adjacencies: IndexMap<Coordinates, IndexSet<Coordinates>>,
    epsilon: f64,
}

#[derive(Debug)]
pub struct PathGraph {
    pub intersections: IndexMap<(Coordinates, Coordinates), ProjectedSuccessors>,
    pub epsilon: f64,
}

impl ProjectedIntersection {
    pub fn on_plane(
        plane: &PlaneMatcher,
        current: (Coordinates, Coordinates),
        successor: (Coordinates, Coordinates),
    ) -> Self {
        ProjectedIntersection {
            successor,
            angle: plane.project_angle_between(
                &CoordinatesVector::from(&current),
                &CoordinatesVector::from(&successor),
            ),
        }
    }
}

impl PathGraphBuilder {
    pub fn from(connections: &Vec<(Coordinates, Coordinates)>, epsilon: f64) -> Self {
        let mut adjacencies = IndexMap::<Coordinates, IndexSet<Coordinates>>::new();

        for (u, v) in connections {
            adjacencies
                .entry(*u)
                .and_modify(|to| {
                    to.insert(*v);
                })
                .or_insert(IndexSet::from([*v]));

            adjacencies
                .entry(*v)
                .and_modify(|to| {
                    to.insert(*u);
                })
                .or_insert(IndexSet::from([*u]));
        }

        let mut leaves = adjacencies
            .iter()
            .filter(|(_, to)| to.len() == 1)
            .map(|(leaf, _)| *leaf)
            .collect::<IndexSet<_>>();

        while !leaves.is_empty() {
            let mut updated = IndexSet::<Coordinates>::new();

            for leaf in &leaves {
                if adjacencies.contains_key(leaf) {
                    if let Some(adjacent) = adjacencies[leaf].iter().next() {
                        if adjacencies[adjacent].len() <= 2 {
                            updated.insert(*adjacent);
                        }

                        adjacencies.entry(*adjacent).and_modify(|to| {
                            to.swap_remove(leaf);
                        });
                    }

                    adjacencies.swap_remove(leaf);
                }
            }

            leaves = updated;
        }

        Self {
            adjacencies,
            epsilon,
        }
    }

    pub fn build(&self) -> PathGraph {
        let mut undefined =
            IndexMap::<(Coordinates, Coordinates), (Coordinates, Coordinates)>::new();
        let mut intersections = IndexMap::<
            (Coordinates, Coordinates),
            Vec<(PlaneMatcher, Vec<ProjectedIntersection>)>,
        >::new();

        for (intersection, neighbors) in &self.adjacencies {
            for u in neighbors {
                let incident = (*u, *intersection);

                intersections.entry(incident).or_default();

                for v in neighbors {
                    if u != v {
                        let adjacent = (*intersection, *v);
                        let plane = PlaneMatcher::between(&incident, &adjacent, self.epsilon);

                        intersections.entry(adjacent).or_default();

                        if plane.is_undefined() {
                            undefined.insert(incident, adjacent);
                        } else if let Some(matchers) = intersections.get_mut(&incident) {
                            let mut matching =
                                Option::<(&PlaneMatcher, &mut Vec<ProjectedIntersection>)>::None;
                            let mut coplanarity = f64::INFINITY;

                            for (matcher, successors) in matchers {
                                if let Some(value) = matcher.coplanarity_with(&plane) {
                                    if value < coplanarity && value <= self.epsilon {
                                        coplanarity = value;
                                        matching = Some((matcher, successors));
                                    }
                                }
                            }

                            if let Some((matcher, successors)) = matching {
                                successors.push(ProjectedIntersection::on_plane(
                                    matcher, incident, adjacent,
                                ));
                            } else {
                                intersections.entry(incident).and_modify(|matchers| {
                                    matchers.push((
                                        plane,
                                        vec![ProjectedIntersection::on_plane(
                                            &plane, incident, adjacent,
                                        )],
                                    ));
                                });
                            }
                        }
                    }
                }
            }
        }

        for (incident, adjacent) in &undefined {
            if let Some(matchers) = intersections.get_mut(incident) {
                for (matcher, successors) in matchers {
                    successors.push(ProjectedIntersection::on_plane(
                        matcher, *incident, *adjacent,
                    ));
                }
            }

            intersections.entry(*incident).and_modify(|matchers| {
                matchers.push((
                    PlaneMatcher::undefined(self.epsilon),
                    vec![ProjectedIntersection {
                        successor: *adjacent,
                        angle: None,
                    }],
                ));
            });
        }

        PathGraph {
            intersections: intersections
                .iter()
                .map(|(source, matchers)| {
                    (
                        *source,
                        matchers
                            .iter()
                            .map(|(matcher, successors)| {
                                (
                                    *matcher,
                                    successors
                                        .iter()
                                        .min_by(|u, v| u.angle.partial_cmp(&v.angle).unwrap())
                                        .unwrap()
                                        .successor,
                                )
                            })
                            .collect::<Vec<_>>(),
                    )
                })
                .collect(),
            epsilon: self.epsilon,
        }
    }
}
