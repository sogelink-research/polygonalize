CREATE OR REPLACE FUNCTION plrust.example()
    RETURNS SETOF TEXT
    LANGUAGE plrust STRICT
AS $$
[dependencies]
    indexmap = "2.7.1"
    polygonalize = { git = "https://github.com/sogelink-research/polygonalize.git" }
[code]
    use indexmap::IndexSet; 
    use polygonalize::*;
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
    let unfiltered = paths
        .iter()
        .map(Polygon::from)
        .collect::<Vec<Polygon>>();
    // removes redundant polygons
    let filtered = Polygon::filter_fundamental_polygons_inefficient(unfiltered);
    // paths
    let paths = filtered
        .iter()
        .map(|polygon| polygon.path.clone())
        .collect::<Vec<Path>>();
    // in well-known text format
    Ok(Some(SetOfIterator::new(
        paths.into_iter().map(|path| Some(path.wkt())),
    )))
$$;
