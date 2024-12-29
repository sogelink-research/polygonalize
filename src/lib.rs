pub mod coordinates;
pub mod io;
pub mod pathgraph;
pub mod plane;
pub mod polygon;

pub use coordinates::*;
pub use io::*;
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
        let polygons = polygon::PolygonalPathBuilder::from(&graph).build();
        // this specific inputs contains two planes with positive-oriented normals
        assert_eq!(
            2,
            polygons.len(),
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
        let polygons = polygon::PolygonalPathBuilder::from(&graph).build();
        // this specific inputs contains one single plane with positive-oriented normals
        assert_eq!(
            1,
            polygons.len(),
            "this input must be split in exactly one plane"
        );
    }
}
