#![allow(unused)]
use itertools::Itertools;
use pathfinder_geometry::vector::Vector2F;
use pathfinder_content::outline::{ Contour, ContourIterFlags };

use crate::{ max, min };

fn euclidean_distance(a: Vector2F, b: Vector2F) -> f32 {
    (a - b).length()
}

fn curve_length(contour: &Contour) -> f32 {
    contour
        .iter(ContourIterFlags::empty())
        .flat_map(|segment| vec![segment.baseline.from(), segment.baseline.to()])
        .tuple_windows()
        .map(|(a, b)| euclidean_distance(a, b))
        .sum()
}

fn extend_point_on_line(a: Vector2F, b: Vector2F, dist: f32) -> Vector2F {
    let norm = dist / euclidean_distance(a, b);
    b + (a - b) * norm
}

fn calc_value(
    i: usize,
    j: usize,
    prev_results_col: &[f32],
    current_results_col: &[f32],
    long_curve: &[Vector2F],
    short_curve: &[Vector2F]
) -> f32 {
    if i == 0 && j == 0 {
        return euclidean_distance(long_curve[0], short_curve[0]);
    }
    if i > 0 && j == 0 {
        return max(prev_results_col[0], euclidean_distance(long_curve[i], short_curve[0]));
    }
    let last_result = current_results_col[current_results_col.len() - 1];
    if i == 0 && j > 0 {
        return max(last_result, euclidean_distance(long_curve[0], short_curve[j]));
    }
    max(
        min(min(prev_results_col[j], prev_results_col[j - 1]), last_result),
        euclidean_distance(long_curve[i], short_curve[j])
    )
}

pub fn frechet_distance(curve1: &Contour, curve2: &Contour) -> f32 {
    // Extract points from contours
    let points1: Vec<Vector2F> = curve1
        .iter(ContourIterFlags::empty())
        .flat_map(|segment| vec![segment.baseline.from(), segment.baseline.to()])
        .collect();
    let points2: Vec<Vector2F> = curve2
        .iter(ContourIterFlags::empty())
        .flat_map(|segment| vec![segment.baseline.from(), segment.baseline.to()])
        .collect();

    let (longcalcurve, shortcalcurve) = if points1.len() > points2.len() {
        (&points1[..], &points2[..])
    } else {
        (&points2[..], &points1[..])
    };

    let mut prev_resultscalcol = vec![];
    for i in 0..longcalcurve.len() {
        let mut current_resultscalcol = vec![];
        for j in 0..shortcalcurve.len() {
            current_resultscalcol.push(
                calc_value(
                    i,
                    j,
                    &prev_resultscalcol,
                    &current_resultscalcol,
                    longcalcurve,
                    shortcalcurve
                )
            );
        }
        prev_resultscalcol = current_resultscalcol;
    }
    prev_resultscalcol[shortcalcurve.len() - 1]
}
