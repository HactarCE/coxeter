use smallvec::{smallvec, SmallVec};
use std::ops::*;

use crate::vector::Vector;

#[test]
fn test_blah() {
    assert_eq!(0, std::mem::size_of::<Polytope>())
}

#[derive(Debug, Default)]
pub struct PolytopeArena {
    polytopes: Vec<Polytope>,
}
impl Index<PolytopeId> for PolytopeArena {
    type Output = Polytope;

    fn index(&self, index: PolytopeId) -> &Self::Output {
        &self.polytopes[index.0 as usize]
    }
}
impl IndexMut<PolytopeId> for PolytopeArena {
    fn index_mut(&mut self, index: PolytopeId) -> &mut Self::Output {
        &mut self.polytopes[index.0 as usize]
    }
}
impl PolytopeArena {
    pub fn new_cube(ndim: u8, radius: f32) -> Self {
        // Based on Andrey Astrelin's implementation of `GenCube()` in MPUlt
        // (FaceCuts.cs)

        let mut ret = Self { polytopes: vec![] };

        let powers_of_3 = || std::iter::successors(Some(1), |x| Some(x * 3));

        // Make a 3^NDIM grid of polytopes to construct a hypercube. The corners
        // are vertices. Between those are edges, etc.
        //
        // ```
        // • - •
        // | # |
        // • - •
        // ```
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

            ret.push(Polytope { parents, contents });
        }

        ret
    }

    fn push(&mut self, polytope: Polytope) -> PolytopeId {
        self.polytopes.push(polytope);
        PolytopeId(self.polytopes.len() as u32 - 1)
    }
    fn push_point(&mut self, point: Vector<f32>) -> PolytopeId {
        self.push(Polytope {
            parents: smallvec![],
            contents: PolytopeContents::Point(point),
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
}

#[derive(Debug, Clone, PartialEq)]
struct Polytope {
    parents: SmallVec<[PolytopeId; 4]>,
    contents: PolytopeContents,
}
impl Polytope {
    fn rank(&self) -> u8 {
        self.contents.rank()
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

#[derive(Debug, Copy, Clone, PartialEq)]
struct PolytopeId(u32);

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
        dbg!(PolytopeArena::new_cube(2, 5.0));
        panic!();
    }
}
