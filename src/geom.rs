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

use crate::svg::Svg;
use usvg::{Group, Node};

pub const BG_CANVAS_FRACTION: f32 = 0.95;
pub const NODE_FILL_FRACTION: f32 = 0.50;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Node,
    Edge,
    Label,
    Background,
    Other,
}

#[derive(Debug, Clone)]
pub struct Element {
    pub kind: Kind,
    pub bbox: BBox,
}

fn rect_to_bbox(r: usvg::Rect) -> BBox {
    BBox {
        x: r.x(),
        y: r.y(),
        w: r.width(),
        h: r.height(),
    }
}

fn node_bbox(n: &Node) -> BBox {
    let r = match n {
        Node::Group(g) => g.abs_bounding_box(),
        Node::Path(p) => p.abs_bounding_box(),
        Node::Image(i) => i.abs_bounding_box(),
        Node::Text(t) => t.abs_bounding_box(),
    };
    rect_to_bbox(r)
}

/// Apply `f` to every Path under `n` (recursing into groups).
fn for_each_path(n: &Node, f: &mut impl FnMut(&usvg::Path)) {
    match n {
        Node::Path(p) => f(p),
        Node::Group(g) => {
            for c in g.children() {
                for_each_path(c, f);
            }
        }
        _ => {}
    }
}

/// Descend through trivial single-child wrapper groups to the diagram "content" group.
fn content_group(root: &Group) -> &Group {
    let mut g = root;
    while g.children().len() == 1 {
        match &g.children()[0] {
            Node::Group(inner) => g = inner.as_ref(),
            _ => break,
        }
    }
    g
}

fn classify(unit: &Node, canvas_area: f32) -> Element {
    if let Node::Text(t) = unit {
        return Element {
            kind: Kind::Label,
            bbox: rect_to_bbox(t.abs_bounding_box()),
        };
    }
    let bbox = node_bbox(unit);
    let mut max_filled = 0.0f32;
    let mut has_open_stroke = false;
    for_each_path(unit, &mut |p| {
        let a = rect_to_bbox(p.abs_bounding_box()).area();
        if p.fill().is_some() {
            if a > max_filled {
                max_filled = a;
            }
        } else if p.stroke().is_some() {
            has_open_stroke = true;
        }
    });
    let kind = if max_filled >= BG_CANVAS_FRACTION * canvas_area {
        Kind::Background
    } else if bbox.area() > 0.0 && max_filled >= NODE_FILL_FRACTION * bbox.area() {
        Kind::Node
    } else if has_open_stroke {
        Kind::Edge
    } else {
        Kind::Other
    };
    Element { kind, bbox }
}

pub fn model(svg: &Svg) -> Vec<Element> {
    let size = svg.tree.size();
    let canvas_area = size.width() * size.height();
    let content = content_group(svg.tree.root());
    content
        .children()
        .iter()
        .map(|u| classify(u, canvas_area))
        .collect()
}

pub fn nodes(svg: &Svg) -> Vec<BBox> {
    model(svg)
        .into_iter()
        .filter(|e| e.kind == Kind::Node)
        .map(|e| e.bbox)
        .collect()
}

#[cfg(test)]
mod model_tests {
    use super::*;
    use crate::svg::load;
    use std::path::Path;

    fn count(svg: &Svg, k: Kind) -> usize {
        model(svg).into_iter().filter(|e| e.kind == k).count()
    }

    #[test]
    fn d2_clean_has_two_nodes_and_background() {
        let svg = load(Path::new("tests/fixtures/d2-clean.svg")).unwrap();
        assert_eq!(count(&svg, Kind::Node), 2, "d2 node count");
        assert_eq!(count(&svg, Kind::Background), 1, "d2 background count");
        assert!(count(&svg, Kind::Edge) >= 1, "d2 should have an edge");
    }

    #[test]
    fn plantuml_clean_collapses_subrects_to_two_nodes() {
        let svg = load(Path::new("tests/fixtures/plantuml-clean.svg")).unwrap();
        // Each PlantUML node wraps several filled sub-rects; grouping must yield 2 nodes.
        assert_eq!(count(&svg, Kind::Node), 2, "plantuml node count");
    }

    #[test]
    fn overlap_fixture_has_two_nodes() {
        let svg = load(Path::new("tests/fixtures/overlap.svg")).unwrap();
        assert_eq!(count(&svg, Kind::Node), 2, "overlap node count");
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
