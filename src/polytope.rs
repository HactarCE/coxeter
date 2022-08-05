use smallvec::{smallvec, SmallVec};
use std::{collections::HashMap, ops::*};

use crate::matrix::Matrix;
use crate::util::EPSILON;
use crate::vector::{Vector, VectorRef};

pub fn shape_geom(
    ndim: u8,
    generators: &[Matrix<f32>],
    base_facets: &[Vector<f32>],
) -> Vec<Polygon> {
    let radius = base_facets
        .iter()
        .map(|pole| pole.mag())
        .reduce(f32::max)
        .expect("no base facets");
    let initial_radius = radius * 2.0 * ndim as f32;
    // TODO: check if radius is too small (any original point remains).
    let mut arena = PolytopeArena::new_cube(ndim, initial_radius);

    let mut facet_poles: Vec<Vector<f32>> = base_facets.to_vec();
    let mut next_unprocessed = 0;
    while next_unprocessed < facet_poles.len() {
        facet_poles[next_unprocessed].set_ndim(ndim);
        for gen in generators {
            let new_pole = gen.transform(&facet_poles[next_unprocessed]);
            if facet_poles.iter().all(|pole| !pole.approx_eq(&new_pole)) {
                facet_poles.push(new_pole);
            }
        }
        next_unprocessed += 1;
    }
    for pole in &facet_poles {
        arena.slice_by_plane(pole);
    }
    arena.polygons()
}

#[derive(Debug)]
pub struct PolytopeArena {
    polytopes: Vec<Option<Polytope>>,
    root: PolytopeId,
}
impl Index<PolytopeId> for PolytopeArena {
    type Output = Polytope;

    fn index(&self, index: PolytopeId) -> &Self::Output {
        self.polytopes[index.0 as usize].as_ref().unwrap()
    }
}
impl IndexMut<PolytopeId> for PolytopeArena {
    fn index_mut(&mut self, index: PolytopeId) -> &mut Self::Output {
        self.polytopes[index.0 as usize].as_mut().unwrap()
    }
}
impl PolytopeArena {
    pub fn new_cube(ndim: u8, radius: f32) -> Self {
        // Based on Andrey Astrelin's implementation of `GenCube()` in MPUlt
        // (FaceCuts.cs)

        // Make a 3^NDIM grid of polytopes to construct a hypercube. The corners
        // are vertices. Between those are edges, etc.
        //
        // ```
        // • - •
        // | # |
        // • - •
        // ```

        let mut ret = Self {
            polytopes: vec![],
            root: PolytopeId(3_u32.pow(ndim as _) / 2), // center of the 3^NDIM cube
        };

        let powers_of_3 = || std::iter::successors(Some(1), |x| Some(x * 3));

        for i in 0..3_u32.pow(ndim as _) {
            let rank = base_3_expansion(i, ndim)
                .filter(|&digit| digit == 1)
                .count() as u8;

            let contents = if rank == 0 {
                // This is a vertex.
                let point = base_3_expansion(i, ndim)
                    .map(|digit| (digit as f32 - 1.0) * radius)
                    .collect();
                PolytopeContents::Point(point)
            } else {
                // This is a branch node.
                let children = powers_of_3()
                    .zip(base_3_expansion(i, ndim))
                    // For each axis we are straddling ...
                    .filter(|&(_, digit)| digit == 1)
                    // ... add two children along that axis.
                    .flat_map(|(power_of_3, _)| {
                        [
                            PolytopeId(i - power_of_3 as u32),
                            PolytopeId(i + power_of_3 as u32),
                        ]
                    })
                    .collect();
                PolytopeContents::Branch { rank, children }
            };

            let parents = powers_of_3()
                .zip(base_3_expansion(i, ndim))
                // For each axis we are not straddling ...
                .filter(|&(_, digit)| digit != 1)
                // ... add the parent that straddles that axis.
                .map(|(power_of_3, digit)| i - power_of_3 * digit + power_of_3)
                .map(PolytopeId)
                .collect();

            ret.push(Polytope {
                parents,
                contents,
                slice_result: SliceResult::Unknown,
            });
        }

        ret
    }

    fn push(&mut self, polytope: Polytope) -> PolytopeId {
        self.polytopes.push(Some(polytope));
        PolytopeId(self.polytopes.len() as u32 - 1)
    }
    fn push_point(&mut self, point: Vector<f32>) -> PolytopeId {
        self.push(Polytope {
            parents: smallvec![],
            contents: PolytopeContents::Point(point),
            slice_result: SliceResult::Unknown,
        })
    }
    fn push_polytope(&mut self, children: impl IntoIterator<Item = PolytopeId>) -> PolytopeId {
        let children: SmallVec<[PolytopeId; 4]> = children.into_iter().collect();
        assert!(
            !children.is_empty(),
            "cannot construct non-point polytope with no children",
        );

        let rank = self[children[0]].rank() + 1;

        let ret = self.push(Polytope {
            parents: smallvec![],
            contents: PolytopeContents::Branch {
                rank,
                children: children.clone(),
            },
            slice_result: SliceResult::Unknown,
        });

        for &child in &children {
            self[child].parents.push(ret);
            debug_assert_eq!(
                self[child].rank() + 1,
                rank,
                "cannot construct polytope with mismsatched ranks"
            );
        }
        ret
    }
    fn add_child(&mut self, parent: PolytopeId, child: PolytopeId) {
        match &mut self[parent].contents {
            PolytopeContents::Point(_) => panic!("cannot add child to point"),
            PolytopeContents::Branch { rank, children } => {
                children.push(child);
                self[child].parents.push(parent);
                debug_assert_eq!(self[parent].rank(), self[child].rank() + 1);
            }
        }
    }

    pub fn polygons(&self) -> Vec<Polygon> {
        self.polytopes
            .iter()
            .filter_map(|x| x.as_ref())
            .filter(|p| p.rank() == 2)
            // For each polygon ...
            .map(|p| {
                let mut verts = Vec::with_capacity(p.children().len());

                // Make an adjacency list for each vertex.
                let mut edges: HashMap<PolytopeId, SmallVec<[PolytopeId; 2]>> = HashMap::new();
                for (v1, v2) in p
                    .children()
                    .iter()
                    .map(|&edge| self[edge].children())
                    .flat_map(|ch| [(ch[0], ch[1]), (ch[1], ch[0])])
                {
                    edges.entry(v1).or_default().push(v2);
                }

                let first_edge = p.children()[0];
                let first_vertex = self[first_edge].children()[0];
                let mut prev = first_vertex;
                let mut current = self[first_edge].children()[1];
                verts.push(self[current].unwrap_point().clone());
                while current != first_vertex {
                    let new = edges
                        .get(&current)
                        .unwrap()
                        .iter()
                        .copied()
                        .find(|&v| v != prev)
                        .expect("invalid polygon");
                    prev = current;
                    current = new;
                    verts.push(self[current].unwrap_point().clone());
                }

                Polygon { verts }
            })
            .collect()
    }

    pub fn slice_by_plane(&mut self, pole: &Vector<f32>) {
        self.slice_polytope(self.root, pole);

        for polytope in &mut self.polytopes {
            if let Some(p) = polytope {
                match p.slice_result {
                    SliceResult::Unknown => {
                        panic!("orphans in polytope arena")
                    }
                    // Remove dead polytopes.
                    SliceResult::Removed => *polytope = None,
                    // Reset slice results.
                    SliceResult::Kept | SliceResult::Modified(_) => {
                        p.slice_result = SliceResult::Unknown
                    }
                }
            }
        }
    }

    fn slice_polytope(&mut self, p: PolytopeId, pole: &Vector<f32>) -> SliceResult {
        if self[p].slice_result != SliceResult::Unknown {
            return self[p].slice_result;
        }

        let ret = match &self[p].contents {
            PolytopeContents::Point(point) => {
                if (pole - point).dot(pole) > -EPSILON {
                    SliceResult::Kept
                } else {
                    SliceResult::Removed
                }
            }
            PolytopeContents::Branch { rank, children } => {
                let rank = *rank;
                let mut intersection_boundary = vec![];
                let old_children = children.clone();
                let new_children: SmallVec<[PolytopeId; 4]> = old_children
                    .iter()
                    .copied()
                    .filter(|&child| match self.slice_polytope(child, pole) {
                        SliceResult::Unknown => panic!("polytope didn't get slice result computed"),
                        SliceResult::Kept => true,
                        SliceResult::Removed => false,
                        SliceResult::Modified(intersection) => {
                            intersection_boundary.push(intersection);
                            true
                        }
                    })
                    .collect();

                let removed = new_children.len() == 0;
                *self[p].unwrap_children_mut() = new_children;

                if removed {
                    SliceResult::Removed
                } else if old_children
                    .iter()
                    .all(|&child| self[child].slice_result == SliceResult::Kept)
                {
                    SliceResult::Kept
                } else {
                    let new_child = if rank == 1 {
                        let a = self[old_children[0]].unwrap_point();
                        let b = self[old_children[1]].unwrap_point();
                        let a_distance = (pole - a).dot(pole);
                        let b_distance = -(pole - b).dot(pole);
                        let sum = a_distance + b_distance;
                        self.push_point((b * a_distance + a * b_distance) / sum)
                    } else {
                        self.push_polytope(intersection_boundary)
                    };
                    self[new_child].slice_result = SliceResult::Kept;
                    self.add_child(p, new_child);
                    SliceResult::Modified(new_child)
                }
            }
        };
        self[p].slice_result = ret;
        ret
    }
}

#[derive(Debug, Clone, PartialEq)]
struct Polytope {
    parents: SmallVec<[PolytopeId; 4]>,
    contents: PolytopeContents,
    slice_result: SliceResult,
}
impl Polytope {
    fn rank(&self) -> u8 {
        self.contents.rank()
    }
    fn unwrap_point(&self) -> &Vector<f32> {
        match &self.contents {
            PolytopeContents::Point(point) => point,
            _ => panic!("expected point, got branch"),
        }
    }
    fn children(&self) -> &[PolytopeId] {
        match &self.contents {
            PolytopeContents::Point(_) => &[],
            PolytopeContents::Branch { children, .. } => children,
        }
    }
    fn unwrap_children_mut(&mut self) -> &mut SmallVec<[PolytopeId; 4]> {
        match &mut self.contents {
            PolytopeContents::Point(_) => panic!("expected brancch, got point"),
            PolytopeContents::Branch { children, .. } => children,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
enum PolytopeContents {
    Point(Vector<f32>),
    Branch {
        rank: u8,
        children: SmallVec<[PolytopeId; 4]>,
    },
}
impl PolytopeContents {
    fn rank(&self) -> u8 {
        match self {
            PolytopeContents::Point(_) => 0,
            PolytopeContents::Branch { rank, .. } => *rank,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
struct PolytopeId(u32);

#[derive(Debug, Clone, PartialEq)]
pub struct Polygon {
    pub verts: Vec<Vector<f32>>,
}

struct ConvexPolytope {
    verts: Vec<Vector<f32>>,
    faces: Vec<Vec<u32>>,
}

fn base_3_expansion(n: u32, digit_count: u8) -> impl Iterator<Item = u32> {
    std::iter::successors(Some(n), |x| Some(x / 3))
        .take(digit_count as _)
        .map(|x| x % 3)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cube() {
        panic!();
    }
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Hash)]
enum SliceResult {
    /// The slice result hasn't been computed yet.
    #[default]
    Unknown,

    /// The entire polytope was kept by the slice.
    Kept,
    /// The entire polytope was removed by the slice.
    Removed,
    /// The polytope was modified by the slice, and this is the intersection of
    /// the polytope and the slicing hyperplane.
    Modified(PolytopeId),
}
