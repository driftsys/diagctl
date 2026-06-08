//! Geometry foundation: classify a rendered SVG into nodes/edges/labels/background.

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BBox {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl BBox {
    pub fn area(&self) -> f32 {
        self.w.max(0.0) * self.h.max(0.0)
    }
    pub fn right(&self) -> f32 {
        self.x + self.w
    }
    pub fn bottom(&self) -> f32 {
        self.y + self.h
    }
    pub fn intersection_area(&self, o: &BBox) -> f32 {
        let ox = (self.right().min(o.right()) - self.x.max(o.x)).max(0.0);
        let oy = (self.bottom().min(o.bottom()) - self.y.max(o.y)).max(0.0);
        ox * oy
    }
    pub fn intersects(&self, o: &BBox) -> bool {
        self.intersection_area(o) > 0.0
    }
    pub fn contains(&self, o: &BBox) -> bool {
        self.x <= o.x && self.y <= o.y && self.right() >= o.right() && self.bottom() >= o.bottom()
    }
    pub fn inset(&self, d: f32) -> BBox {
        BBox {
            x: self.x + d,
            y: self.y + d,
            w: self.w - 2.0 * d,
            h: self.h - 2.0 * d,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn b(x: f32, y: f32, w: f32, h: f32) -> BBox {
        BBox { x, y, w, h }
    }

    #[test]
    fn overlapping_boxes_intersect() {
        let a = b(0.0, 0.0, 10.0, 10.0);
        let c = b(5.0, 5.0, 10.0, 10.0);
        assert!(a.intersects(&c));
        assert_eq!(a.intersection_area(&c), 25.0);
    }

    #[test]
    fn touching_edges_do_not_intersect() {
        let a = b(0.0, 0.0, 10.0, 10.0);
        let c = b(10.0, 0.0, 10.0, 10.0);
        assert!(!a.intersects(&c));
        assert_eq!(a.intersection_area(&c), 0.0);
    }

    #[test]
    fn containment_detected_one_way() {
        let outer = b(0.0, 0.0, 100.0, 100.0);
        let inner = b(10.0, 10.0, 20.0, 20.0);
        assert!(outer.contains(&inner));
        assert!(!inner.contains(&outer));
    }

    #[test]
    fn inset_shrinks_on_all_sides() {
        let a = b(0.0, 0.0, 10.0, 10.0).inset(1.0);
        assert_eq!(a, b(1.0, 1.0, 8.0, 8.0));
    }
}
