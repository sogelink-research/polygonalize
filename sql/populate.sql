DROP TABLE IF EXISTS lines;

CREATE TABLE lines(
    linestring TEXT
);

INSERT INTO lines(linestring) VALUES('LINESTRING (0.0 0.0 0.0, 7.0 0.0 0.0)');
INSERT INTO lines(linestring) VALUES('LINESTRING (7.0 0.0 0.0, 10.0 0.0 0.0)');
INSERT INTO lines(linestring) VALUES('LINESTRING (0.0 0.0 0.0, 0.0 25.0 15.0)');
INSERT INTO lines(linestring) VALUES('LINESTRING (10.0 0.0 0.0, 10.0 25.0 15.0)');
INSERT INTO lines(linestring) VALUES('LINESTRING (0.0 25.0 15.0, 10.0 25.0 15.0)');
INSERT INTO lines(linestring) VALUES('LINESTRING (0.0 0.0 0.0, 0.0 5.0 -5.0)');
INSERT INTO lines(linestring) VALUES('LINESTRING (7.0 0.0 0.0, 7.0 5.0 -5.0)');