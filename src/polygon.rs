use std::collections::BTreeSet;
use std::collections::HashMap;
use std::collections::HashSet;
use std::hash::Hash;

use super::coordinates::Coordinates;
use super::coordinates::CoordinatesVector;
use super::pathgraph::PathGraph;
use super::plane::PlaneMatcher;

pub struct PolygonalPath {
    pub sequence: Vec<Coordinates>,
    set: BTreeSet<Coordinates>,
}

pub struct PolygonalPathBuilder<'a> {
    graph: &'a PathGraph,
    cache: RecursionCache,
    paths: HashSet<PolygonalPath>,
    stack: Vec<Coordinates>,
    seen: HashSet<Coordinates>,
}

struct RecursionCache {
    table: HashMap<(Coordinates, Coordinates, Coordinates), Vec<PlaneMatcher>>,
}

enum RecursionResult {
    Backtrack {
        destination: Coordinates,
        plane: PlaneMatcher,
        sequence: Vec<Coordinates>,
    },
    Closure,
    Done,
}

impl<'a> PolygonalPathBuilder<'a> {
    pub fn from(graph: &'a PathGraph) -> Self {
        Self {
            graph,
            cache: RecursionCache::new(),
            paths: HashSet::new(),
            stack: Vec::new(),
            seen: HashSet::new(),
        }
    }

    pub fn build(mut self) -> HashSet<PolygonalPath> {
        for source in self.graph.intersections.keys() {
            self.push(source.0);
            self.traverse(source, &PlaneMatcher::undefined(self.graph.epsilon));
            self.pop();
        }

        self.paths
    }

    fn traverse(
        &mut self,
        current: &(Coordinates, Coordinates),
        plane: &PlaneMatcher,
    ) -> RecursionResult {
        if let Some(precedent) = self.precedent() {
            if self
                .cache
                .contains(&(precedent.0, precedent.1, current.1), plane)
            {
                return RecursionResult::done();
            }
        }

        if current.1 == self.root().unwrap() {
            self.save(PolygonalPath::from(&self.stack), plane);
            RecursionResult::closure()
        } else if self.contains(&current.1) {
            RecursionResult::backtrack(&current.1, plane)
        } else {
            if let Some(matchers) = self.graph.intersections.get(current) {
                for (matcher, successor) in matchers {
                    if let Some(plane) = matcher.match_against(plane) {
                        self.push(current.1);

                        let result = self.traverse(successor, &plane);

                        if let RecursionResult::Backtrack {
                            destination,
                            plane,
                            sequence,
                        } = &result
                        {
                            if *destination == current.1 {
                                self.save(PolygonalPath::from(sequence), plane);
                            } else {
                                self.pop();

                                return result.enqueue(&current.1);
                            }
                        }

                        self.pop();
                    }
                }
            }

            if let Some(precedent) = self.precedent() {
                self.cache
                    .insert(&(precedent.0, precedent.1, current.1), plane);
            }

            RecursionResult::done()
        }
    }

    fn save(&mut self, path: PolygonalPath, plane: &PlaneMatcher) {
        if !plane.is_undefined() && path.is_valid_on(plane, self.graph.epsilon) {
            self.paths
                .insert(path.reverse_if_normal_is_negative(self.graph.epsilon));
        }
        // else {
        //     self.paths.insert(path);
        // }
    }

    fn push(&mut self, coordinates: Coordinates) {
        self.seen.insert(coordinates);
        self.stack.push(coordinates);
    }

    fn pop(&mut self) {
        if let Some(coordinates) = self.stack.pop() {
            self.seen.remove(&coordinates);
        }
    }

    fn root(&self) -> Option<Coordinates> {
        self.stack.first().copied()
    }

    fn precedent(&self) -> Option<(Coordinates, Coordinates)> {
        if self.stack.len() < 2 {
            None
        } else {
            Some((
                self.stack[self.stack.len() - 2],
                self.stack[self.stack.len() - 1],
            ))
        }
    }

    fn contains(&self, coordinates: &Coordinates) -> bool {
        self.seen.contains(coordinates)
    }
}

impl PolygonalPath {
    fn new() -> Self {
        Self {
            sequence: Vec::new(),
            set: BTreeSet::new(),
        }
    }

    fn from(sequence: &Vec<Coordinates>) -> Self {
        let mut result = Self::new();

        for coordinates in sequence {
            result.push(coordinates);
        }

        if let Some(root) = sequence.first() {
            result.sequence.push(*root);
        }

        result
    }

    fn push(&mut self, coordinates: &Coordinates) {
        self.sequence.push(*coordinates);
        self.set.insert(*coordinates);
    }

    fn sum_interior_angles_on(&self, plane: &PlaneMatcher) -> Option<f64> {
        let mut total = 0f64;

        for index in 0..(self.sequence.len() - 2) {
            total += plane.project_angle_between(
                &CoordinatesVector::from(&(self.sequence[index], self.sequence[index + 1])),
                &CoordinatesVector::from(&(self.sequence[index + 1], self.sequence[index + 2])),
            )?;
        }

        plane
            .project_angle_between(
                &CoordinatesVector::from(&(
                    self.sequence[self.sequence.len() - 2],
                    self.sequence[0],
                )),
                &CoordinatesVector::from(&(self.sequence[0], self.sequence[1])),
            )
            .map(|current| total + current)
    }

    fn is_valid_on(&self, plane: &PlaneMatcher, tolerance: f64) -> bool {
        if self.sequence.is_empty() || self.sequence.first().ne(&self.sequence.last()) {
            false
        } else if let Some(total) = self.sum_interior_angles_on(plane) {
            (total - std::f64::consts::PI * (self.sequence.len() - 3) as f64).abs() <= tolerance
        } else {
            false
        }
    }

    fn reverse_if_normal_is_negative(mut self, epsilon: f64) -> Self {
        let mut index = 0usize;

        while index < self.sequence.len() - 2
            && CoordinatesVector::from(&(self.sequence[index], self.sequence[index + 1]))
                .is_parallel_to(
                    &CoordinatesVector::from(&(self.sequence[index + 1], self.sequence[index + 2])),
                    epsilon,
                )
        {
            index += 1;
        }

        if index >= self.sequence.len() - 2 {
            return self;
        }

        if CoordinatesVector::from(&(self.sequence[index], self.sequence[index + 1]))
            .cross(&CoordinatesVector::from(&(
                self.sequence[index + 1],
                self.sequence[index + 2],
            )))
            .z
            < 0f64
        {
            self.sequence.reverse();
        }

        self
    }
}

impl PartialEq for PolygonalPath {
    fn eq(&self, other: &Self) -> bool {
        self.set.eq(&other.set)
    }
}

impl Eq for PolygonalPath {}

impl Hash for PolygonalPath {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.set
            .iter()
            .for_each(|coordinates| coordinates.hash(state));
    }
}

impl RecursionCache {
    fn new() -> Self {
        Self {
            table: HashMap::new(),
        }
    }

    fn contains(
        &self,
        path: &(Coordinates, Coordinates, Coordinates),
        plane: &PlaneMatcher,
    ) -> bool {
        if let Some(matchers) = self.table.get(path) {
            for matcher in matchers {
                if matcher == plane {
                    return true;
                }
            }
        }

        false
    }

    fn insert(&mut self, path: &(Coordinates, Coordinates, Coordinates), plane: &PlaneMatcher) {
        self.table
            .entry(*path)
            .and_modify(|matchers| {
                matchers.push(*plane);
            })
            .or_insert(vec![*plane]);
    }
}

impl RecursionResult {
    fn done() -> Self {
        Self::Done
    }

    fn closure() -> Self {
        Self::Closure
    }

    fn backtrack(destination: &Coordinates, plane: &PlaneMatcher) -> Self {
        Self::Backtrack {
            destination: *destination,
            plane: *plane,
            sequence: vec![*destination],
        }
    }

    fn enqueue(mut self, coordinates: &Coordinates) -> Self {
        if let Self::Backtrack {
            destination: _,
            plane: _,
            sequence,
        } = &mut self
        {
            sequence.push(*coordinates);
        }

        self
    }
}
