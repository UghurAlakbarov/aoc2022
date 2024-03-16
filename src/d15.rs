use std::{
    collections::{HashMap, HashSet},
    str::FromStr,
};

use anyhow::{Context, Error, Result};
use derive_deref::Deref;
use itertools::Itertools;
use rayon::{
    iter::{IntoParallelRefIterator, ParallelBridge, ParallelIterator},
    str::ParallelString,
};
use regex::Regex;

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
struct Point2D(i32, i32);

impl Point2D {
    fn manhattan_distance(self, other: Point2D) -> u32 {
        self.0.abs_diff(other.0) + self.1.abs_diff(other.1)
    }
}

type SensorPosition = Point2D;
type BeaconPosition = Point2D;

#[derive(Deref)]
struct SensorsWithBeacons(HashMap<SensorPosition, BeaconPosition>);

impl FromStr for SensorsWithBeacons {
    type Err = Error;

    fn from_str(s: &str) -> std::prelude::v1::Result<Self, Self::Err> {
        let coords_regex = Regex::new(
            r"Sensor at x=(-?\d+), y=(-?\d+): closest beacon is at x=(-?\d+), y=(-?\d+)",
        )?;
        let sensors_with_beacons = s
            .par_lines()
            .map(|line| {
                coords_regex
                    .captures_iter(line)
                    .map(|caps| caps.extract().1.map(|coord| i32::from_str(coord).unwrap()))
                    .exactly_one()
                    .unwrap()
            })
            .map(|[sensor_x, sensor_y, beacon_x, beacon_y]| {
                (Point2D(sensor_x, sensor_y), Point2D(beacon_x, beacon_y))
            })
            .collect::<HashMap<_, _>>();

        Ok(Self(sensors_with_beacons))
    }
}

#[allow(clippy::cast_sign_loss, clippy::cast_possible_wrap)]
pub fn p1(file: &str, analyzed_row_num: i32) -> Result<usize> {
    let sensors_with_beacons = SensorsWithBeacons::from_str(file)?;

    let mut impossible_locations_of_distress_beacon: HashSet<i32> = sensors_with_beacons
        .par_iter()
        .filter_map(|(signal, beacon)| {
            let distance_to_beacon = signal.manhattan_distance(*beacon);
            let distance_to_analyzed_row = signal.1.abs_diff(analyzed_row_num);

            match distance_to_analyzed_row.cmp(&distance_to_beacon) {
                std::cmp::Ordering::Greater => None,
                std::cmp::Ordering::Equal => Some(signal.0..=signal.0),
                std::cmp::Ordering::Less => {
                    let width_of_covered_space_on_the_analyzed_row =
                        distance_to_beacon - distance_to_analyzed_row;

                    Some(
                        (signal.0 - width_of_covered_space_on_the_analyzed_row as i32)
                            ..=(signal.0 + width_of_covered_space_on_the_analyzed_row as i32),
                    )
                }
            }
        })
        .flatten()
        .collect();

    // "is `x=2,y=10` a "position where a beacon cannot be present"?"
    for beacon in sensors_with_beacons.values() {
        if beacon.1 == analyzed_row_num {
            impossible_locations_of_distress_beacon.remove(&beacon.0);
        }
    }

    Ok(impossible_locations_of_distress_beacon.len())
}

#[derive(Deref)]
struct SensorsWithDistances(HashMap<SensorPosition, u32>);

impl FromStr for SensorsWithDistances {
    type Err = Error;

    fn from_str(s: &str) -> std::prelude::v1::Result<Self, Self::Err> {
        let coords_regex = Regex::new(
            r"Sensor at x=(-?\d+), y=(-?\d+): closest beacon is at x=(-?\d+), y=(-?\d+)",
        )?;
        let sensors_with_beacons = s
            .par_lines()
            .map(|line| {
                coords_regex
                    .captures_iter(line)
                    .map(|caps| caps.extract().1.map(|coord| i32::from_str(coord).unwrap()))
                    .exactly_one()
                    .unwrap()
            })
            .map(|[sensor_x, sensor_y, beacon_x, beacon_y]| {
                let sensor_coords = Point2D(sensor_x, sensor_y);
                let beacon_coords = Point2D(beacon_x, beacon_y);
                let distance = sensor_coords.manhattan_distance(beacon_coords);
                (sensor_coords, distance)
            })
            .collect::<HashMap<_, _>>();

        Ok(Self(sensors_with_beacons))
    }
}

#[allow(clippy::cast_possible_wrap)]
pub fn p2(file: &str, search_space_side_size: i32) -> Result<u64> {
    let sensors_with_distances = SensorsWithDistances::from_str(file)?;
    let distress_beacon = (0..=search_space_side_size)
        .cartesian_product(0..=search_space_side_size)
        .par_bridge()
        .map(|(x, y)| Point2D(x, y))
        .find_any(|point| {
            sensors_with_distances
                .par_iter()
                .all(|(sensor, distance_to_nearest_beacon)| {
                    sensor.manhattan_distance(*point) > *distance_to_nearest_beacon
                })
        })
        .context("No distress beacon found")?;
    let tuning_frequency: u64 = 4_000_000u64 * distress_beacon.0 as u64 + distress_beacon.1 as u64;
    Ok(tuning_frequency)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::read_to_string;
    #[test]
    fn test_p1() {
        let inp = read_to_string("inputs/d15/test.txt").unwrap();
        assert_eq!(p1(&inp, 10).unwrap(), 26);
    }
    #[test]
    fn real_p1() {
        let inp = read_to_string("inputs/d15/real.txt").unwrap();
        assert_eq!(p1(&inp, 2_000_000).unwrap(), 4748135);
    }
    #[test]
    fn test_p2() {
        let inp = read_to_string("inputs/d15/test.txt").unwrap();
        assert_eq!(p2(&inp, 20).unwrap(), 56_000_011);
    }
    #[test]
    fn real_p2() {
        let inp = read_to_string("inputs/d15/real.txt").unwrap();
        assert_eq!(p2(&inp).unwrap(), 0);
    }
}
