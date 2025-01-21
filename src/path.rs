use indexmap::IndexMap;
use indexmap::IndexSet;
use std::collections::BTreeSet;
use std::hash::Hash;

use super::coordinates::Coordinates;
use super::coordinates::CoordinatesVector;
use super::pathgraph::PathGraph;
use super::plane::PlaneMatcher;

#[derive(Clone)]
pub struct Path {
    pub sequence: Vec<Coordinates>,
    pub set: BTreeSet<Coordinates>,
}

pub struct PathBuilder<'a> {
    graph: &'a PathGraph,
    cache: RecursionCache,
    paths: IndexSet<Path>,
    stack: Vec<Coordinates>,
    seen: IndexSet<Coordinates>,
}

struct RecursionCache {
    table: IndexMap<(Coordinates, Coordinates, Coordinates), Vec<PlaneMatcher>>,
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

impl<'a> PathBuilder<'a> {
    pub fn from(graph: &'a PathGraph) -> Self {
        Self {
            graph,
            cache: RecursionCache::new(),
            paths: IndexSet::new(),
            stack: Vec::new(),
            seen: IndexSet::new(),
        }
    }

    pub fn build(mut self) -> IndexSet<Path> {
        for source in self.graph.intersections.keys() {
            self.cache.table.clear();
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
            self.save(Path::from(&self.stack), plane);
            RecursionResult::closure()
        } else if self.contains(&current.1) {
            RecursionResult::backtrack(&current.1, plane)
        } else {
            if let Some(matchers) = self.graph.intersections.get(current) {
                for (matcher, successor) in matchers {
                    if let Some(plane) = matcher.match_against(plane, matchers.len() == 1) {
                        self.push(current.1);

                        let result = self.traverse(successor, &plane);

                        if let RecursionResult::Backtrack {
                            destination,
                            plane,
                            sequence,
                        } = &result
                        {
                            if *destination == current.1 {
                                self.save(Path::from(sequence), plane);
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

    fn save(&mut self, path: Path, plane: &PlaneMatcher) {
        if path.is_valid_on(plane, self.graph.epsilon) {
            self.paths.insert(path.reverse_if_normal_is_negative());
        }
    }

    fn push(&mut self, coordinates: Coordinates) {
        self.seen.insert(coordinates);
        self.stack.push(coordinates);
    }

    fn pop(&mut self) {
        if let Some(coordinates) = self.stack.pop() {
            self.seen.swap_remove(&coordinates);
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

impl Path {
    pub fn new() -> Self {
        Self {
            sequence: Vec::new(),
            set: BTreeSet::new(),
        }
    }

    pub fn from(sequence: &Vec<Coordinates>) -> Self {
        let mut result = Self::new();

        for coordinates in sequence {
            result.push(coordinates);
        }

        if let Some(root) = sequence.first() {
            result.sequence.push(*root);
        }

        result
    }

    pub fn push(&mut self, coordinates: &Coordinates) {
        self.sequence.push(*coordinates);
        self.set.insert(*coordinates);
    }

    pub fn contains(&self, coordinates: &Coordinates) -> bool {
        self.set.contains(coordinates)
    }

    fn sum_interior_angles_on(&self, plane: &PlaneMatcher) -> Option<f64> {
        let mut total = 0f64;

        for index in 0..(self.sequence.len() - 2) {
            match plane.project_angle_between(
                &CoordinatesVector::from(&(self.sequence[index], self.sequence[index + 1])),
                &CoordinatesVector::from(&(self.sequence[index + 1], self.sequence[index + 2])),
            ) {
                Some(current) => total += current,
                None => return None,
            }
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

    fn reverse_if_normal_is_negative(mut self) -> Self {
        for index in 0..(self.sequence.len() - 2) {
            if let Some(normal) =
                CoordinatesVector::from(&(self.sequence[index], self.sequence[index + 1])).normal(
                    &CoordinatesVector::from(&(self.sequence[index + 1], self.sequence[index + 2])),
                    f64::EPSILON,
                )
            {
                if normal.z < 0f64 {
                    self.sequence.reverse();
                    break;
                }
            }

            if index + 1 >= self.sequence.len() - 2 {
                break;
            }
        }

        self
    }
}

impl PartialEq for Path {
    fn eq(&self, other: &Self) -> bool {
        self.set.eq(&other.set)
    }
}

impl Eq for Path {}

impl Hash for Path {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.set
            .iter()
            .for_each(|coordinates| coordinates.hash(state));
    }
}

impl RecursionCache {
    fn new() -> Self {
        Self {
            table: IndexMap::new(),
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
