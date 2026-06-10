//! Pure geometry math for edge checks: bezier flattening + segment intersection.
//! No usvg, no I/O — fully unit-testable with exact values.

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

impl Point {
    pub fn new(x: f32, y: f32) -> Point {
        Point { x, y }
    }
}

pub const FLATTEN_TOLERANCE_PX: f32 = 0.25;
const MAX_FLATTEN_DEPTH: u8 = 16;

fn mid(a: Point, b: Point) -> Point {
    Point {
        x: (a.x + b.x) / 2.0,
        y: (a.y + b.y) / 2.0,
    }
}

fn point_line_dist(p: Point, a: Point, b: Point) -> f32 {
    let dx = b.x - a.x;
    let dy = b.y - a.y;
    let len = (dx * dx + dy * dy).sqrt();
    if len == 0.0 {
        return ((p.x - a.x).powi(2) + (p.y - a.y).powi(2)).sqrt();
    }
    (dx * (a.y - p.y) - (a.x - p.x) * dy).abs() / len
}

fn cubic_is_flat(p0: Point, p1: Point, p2: Point, p3: Point) -> bool {
    point_line_dist(p1, p0, p3) <= FLATTEN_TOLERANCE_PX
        && point_line_dist(p2, p0, p3) <= FLATTEN_TOLERANCE_PX
}

/// Append the flattened cubic to `out`, excluding p0 and including p3.
pub fn flatten_cubic(p0: Point, p1: Point, p2: Point, p3: Point, out: &mut Vec<Point>) {
    subdiv_cubic(p0, p1, p2, p3, 0, out);
    out.push(p3);
}

fn subdiv_cubic(p0: Point, p1: Point, p2: Point, p3: Point, depth: u8, out: &mut Vec<Point>) {
    if depth >= MAX_FLATTEN_DEPTH || cubic_is_flat(p0, p1, p2, p3) {
        return;
    }
    let p01 = mid(p0, p1);
    let p12 = mid(p1, p2);
    let p23 = mid(p2, p3);
    let p012 = mid(p01, p12);
    let p123 = mid(p12, p23);
    let p0123 = mid(p012, p123);
    subdiv_cubic(p0, p01, p012, p0123, depth + 1, out);
    out.push(p0123);
    subdiv_cubic(p0123, p123, p23, p3, depth + 1, out);
}

/// Append the flattened quadratic to `out`, excluding p0 and including p2.
pub fn flatten_quad(p0: Point, p1: Point, p2: Point, out: &mut Vec<Point>) {
    // Elevate quadratic to cubic, then reuse the cubic flattener.
    let c1 = Point {
        x: p0.x + 2.0 / 3.0 * (p1.x - p0.x),
        y: p0.y + 2.0 / 3.0 * (p1.y - p0.y),
    };
    let c2 = Point {
        x: p2.x + 2.0 / 3.0 * (p1.x - p2.x),
        y: p2.y + 2.0 / 3.0 * (p1.y - p2.y),
    };
    flatten_cubic(p0, c1, c2, p2, out);
}

/// Proper crossing point of segments a–b and c–d, or None if they do not
/// properly cross (parallel, collinear, or only touching at an endpoint).
pub fn segment_intersection(a: Point, b: Point, c: Point, d: Point) -> Option<Point> {
    let rx = b.x - a.x;
    let ry = b.y - a.y;
    let sx = d.x - c.x;
    let sy = d.y - c.y;
    let denom = rx * sy - ry * sx;
    if denom.abs() < 1e-9 {
        return None;
    }
    let qpx = c.x - a.x;
    let qpy = c.y - a.y;
    let t = (qpx * sy - qpy * sx) / denom;
    let u = (qpx * ry - qpy * rx) / denom;
    if t > 0.0 && t < 1.0 && u > 0.0 && u < 1.0 {
        Some(Point {
            x: a.x + t * rx,
            y: a.y + t * ry,
        })
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn intersecting_segments_return_crossing_point() {
        let p = segment_intersection(
            Point::new(0.0, 0.0),
            Point::new(10.0, 10.0),
            Point::new(0.0, 10.0),
            Point::new(10.0, 0.0),
        );
        assert_eq!(p, Some(Point::new(5.0, 5.0)));
    }

    #[test]
    fn parallel_segments_return_none() {
        let p = segment_intersection(
            Point::new(0.0, 0.0),
            Point::new(10.0, 0.0),
            Point::new(0.0, 5.0),
            Point::new(10.0, 5.0),
        );
        assert_eq!(p, None);
    }

    #[test]
    fn segments_sharing_endpoint_do_not_properly_cross() {
        let p = segment_intersection(
            Point::new(0.0, 0.0),
            Point::new(10.0, 0.0),
            Point::new(10.0, 0.0),
            Point::new(10.0, 10.0),
        );
        assert_eq!(p, None);
    }

    #[test]
    fn straight_cubic_flattens_to_single_endpoint() {
        let mut out = Vec::new();
        flatten_cubic(
            Point::new(0.0, 0.0),
            Point::new(1.0, 1.0),
            Point::new(2.0, 2.0),
            Point::new(3.0, 3.0),
            &mut out,
        );
        assert_eq!(out, vec![Point::new(3.0, 3.0)]);
    }

    #[test]
    fn curved_cubic_subdivides_and_ends_at_p3() {
        let mut out = Vec::new();
        flatten_cubic(
            Point::new(0.0, 0.0),
            Point::new(0.0, 10.0),
            Point::new(10.0, 10.0),
            Point::new(10.0, 0.0),
            &mut out,
        );
        assert!(out.len() > 1, "curved cubic should subdivide");
        assert_eq!(out.last().copied(), Some(Point::new(10.0, 0.0)));
    }
}
