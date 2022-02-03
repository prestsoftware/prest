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
}
