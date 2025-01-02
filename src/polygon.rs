use std::collections::BTreeSet;
use std::collections::HashMap;
use std::collections::HashSet;
use std::hash::Hash;

use super::coordinates::Coordinates;
use super::coordinates::CoordinatesVector;
use super::pathgraph::PathGraph;
use super::plane::PlaneMatcher;

/// A polygon is an ordered sequence of three-dimensional coordinates forming a path.
///
/// Note that a polygon exists in three dimensions but must always lie on a plane.
pub struct PolygonalPath {
    /// Sequence of coordinates.
    pub sequence: Vec<Coordinates>,
    /// Set of coordinates.
    set: BTreeSet<Coordinates>,
}

/// Builder of polygonal paths from a graph.
pub struct PolygonalPathBuilder<'a> {
    /// Reference to analyzed graph.
    graph: &'a PathGraph,
    /// Cache to memoize during recursion.
    cache: RecursionCache,
    /// Set of all detected polygons or closed paths.
    paths: HashSet<PolygonalPath>,
    /// Recursion stack when traversing the graph.
    stack: Vec<Coordinates>,
    /// Set of encountered coordinates from the recursion stack.
    seen: HashSet<Coordinates>,
}

/// Recursion cache table.
struct RecursionCache {
    /// Table of encountered planes associated to each triple of coordinates.
    table: HashMap<(Coordinates, Coordinates, Coordinates), Vec<PlaneMatcher>>,
}

/// Result returned when recursively traversing the graph to detect polygons.
enum RecursionResult {
    /// Backtrack to some specific coordinates.
    Backtrack {
        destination: Coordinates,
        plane: PlaneMatcher,
        sequence: Vec<Coordinates>,
    },
    /// Polygon detected.
    Closure,
    /// All neighbors were visited.
    Done,
}

impl<'a> PolygonalPathBuilder<'a> {
    /// Initializes a builder of polygons from a graph instance.
    pub fn from(graph: &'a PathGraph) -> Self {
        Self {
            graph,
            cache: RecursionCache::new(),
            paths: HashSet::new(),
            stack: Vec::new(),
            seen: HashSet::new(),
        }
    }

    /// Constructs the set of polygons from the graph and takes ownership of the builder itself afterwards.
    pub fn build(mut self) -> HashSet<PolygonalPath> {
        // explores every connection from the graph
        for source in self.graph.intersections.keys() {
            // adds the first encountered coordinates to the stack of visited coordinates
            self.push(source.0);
            // traverses the graph trying to construct a polygon beginning from `source` and
            // recursively visits its successors while finding the planes they belong to
            self.traverse(source, &PlaneMatcher::undefined(self.graph.epsilon));
            // clears the stack of visited coordinates
            self.pop();
        }
        // returns the detected polygons
        self.paths
    }

    /// Recursively traverses the `current` connection and the successors forming a plane if any that matches `plane`.
    fn traverse(
        &mut self,
        current: &(Coordinates, Coordinates),
        plane: &PlaneMatcher,
    ) -> RecursionResult {
        // verifies whether we have already visited a full connection until now
        if let Some(precedent) = self.precedent() {
            // checks whether the triple of vertices composed by the previous connection
            // and the next coordinates has been visited already with the current `plane`
            if self
                .cache
                .contains(&(precedent.0, precedent.1, current.1), plane)
            {
                // in such a case it stops the recursion because we have already been here
                return RecursionResult::done();
            }
        }
        // checks whether we are closing a path by touching the source vertex of the traversal
        if current.1 == self.root().unwrap() {
            // in case it constructs and saves the polygon lying on `plane`
            self.save(PolygonalPath::from(&self.stack), plane);
            // polygon has been constructed
            RecursionResult::closure()
        }
        // checks whether we are closing a path on an already encountered vertex that is not the source
        else if self.contains(&current.1) {
            // in that case it unwinds the recursion stack and backtracks to such vertex
            RecursionResult::backtrack(&current.1, plane)
        }
        // otherwise we are exploring this combination of connection and plane for the first time
        else {
            // we need to exahust all successors
            if let Some(matchers) = self.graph.intersections.get(current) {
                // explores each successor and tries to match the corresponding `matcher` with `plane`
                for (matcher, successor) in matchers {
                    // checks whether the plane of already visited vertices matches `plane`
                    // namely we can proceed by adding such a successor to the list of visited
                    // ones because we are violating coplanarity
                    if let Some(plane) = matcher.match_against(plane) {
                        // adds the next coordinates to the traversal stack
                        self.push(current.1);
                        // the result of the recursive traversal
                        let result = self.traverse(successor, &plane);
                        // in case of backtracking we need to decide whether to stop and continue
                        if let RecursionResult::Backtrack {
                            destination,
                            plane,
                            sequence,
                        } = &result
                        {
                            // checks whether we reached the destination of backtracking when unwinding
                            if *destination == current.1 {
                                // in that case it constructs the polygon lying on `plane` and saves it
                                self.save(PolygonalPath::from(sequence), plane);
                            } else {
                                // otherwise it removes the current coordinates from the traversal stack
                                self.pop();
                                // and keeps unwinding after adding the current coordinates to the polygonal path
                                return result.enqueue(&current.1);
                            }
                        }
                        // the current successor has been explored and we remove it from the recursion stack and pass
                        // to the next successor if any is left to be explored
                        self.pop();
                    }
                }
            }
            // if we have a predecessor we memoize the ordered triple composed by that and the next coordinates
            // into the traversal cache such that we do not come back here again
            if let Some(precedent) = self.precedent() {
                self.cache
                    .insert(&(precedent.0, precedent.1, current.1), plane);
            }
            // the vertex has been fully explored
            RecursionResult::done()
        }
    }

    /// Saves the polygons with positive normal orientation if it lies correctly on `plane`.
    fn save(&mut self, path: PolygonalPath, plane: &PlaneMatcher) {
        // saves only if the plane is not undefined and the polygon perfectly lies on the plane
        if !plane.is_undefined() && path.is_valid_on(plane, self.graph.epsilon) {
            // adds the polygon only with positive normal orientation
            self.paths
                .insert(path.reverse_if_normal_is_negative(self.graph.epsilon));
        }
    }

    /// Adds the visited coordinates during recursive traversal.
    fn push(&mut self, coordinates: Coordinates) {
        self.seen.insert(coordinates);
        self.stack.push(coordinates);
    }

    /// Removes the last visited coordinates during recursive traversal.
    fn pop(&mut self) {
        if let Some(coordinates) = self.stack.pop() {
            self.seen.remove(&coordinates);
        }
    }

    /// Returns the first visited coordinates if any during recursive traversal.
    fn root(&self) -> Option<Coordinates> {
        self.stack.first().copied()
    }

    /// Using the traversal stack it returns the last visited connection as pair of coordinates.
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

    /// Checks whether `coordinates` have been already visited during recursive traversal.
    fn contains(&self, coordinates: &Coordinates) -> bool {
        self.seen.contains(coordinates)
    }
}

impl PolygonalPath {
    /// Constructs an empty polygon.
    fn new() -> Self {
        Self {
            sequence: Vec::new(),
            set: BTreeSet::new(),
        }
    }

    /// Constructs a polygon from a sequence of coordinates.
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

    /// Adds a vertex to the polygon.
    fn push(&mut self, coordinates: &Coordinates) {
        self.sequence.push(*coordinates);
        self.set.insert(*coordinates);
    }

    /// Projects the polygon on `plane` if possible and computes the sum of interior angles.
    fn sum_interior_angles_on(&self, plane: &PlaneMatcher) -> Option<f64> {
        // the total sum of interior angles
        let mut total = 0f64;
        // iteratively adds the interior angle between each pair of sides of the polygon
        for index in 0..(self.sequence.len() - 2) {
            total += plane.project_angle_between(
                &CoordinatesVector::from(&(self.sequence[index], self.sequence[index + 1])),
                &CoordinatesVector::from(&(self.sequence[index + 1], self.sequence[index + 2])),
            )?;
        }
        // and the closing angle
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

    /// Verifies whether the polygon lies on `plane` using `tolerance` to test coplanarity.
    fn is_valid_on(&self, plane: &PlaneMatcher, tolerance: f64) -> bool {
        if self.sequence.is_empty() || self.sequence.first().ne(&self.sequence.last()) {
            false
        } else if let Some(total) = self.sum_interior_angles_on(plane) {
            (total - std::f64::consts::PI * (self.sequence.len() - 3) as f64).abs() <= tolerance
        } else {
            false
        }
    }

    /// Reverses in-place the sequence of vertices if they form a negative normal vector
    /// with respect to the plane where the polygon lies.
    fn reverse_if_normal_is_negative(mut self, epsilon: f64) -> Self {
        // index of the current vertex
        let mut index = 0usize;
        // tries to identify the normal vector of the plane on which the polygon lies by finding the first
        // couple of sides that form a plane
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
    /// Two polygons are equal if they have the same vertices regardless of the order.
    fn eq(&self, other: &Self) -> bool {
        self.set.eq(&other.set)
    }
}

impl Eq for PolygonalPath {}

impl Hash for PolygonalPath {
    /// Two polygons with the same vertices share the same hash regardless of the order.
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.set
            .iter()
            .for_each(|coordinates| coordinates.hash(state));
    }
}

impl RecursionCache {
    /// Initializes an empty memoization table for recursion.
    fn new() -> Self {
        Self {
            table: HashMap::new(),
        }
    }

    /// Tests whether the ordered triple of vertices has been tested already to lie on `plane`.
    fn contains(
        &self,
        path: &(Coordinates, Coordinates, Coordinates),
        plane: &PlaneMatcher,
    ) -> bool {
        // checks if `plane` has already been associated with the triple `path`
        if let Some(matchers) = self.table.get(path) {
            for matcher in matchers {
                if matcher == plane {
                    return true;
                }
            }
        }

        false
    }

    /// Marks the ordered triple of coordinates `path` as tested on `plane`.
    fn insert(&mut self, path: &(Coordinates, Coordinates, Coordinates), plane: &PlaneMatcher) {
        // adds the pair (triple, plane) to the memoization table
        self.table
            .entry(*path)
            .and_modify(|matchers| {
                matchers.push(*plane);
            })
            .or_insert(vec![*plane]);
    }
}

impl RecursionResult {
    /// Constructs a result returned when a connection has been fully visited.
    fn done() -> Self {
        Self::Done
    }

    /// Constructs a result returned when a polygon has been fully constructed.
    fn closure() -> Self {
        Self::Closure
    }

    /// Constructs a result returned when backing is performed.
    fn backtrack(destination: &Coordinates, plane: &PlaneMatcher) -> Self {
        Self::Backtrack {
            destination: *destination,
            plane: *plane,
            sequence: vec![*destination],
        }
    }

    /// Only when backtracking it adds a coordinates to the vertices of the polygon.
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
