use std::collections::HashMap;
use std::collections::HashSet;

use super::coordinates::Coordinates;
use super::coordinates::CoordinatesVector;
use super::plane::PlaneMatcher;

/// List of successor connections along with the plane on which they lie
/// in accordance with their shared ancestor connection.
type ProjectedSuccessors = Vec<(PlaneMatcher, (Coordinates, Coordinates))>;

/// Successor connection projected on a specific plane.
#[derive(Debug)]
struct ProjectedIntersection {
    /// Successor connection.
    successor: (Coordinates, Coordinates),
    /// Projected angle if not collinear between ancestor and successor.
    angle: Option<f64>,
}

/// Graph builder which builds [PathGraph] as an undirected graph.
#[derive(Debug)]
pub struct PathGraphBuilder {
    /// Adjacency list of neighboring coordinates connected by a line.
    adjacencies: HashMap<Coordinates, HashSet<Coordinates>>,
    /// Threshold used to determine collinearity.
    epsilon: f64,
}

/// Graph of connections, namely oriented pairs of coordinates, built from [PathGraphBuilder].
#[derive(Debug)]
pub struct PathGraph {
    /// Maps each connection to its list of projected successors.
    pub intersections: HashMap<(Coordinates, Coordinates), ProjectedSuccessors>,
    /// Threshold used to determine collinearity.
    pub epsilon: f64,
}

impl ProjectedIntersection {
    /// Projects `successor` on plane `plane` shared by both `current` and `successor`.
    pub fn on_plane(
        plane: &PlaneMatcher,
        current: (Coordinates, Coordinates),
        successor: (Coordinates, Coordinates),
    ) -> Self {
        ProjectedIntersection {
            // successor
            successor,
            // projected angle
            angle: plane.project_angle_between(
                &CoordinatesVector::from(&current),
                &CoordinatesVector::from(&successor),
            ),
        }
    }
}

impl PathGraphBuilder {
    /// Instantiates a graph builder from a sequence of connections, namely oriented pairs of coordinates.
    pub fn from(connections: &Vec<(Coordinates, Coordinates)>, epsilon: f64) -> Self {
        // adjacency list of neighboring coordinates
        let mut adjacencies = HashMap::<Coordinates, HashSet<Coordinates>>::new();
        // update `adjacencies` with every pair of coordinates in `connection`
        for (u, v) in connections {
            // adds `v` to the successors of `u`
            adjacencies
                .entry(*u)
                .and_modify(|to| {
                    to.insert(*v);
                })
                .or_insert(HashSet::from([*v]));
            // adds `u` to the successors of `v` because of the undirected nature of the graph
            adjacencies
                .entry(*v)
                .and_modify(|to| {
                    to.insert(*u);
                })
                .or_insert(HashSet::from([*u]));
        }
        // finds all the coordinates that behave like a sink, namely they are dead ends
        let mut leaves = adjacencies
            .iter()
            .filter(|(_, to)| to.len() == 1)
            .map(|(leaf, _)| *leaf)
            .collect::<HashSet<_>>();
        // keeps removing the coordinates from the graph that give birth to dead ends until we have none
        while !leaves.is_empty() {
            // next generation of leaves
            let mut updated = HashSet::<Coordinates>::new();
            // remove `leaf` from all its neighbors
            for leaf in &leaves {
                if adjacencies.contains_key(leaf) {
                    if let Some(adjacent) = adjacencies[leaf].iter().next() {
                        // if `adjacent` is now a dead end then it is added to `updated` for the next pruning round
                        if adjacencies[adjacent].len() <= 2 {
                            updated.insert(*adjacent);
                        }
                        // `leaf` is removed from the neighbors of `adjacent`
                        adjacencies.entry(*adjacent).and_modify(|to| {
                            to.remove(leaf);
                        });
                    }
                    // `leaf` if finally pruned from the whole adjacency list
                    adjacencies.remove(leaf);
                }
            }
            // `leaves` is now pointing to the new set of dead ends if any
            leaves = updated;
        }
        // the output builder has already pruned nodes meaning coordinates
        Self {
            adjacencies,
            epsilon,
        }
    }

    /// Builds an undirected graph from the pruned adjacency list of coordinates.
    pub fn build(&self) -> PathGraph {
        // map of pairs of connections that suffer collinearity
        let mut undefined =
            HashMap::<(Coordinates, Coordinates), (Coordinates, Coordinates)>::new();
        // map of connection and successors lying on detected planes
        let mut intersections = HashMap::<
            (Coordinates, Coordinates),
            Vec<(PlaneMatcher, Vec<ProjectedIntersection>)>,
        >::new();
        // constructs the preliminar graph from the adjacency list for each `intersection` coordinates
        for (intersection, neighbors) in &self.adjacencies {
            // iterates over neighbors of `intersection`
            for u in neighbors {
                // `incident` connection (u, intersection) will be considered entering `intersection`
                let incident = (*u, *intersection);
                // initializes the successors list of incident if empty
                intersections.entry(incident).or_default();
                // iterates again over neighbors of `intersection`
                for v in neighbors {
                    // avoids adding (u, v) to the successors of (v, u) otherwise it would create an unwanted loop
                    if u != v {
                        // constructs exiting connection (intersection, v) from coordinates `intersection`
                        let adjacent = (*intersection, *v);
                        // constructs the plane between `incident` and its successor `adjacent`
                        let plane = PlaneMatcher::between(
                            &CoordinatesVector::from(&incident),
                            &CoordinatesVector::from(&adjacent),
                            self.epsilon,
                        );
                        // initializes as an empty list the successors of `adjacent`
                        intersections.entry(adjacent).or_default();
                        // delays the registration of `adjacent` is it is collinear with `incident` meaning undefined plane
                        if plane.is_undefined() {
                            // collinear neighbors will be handled later
                            undefined.insert(incident, adjacent);
                        } else if let Some(matchers) = intersections.get_mut(&incident) {
                            // checks whether `adjacent` lies on an already identified plane on which `incident` lies
                            let mut matched = false;
                            // iterates over all previously identified planes on which `incident` lies considering its successors
                            for (matcher, successors) in matchers {
                                // in case the plane already exists
                                if *matcher == plane {
                                    // match found
                                    matched = true;
                                    // adds `adjacent` to the already existing plane as projected successor
                                    successors.push(ProjectedIntersection::on_plane(
                                        matcher, incident, adjacent,
                                    ));
                                }
                            }
                            // if `adjacent` forms a new plane with `incident` without them being collinear
                            if !matched {
                                // the new plane is registered for `incident`
                                intersections.entry(incident).and_modify(|matchers| {
                                    // and `adjacent` is added as projected successor lying on that plane
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
        // pairs of (`incident`, `adjacent`) being collinear are now processed
        for (incident, adjacent) in &undefined {
            // since the plane is undefined, we add `adjacent` on any detected plane on which `incident` lies
            if let Some(matchers) = intersections.get_mut(incident) {
                for (matcher, successors) in matchers {
                    successors.push(ProjectedIntersection::on_plane(
                        matcher, *incident, *adjacent,
                    ));
                }
            }
            // and we also add `adjacent` on an undefined plane on which `incident` lies
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
        // constructs the path graph by keeping for each connection and for each detected plane on which it lies
        // only the successor connection that minimizes the plane-projected clockwise angle
        PathGraph {
            intersections: intersections
                .iter()
                .map(|(source, matchers)| {
                    (
                        // the current connection
                        *source,
                        // all correspondigly detected planes
                        matchers
                            .iter()
                            .map(|(matcher, successors)| {
                                (
                                    // any detected plane
                                    *matcher,
                                    // finds the successor connection that minimizes the projected clockwise angle
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
            // threshold used to recognize the collinearity between a pair of connection
            epsilon: self.epsilon,
        }
    }
}
