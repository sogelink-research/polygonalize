pub mod coordinates;
pub mod io;
pub mod path;
pub mod pathgraph;
pub mod plane;
pub mod polygon;

pub use coordinates::*;
pub use io::*;
pub use path::*;
pub use pathgraph::*;
pub use plane::*;
pub use polygon::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn example_two_planes() {
        // tolerance to compute cross products and determine coplanarity
        const EPSILON: f64 = 0.1;
        // three dimensional example
        let lines = vec![
            (
                Coordinates {
                    x: 0f64,
                    y: 0f64,
                    z: 0f64,
                },
                Coordinates {
                    x: 7f64,
                    y: 0f64,
                    z: 0f64,
                },
            ),
            (
                Coordinates {
                    x: 7f64,
                    y: 0f64,
                    z: 0f64,
                },
                Coordinates {
                    x: 10f64,
                    y: 0f64,
                    z: 0f64,
                },
            ),
            (
                Coordinates {
                    x: 0f64,
                    y: 0f64,
                    z: 0f64,
                },
                Coordinates {
                    x: 0f64,
                    y: 25f64,
                    z: 15f64,
                },
            ),
            (
                Coordinates {
                    x: 10f64,
                    y: 0f64,
                    z: 0f64,
                },
                Coordinates {
                    x: 10f64,
                    y: 25f64,
                    z: 15f64,
                },
            ),
            (
                Coordinates {
                    x: 0f64,
                    y: 25f64,
                    z: 15f64,
                },
                Coordinates {
                    x: 10f64,
                    y: 25f64,
                    z: 15f64,
                },
            ),
            (
                Coordinates {
                    x: 0f64,
                    y: 0f64,
                    z: 0f64,
                },
                Coordinates {
                    x: 0f64,
                    y: 5f64,
                    z: -5f64,
                },
            ),
            (
                Coordinates {
                    x: 7f64,
                    y: 0f64,
                    z: 0f64,
                },
                Coordinates {
                    x: 7f64,
                    y: 5f64,
                    z: -5f64,
                },
            ),
            (
                Coordinates {
                    x: 0f64,
                    y: 5f64,
                    z: -5f64,
                },
                Coordinates {
                    x: 7f64,
                    y: 5f64,
                    z: -5f64,
                },
            ),
        ];
        // builds the path graph from oriented lines
        let graph = pathgraph::PathGraphBuilder::from(&lines, EPSILON).build();
        // builds polygons as planes from the graph
        let paths = path::PathBuilder::from(&graph).build();
        // this specific inputs contains two planes with positive-oriented normals
        assert_eq!(
            2,
            paths.len(),
            "this input must be split in exactly two planes"
        );
    }

    #[test]
    fn example_one_plane_with_dead_ends() {
        // tolerance to compute cross products and determine coplanarity
        const EPSILON: f64 = 0.1;
        // same as before but without a connection forming a plane
        let lines = vec![
            (
                Coordinates {
                    x: 0f64,
                    y: 0f64,
                    z: 0f64,
                },
                Coordinates {
                    x: 7f64,
                    y: 0f64,
                    z: 0f64,
                },
            ),
            (
                Coordinates {
                    x: 7f64,
                    y: 0f64,
                    z: 0f64,
                },
                Coordinates {
                    x: 10f64,
                    y: 0f64,
                    z: 0f64,
                },
            ),
            (
                Coordinates {
                    x: 0f64,
                    y: 0f64,
                    z: 0f64,
                },
                Coordinates {
                    x: 0f64,
                    y: 25f64,
                    z: 15f64,
                },
            ),
            (
                Coordinates {
                    x: 10f64,
                    y: 0f64,
                    z: 0f64,
                },
                Coordinates {
                    x: 10f64,
                    y: 25f64,
                    z: 15f64,
                },
            ),
            (
                Coordinates {
                    x: 0f64,
                    y: 25f64,
                    z: 15f64,
                },
                Coordinates {
                    x: 10f64,
                    y: 25f64,
                    z: 15f64,
                },
            ),
            (
                Coordinates {
                    x: 0f64,
                    y: 0f64,
                    z: 0f64,
                },
                Coordinates {
                    x: 0f64,
                    y: 5f64,
                    z: -5f64,
                },
            ),
            (
                Coordinates {
                    x: 7f64,
                    y: 0f64,
                    z: 0f64,
                },
                Coordinates {
                    x: 7f64,
                    y: 5f64,
                    z: -5f64,
                },
            ),
        ];
        // builds the path graph from oriented lines
        let graph = pathgraph::PathGraphBuilder::from(&lines, EPSILON).build();
        // builds polygons as planes from the graph
        let paths = path::PathBuilder::from(&graph).build();
        // this specific inputs contains one single plane with positive-oriented normals
        assert_eq!(
            1,
            paths.len(),
            "this input must be split in exactly one plane"
        );
    }

    fn f() -> Result<Option<Vec<Option<String>>>, ()> {
        use indexmap::IndexSet;
        // well-known text conversion
        trait WellKnownTextConversion {
            fn wkt(&self) -> String;
        }
        // implement well-known text conversion
        impl WellKnownTextConversion for Path {
            fn wkt(&self) -> String {
                format!(
                    "POLYGON (({}))",
                    self.sequence
                        .iter()
                        .map(|coordinates| format!(
                            "{} {} {}",
                            coordinates.x, coordinates.y, coordinates.z
                        ))
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
        }
        // tolerance to compute cross products and determine coplanarity
        const EPSILON: f64 = 0.1;
        // same as before but without a connection forming a plane
        let lines = vec![
            (
                Coordinates {
                    x: 0f64,
                    y: 0f64,
                    z: 0f64,
                },
                Coordinates {
                    x: 7f64,
                    y: 0f64,
                    z: 0f64,
                },
            ),
            (
                Coordinates {
                    x: 7f64,
                    y: 0f64,
                    z: 0f64,
                },
                Coordinates {
                    x: 10f64,
                    y: 0f64,
                    z: 0f64,
                },
            ),
            (
                Coordinates {
                    x: 0f64,
                    y: 0f64,
                    z: 0f64,
                },
                Coordinates {
                    x: 0f64,
                    y: 25f64,
                    z: 15f64,
                },
            ),
            (
                Coordinates {
                    x: 10f64,
                    y: 0f64,
                    z: 0f64,
                },
                Coordinates {
                    x: 10f64,
                    y: 25f64,
                    z: 15f64,
                },
            ),
            (
                Coordinates {
                    x: 0f64,
                    y: 25f64,
                    z: 15f64,
                },
                Coordinates {
                    x: 10f64,
                    y: 25f64,
                    z: 15f64,
                },
            ),
            (
                Coordinates {
                    x: 0f64,
                    y: 0f64,
                    z: 0f64,
                },
                Coordinates {
                    x: 0f64,
                    y: 5f64,
                    z: -5f64,
                },
            ),
            (
                Coordinates {
                    x: 7f64,
                    y: 0f64,
                    z: 0f64,
                },
                Coordinates {
                    x: 7f64,
                    y: 5f64,
                    z: -5f64,
                },
            ),
            (
                Coordinates {
                    x: 0f64,
                    y: 5f64,
                    z: -5f64,
                },
                Coordinates {
                    x: 7f64,
                    y: 5f64,
                    z: -5f64,
                },
            ),
        ];
        // all paths
        let mut paths = IndexSet::<Path>::new();
        // tries different thresholds
        for epsilon in [0.005, 0.05, 0.25, 0.5] {
            // computes successors along each computing plane using the adjacency matrix
            let graph = PathGraphBuilder::from(&lines, epsilon).build();
            // constructs all paths from the graph using the given epsilon argument
            paths.extend(PathBuilder::from(&graph).build());
        }
        // maps to different type
        let unfiltered = paths.iter().map(Polygon::from).collect::<Vec<Polygon>>();
        // removes redundant polygons
        let filtered = Polygon::filter_fundamental_polygons_inefficient(unfiltered);
        // paths
        let paths = filtered
            .iter()
            .map(|polygon| polygon.path.clone())
            .collect::<Vec<Path>>();
        // test truthness
        assert_eq!(
            2,
            filtered.len(),
            "this input must be split in exactly one plane"
        );
        // in well-known text format
        Ok(Some(
            paths.into_iter().map(|path| Some(path.wkt())).collect(),
        ))
    }
}
