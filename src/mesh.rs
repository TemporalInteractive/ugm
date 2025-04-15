use bytemuck::{Pod, Zeroable};
use glam::{Vec2, Vec3, Vec3Swizzles, Vec4, Vec4Swizzles};
use speedy::{Readable, Writable};

use crate::packing::PackedNormalizedXyz10;

#[derive(Debug, Pod, Clone, Copy, Zeroable, Readable, Writable)]
#[repr(C)]
pub struct PackedVertex {
    pub position: [f32; 3],
    pub normal: PackedNormalizedXyz10,
    pub tex_coord: [f32; 2],
    pub tangent: PackedNormalizedXyz10,
    pub tangent_handiness: f32,
}

#[derive(Debug, Clone, Readable, Writable)]
pub struct Mesh {
    pub packed_vertices: Vec<PackedVertex>,
    pub triangle_material_indices: Vec<u32>,
    pub indices: Vec<u32>,
    pub opaque: bool,
    pub is_emissive: bool,
    pub bounds_min: [f32; 3],
    pub bounds_max: [f32; 3],
}

impl Mesh {
    pub fn new(
        packed_vertices: Vec<PackedVertex>,
        triangle_material_indices: Vec<u32>,
        indices: Vec<u32>,
        opaque: bool,
        is_emissive: bool,
    ) -> Self {
        debug_assert_eq!(triangle_material_indices.len(), indices.len() / 3);

        let mut bounds_min = Vec3::INFINITY;
        let mut bounds_max = Vec3::NEG_INFINITY;
        for vertex in &packed_vertices {
            bounds_min = bounds_min.min(Vec3::from_array(vertex.position));
            bounds_max = bounds_max.max(Vec3::from_array(vertex.position));
        }

        Mesh {
            packed_vertices,
            triangle_material_indices,
            indices,
            opaque,
            is_emissive,
            bounds_min: bounds_min.to_array(),
            bounds_max: bounds_max.to_array(),
        }
    }
}

pub fn pack_vertices(
    vertex_positions: Vec<Vec3>,
    vertex_normals: Vec<Vec3>,
    vertex_tangents: Vec<Vec4>,
    vertex_tex_coords: Vec<Vec2>,
) -> Vec<PackedVertex> {
    let mut packed_vertices = Vec::with_capacity(vertex_positions.len());
    for i in 0..vertex_positions.len() {
        packed_vertices.push(PackedVertex {
            position: vertex_positions[i].to_array(),
            normal: PackedNormalizedXyz10::new(vertex_normals[i]),
            tex_coord: vertex_tex_coords[i].to_array(),
            tangent: PackedNormalizedXyz10::new(vertex_tangents[i].xyz()),
            tangent_handiness: vertex_tangents[i].w,
        });
    }

    packed_vertices
}

pub fn generate_normals(positions: &[Vec3], indices: &[u32]) -> Vec<Vec3> {
    let mut vertex_normals = vec![Vec3::ZERO; positions.len()];

    for i in 0..(indices.len() / 3) {
        let p0 = positions[indices[i * 3] as usize];
        let p1 = positions[indices[i * 3 + 1] as usize];
        let p2 = positions[indices[i * 3 + 2] as usize];
        let n = (p1 - p0).cross(p2 - p0).normalize();

        vertex_normals[indices[i * 3] as usize] += n;
        vertex_normals[indices[i * 3 + 1] as usize] += n;
        vertex_normals[indices[i * 3 + 2] as usize] += n;
    }

    for normal in &mut vertex_normals {
        *normal = normal.normalize();
    }

    vertex_normals
}

pub fn generate_tangents(
    positions: &[Vec3],
    normals: &[Vec3],
    tex_coords: &[Vec2],
    indices: &[u32],
) -> Vec<Vec4> {
    if tex_coords.is_empty() {
        return vec![Vec4::ZERO; positions.len()];
    }

    // Source: 2001. http://www.terathon.com/code/tangent.html
    let mut tan1 = vec![Vec3::default(); positions.len()];
    let mut tan2 = vec![Vec3::default(); positions.len()];

    for i in (0..indices.len()).step_by(3) {
        let i1 = indices[i] as usize;
        let i2 = indices[i + 1] as usize;
        let i3 = indices[i + 2] as usize;

        let v1 = positions[i1].xyz();
        let v2 = positions[i2].xyz();
        let v3 = positions[i3].xyz();

        let w1 = tex_coords[i1];
        let w2 = tex_coords[i2];
        let w3 = tex_coords[i3];

        let x1 = v2.x - v1.x;
        let x2 = v3.x - v1.x;
        let y1 = v2.y - v1.y;
        let y2 = v3.y - v1.y;
        let z1 = v2.z - v1.z;
        let z2 = v3.z - v1.z;

        let s1 = w2.x - w1.x;
        let s2 = w3.x - w1.x;
        let t1 = w2.y - w1.y;
        let t2 = w3.y - w1.y;

        let rdiv = s1 * t2 - s2 * t1;
        let r = if rdiv == 0.0 { 0.0 } else { 1.0 / rdiv };

        let sdir = Vec3::new(
            (t2 * x1 - t1 * x2) * r,
            (t2 * y1 - t1 * y2) * r,
            (t2 * z1 - t1 * z2) * r,
        );

        let tdir = Vec3::new(
            (s1 * x2 - s2 * x1) * r,
            (s1 * y2 - s2 * y1) * r,
            (s1 * z2 - s2 * z1) * r,
        );

        tan1[i1] += sdir;
        tan1[i2] += sdir;
        tan1[i3] += sdir;

        tan2[i1] += tdir;
        tan2[i2] += tdir;
        tan2[i3] += tdir;
    }

    let mut vertex_tangents = vec![Vec4::ZERO; positions.len()];

    for i in 0..positions.len() {
        let n = normals[i];
        let t = tan1[i];

        let xyz = (t - (n * n.dot(t))).normalize();

        let w = if n.cross(t).dot(tan2[i]) < 0.0 {
            -1.0
        } else {
            1.0
        };

        vertex_tangents[i] = Vec4::new(xyz.x, xyz.y, xyz.z, w);
    }

    vertex_tangents
}
