use indexmap::IndexSet;
use serde_json::json;
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::io;
use std::io::Write;

use super::coordinates::Coordinates;
use super::path::Path;
use super::polygon::Polygon;

/// Different kind of input lines from the expected dataset.
#[derive(Debug, Clone, Copy)]
enum LineKind {
    Ridge,
    Edge,
    RoofGap,
    RoofGapLine,
    Building,
    Helping,
}

/// Stores metadata and file information when reading a geojson dataset file.
pub struct GeoJson {
    /// Name of the file, stored for output file naming.
    filename: std::ffi::OsString,
    /// Input geojson saved metadata.
    metadata: Value,
    /// Saved line kinds to be re-exported when producing an output file.
    linekinds: HashMap<(Coordinates, Coordinates), LineKind>,
}

impl GeoJson {
    /// Reads the input geojson dataset given the `filename`.
    pub fn open(filename: &str) -> Self {
        match fs::read_to_string(filename) {
            Ok(content) => Self {
                filename: std::ffi::OsString::from(
                    std::path::Path::new(filename).file_name().unwrap(),
                ),
                metadata: serde_json::from_str(&content).unwrap(),
                linekinds: HashMap::new(),
            },
            Err(_) => panic!("Unable to read file `{}`", filename),
        }
    }

    /// Parse an input geojson dataset into the list of lines it contains.
    pub fn parse(&mut self) -> Vec<(Coordinates, Coordinates)> {
        // all lines contained in the file as pair of coordinates
        let mut lines = Vec::<(Coordinates, Coordinates)>::new();
        // each one is added and its kind is stored for future retrieval
        for element in self.metadata["features"].as_array().unwrap() {
            // skip the element if not a line
            if &element["geometry"]["type"] != "LineString" {
                continue;
            }
            // extreme coordinates of the line
            let coordinates = element["geometry"]["coordinates"].as_array().unwrap();
            // unpacks them
            let from = coordinates[0].as_array().unwrap();
            let to = coordinates[1].as_array().unwrap();
            // converts to points
            let line = (
                Coordinates {
                    x: from[0].as_f64().unwrap(),
                    y: from[1].as_f64().unwrap(),
                    z: from[2].as_f64().unwrap(),
                },
                Coordinates {
                    x: to[0].as_f64().unwrap(),
                    y: to[1].as_f64().unwrap(),
                    z: to[2].as_f64().unwrap(),
                },
            );
            // matches the line against different kinds
            match element["properties"]["type"].as_str() {
                Some("Takkant") => {
                    self.linekinds.insert(line, LineKind::Edge);
                }
                Some("Mønelinje") => {
                    self.linekinds.insert(line, LineKind::Ridge);
                }
                Some("Taksprang") => {
                    self.linekinds.insert(line, LineKind::RoofGap);
                }
                Some("TaksprangBunn") => {
                    self.linekinds.insert(line, LineKind::RoofGapLine);
                }
                Some("Bygningslinje") => {
                    self.linekinds.insert(line, LineKind::Building);
                }
                Some("Hjelpelinje3D") => {
                    self.linekinds.insert(line, LineKind::Helping);
                }
                _ => (),
            }
            // adds line
            lines.push(line);
        }
        // yields the list of lines that can be used to build the path graph
        lines
    }

    pub fn save(&self, polygons: &Vec<Polygon<'_>>, directory: &str) {
        // creates the geojson features even considering invalid lines to have a full output
        let features = polygons
            .iter()
            .enumerate()
            .map(|(identifier, polygon)| {
                json!({
                    "type": "Feature",
                    "properties": {
                        "label": identifier.to_string()
                    },
                    "geometry": {
                        "type": "Polygon",
                        "coordinates": [
                            polygon.path.sequence
                                .iter()
                                .map(|coordinates| [ coordinates.x, coordinates.y, coordinates.z ])
                                .collect::<Vec<_>>()
                        ]
                    }
                })
            })
            .collect::<Vec<Value>>();
        // writes to an output file having the same name as the input file but located within `directory`
        let outfilename = std::path::Path::new(directory).join(&self.filename);
        let filestream = fs::File::create(&outfilename).unwrap();
        let mut writer = io::BufWriter::new(filestream);
        let _ = match serde_json::to_writer_pretty(
            &mut writer,
            &json!({
                "type": self.metadata["type"],
                "name": self.metadata["name"],
                "crs": {
                    "type": self.metadata["crs"]["type"],
                    "properties": {
                        "name": self.metadata["crs"]["properties"]["name"]
                    }
                },
                "features": features
            }),
        ) {
            Ok(_) => writer.flush(),
            _ => panic!("Unable to write file `{outfilename:?}`"),
        };
    }
}
