CREATE OR REPLACE FUNCTION plrust.rooflines(inputs TEXT[])
    RETURNS SETOF TEXT
    LANGUAGE plrust STRICT
AS $$
[dependencies]
    indexmap = "2.7.1"
    polygonalize = { git = "https://github.com/sogelink-research/polygonalize.git" }
[code]
    // call the routine as
    // once the table is created
    // select * from plrust.rooflines((select array_agg(linestring) from lines));
    use indexmap::IndexSet; 
    use polygonalize::*;
    // linestring to pair of coordinates
    fn from_wkt(line: &str) -> (Coordinates, Coordinates) {
        let begin = line.find('(').unwrap();
        let end = line.find(')').unwrap();
        let comma = line.find(',').unwrap();
        let a = &line[(begin + 1)..comma]
            .trim()
            .split(" ")
            .collect::<Vec<&str>>();
        let b = &line[(comma + 1)..end]
            .trim()
            .split(" ")
            .collect::<Vec<&str>>();

        (
            Coordinates {
                x: a[0].parse().unwrap(),
                y: a[1].parse().unwrap(),
                z: a[2].parse().unwrap(),
            },
            Coordinates {
                x: b[0].parse().unwrap(),
                y: b[1].parse().unwrap(),
                z: b[2].parse().unwrap(),
            },
        )
    }
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
    // construct lines
    let lines = inputs
        .iter()
        .map(|linestring| from_wkt(linestring.unwrap()))
        .collect::<Vec<(Coordinates, Coordinates)>>();
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
