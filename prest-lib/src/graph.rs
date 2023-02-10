use std::hash::Hash;
use std::collections::HashMap;

pub struct Graph<V> {
    pub vertices : Vec<V>,
    pub edges : Vec<(usize, usize)>,
}

impl<V> Graph<V> {
    pub fn empty() -> Self {
        Graph {
            vertices : Vec::new(),
            edges : Vec::new(),
        }
    }

    pub fn from_vertices_edges<I>(vertices : &[V], edges : &mut I) -> Self
        where V : Clone+Eq+Hash, I : Iterator<Item=(V,V)>
    {
        let vertices = Vec::from(vertices);
        let indices : HashMap<&V, usize> = vertices.iter().enumerate().map(
            |(i,v)| (v,i)
        ).collect();
        let edges : Vec<(usize, usize)> = edges.map(
            |(p, q)| (*indices.get(&p).unwrap(), *indices.get(&q).unwrap())
        ).collect();

        Graph {
            vertices,
            edges,
        }
    }

    pub fn iter_isolated_vertices(self : &Self) -> impl Iterator<Item=&V> {
        let mut boolmap = vec![true; self.vertices.len()];  // make all vertices isolated at first
        for &(p, q) in &self.edges {
            // mark these vertices as not isolated
            boolmap[p] = false;
            boolmap[q] = false;
        }

        boolmap.into_iter().zip(&self.vertices).filter_map(
            |(is_isolated, v_ref)| if is_isolated {
                Some(v_ref)
            } else {
                None
            }
        )
    }
}
