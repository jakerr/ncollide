//! Utilities useful for various generations tasks.

use std::num::Zero;
use std::collections::HashMap;
use std::collections::hashmap::{Occupied, Vacant};
use std::mem;
use na;
use na::{Pnt3, Vec3, Dim, Cross, Orig};
use utils::{HashablePartialEq, AsBytes};
use math::{Scalar, Point, Vect};

// FIXME: remove that in favor of `push_xy_circle` ?
/// Pushes a discretized counterclockwise circle to a buffer.
#[inline]
pub fn push_circle<N: Scalar>(radius: N, nsubdiv: u32, dtheta: N, y: N, out: &mut Vec<Pnt3<N>>) {
    let mut curr_theta: N = na::zero();

    for _ in range(0, nsubdiv) {
        out.push(Pnt3::new(curr_theta.cos() * radius, y.clone(), curr_theta.sin() * radius));
        curr_theta = curr_theta + dtheta;
    }
}

/// Pushes a discretized counterclockwise circle to a buffer.
/// The circle is contained on the plane spanned by the `x` and `y` axis.
#[inline]
pub fn push_xy_arc<N, P>(radius: N, nsubdiv: u32, dtheta: N, out: &mut Vec<P>)
    where N: Scalar,
          P: Dim + Orig + Index<uint, N> + IndexMut<uint, N> {
    assert!(na::dim::<P>() >= 2);

    let mut curr_theta: N = na::zero();

    for _ in range(0, nsubdiv) {
        let mut pt = na::orig::<P>();

        pt[0] = curr_theta.cos() * radius;
        pt[1] = curr_theta.sin() * radius;
        out.push(pt);

        curr_theta = curr_theta + dtheta;
    }
}

/// Creates the faces from two circles with the same discretization.
#[inline]
pub fn push_ring_indices(base_lower_circle: u32,
                         base_upper_circle: u32,
                         nsubdiv:           u32,
                         out:               &mut Vec<Vec3<u32>>) {
    push_open_ring_indices(base_lower_circle, base_upper_circle, nsubdiv, out);

    // adjust the last two triangles
    push_rectangle_indices(base_upper_circle, base_upper_circle + nsubdiv - 1,
                           base_lower_circle, base_lower_circle + nsubdiv - 1, out);
}

/// Creates the faces from two circles with the same discretization.
#[inline]
pub fn push_open_ring_indices(base_lower_circle: u32,
                              base_upper_circle: u32,
                              nsubdiv:           u32,
                              out:               &mut Vec<Vec3<u32>>) {
    assert!(nsubdiv > 0);

    for i in range(0, nsubdiv - 1) {
        let bli = base_lower_circle + i;
        let bui = base_upper_circle + i;
        push_rectangle_indices(bui + 1, bui,
                               bli + 1, bli, out);
    }
}

/// Creates the faces from a circle and a point that is shared by all triangle.
#[inline]
pub fn push_degenerate_top_ring_indices(base_circle: u32,
                                        point:       u32,
                                        nsubdiv:     u32,
                                        out:         &mut Vec<Vec3<u32>>) {
    push_degenerate_open_top_ring_indices(base_circle, point, nsubdiv, out);

    out.push(Vec3::new(base_circle + nsubdiv - 1, point, base_circle));
}

/// Creates the faces from a circle and a point that is shared by all triangle.
#[inline]
pub fn push_degenerate_open_top_ring_indices(base_circle: u32,
                                             point:       u32,
                                             nsubdiv:     u32,
                                             out:         &mut Vec<Vec3<u32>>) {
    assert!(nsubdiv > 0);

    for i in range(0, nsubdiv - 1) {
        out.push(Vec3::new(base_circle + i, point, base_circle + i + 1));
    }
}

/// Pushes indices so that a circle is filled with triangles. Each triangle will have the
/// `base_circle` point in common.
/// Pushes `nsubdiv - 2` elements to `out`.
#[inline]
pub fn push_filled_circle_indices(base_circle: u32, nsubdiv: u32, out: &mut Vec<Vec3<u32>>) {
    for i in range(base_circle + 1, base_circle + nsubdiv - 1) {
        out.push(Vec3::new(base_circle, i, i + 1));
    }
}

/// Given four corner points, pushes to two counterclockwise triangles to `out`.
///
/// # Arguments:
/// * `ul` - the up-left point.
/// * `dl` - the down-left point.
/// * `dr` - the down-left point.
/// * `ur` - the up-left point.
#[inline]
pub fn push_rectangle_indices<T: Clone>(ul: T, ur: T, dl: T, dr: T, out: &mut Vec<Vec3<T>>) {
    out.push(Vec3::new(ul.clone(), dl, dr.clone()));
    out.push(Vec3::new(dr        , ur, ul));
}

/// Reverses the clockwising of a set of faces.
#[inline]
pub fn reverse_clockwising(indices: &mut [Vec3<u32>]) {
    for i in indices.iter_mut() {
        mem::swap(&mut i.x, &mut i.y);
    }
}

/// Duplicates the indices of each triangle on the given index buffer.
///
/// For example: [ (0.0, 1.0, 2.0) ] becomes: [ (0.0, 0.0, 0.0), (1.0, 1.0, 1.0), (2.0, 2.0, 2.0)].
#[inline]
pub fn split_index_buffer(indices: &[Vec3<u32>]) -> Vec<Vec3<Vec3<u32>>> {
    let mut resi = Vec::new();

    for vertex in indices.iter() {
        resi.push(
            Vec3::new(
                Vec3::new(vertex.x, vertex.x, vertex.x),
                Vec3::new(vertex.y, vertex.y, vertex.y),
                Vec3::new(vertex.z, vertex.z, vertex.z)
                )
            );
    }

    resi
}

/// Duplicates the indices of each triangle on the given index buffer, giving the same id to each
/// identical vertex.
#[inline]
pub fn split_index_buffer_and_recover_topology<P: PartialEq + AsBytes + Clone>(
                                               indices: &[Vec3<u32>],
                                               coords:  &[P])
                                               -> (Vec<Vec3<Vec3<u32>>>, Vec<P>) {
    let mut vtx_to_id  = HashMap::new();
    let mut new_coords = Vec::with_capacity(coords.len());
    let mut out        = Vec::with_capacity(indices.len());

    fn resolve_coord_id<P: PartialEq + AsBytes + Clone>(
                        coord:      &P,
                        vtx_to_id:  &mut HashMap<HashablePartialEq<P>, u32>,
                        new_coords: &mut Vec<P>)
                        -> u32 {
        let key = unsafe { HashablePartialEq::new(coord.clone()) };
        let id = match vtx_to_id.entry(key) {
            Occupied(entry) => entry.into_mut(),
            Vacant(entry)   => entry.set(new_coords.len() as u32)
        };

        if *id == new_coords.len() as u32 {
            new_coords.push(coord.clone());
        }

        *id
    }

    for t in indices.iter() {
        let va = resolve_coord_id(&coords[t.x as uint], &mut vtx_to_id, &mut new_coords);
        let oa = t.x;

        let vb = resolve_coord_id(&coords[t.y as uint], &mut vtx_to_id, &mut new_coords);
        let ob = t.y;

        let vc = resolve_coord_id(&coords[t.z as uint], &mut vtx_to_id, &mut new_coords);
        let oc = t.z;

        out.push(
            Vec3::new(
                Vec3::new(va, oa, oa),
                Vec3::new(vb, ob, ob),
                Vec3::new(vc, oc, oc)
                )
            );
    }

    new_coords.shrink_to_fit();

    (out, new_coords)
}

/// Computes the normals of a set of vertices.
#[inline]
pub fn compute_normals<N, P, V>(coordinates: &[P], faces: &[Vec3<u32>], normals: &mut Vec<V>)
    where N: Scalar,
          P: Point<N, V>,
          V: Vect<N> + Cross<V> {
    let mut divisor: Vec<N> = Vec::from_elem(coordinates.len(), na::zero());

    // Shrink the output buffer if it is too big.
    if normals.len() > coordinates.len() {
        normals.truncate(coordinates.len())
    }

    // Reinit all normals to zero.
    normals.clear();
    normals.grow(coordinates.len(), na::zero());

    // Accumulate normals ...
    for f in faces.iter() {
        let edge1  = coordinates[f.y as uint] - coordinates[f.x as uint];
        let edge2  = coordinates[f.z as uint] - coordinates[f.x as uint];
        let cross  = na::cross(&edge1, &edge2);
        let normal;

        if !cross.is_zero() {
            normal = na::normalize(&cross)
        }
        else {
            normal = cross
        }

        normals[f.x as uint] = normals[f.x as uint] + normal;
        normals[f.y as uint] = normals[f.y as uint] + normal;
        normals[f.z as uint] = normals[f.z as uint] + normal;

        divisor[f.x as uint] = divisor[f.x as uint] + na::one();
        divisor[f.y as uint] = divisor[f.y as uint] + na::one();
        divisor[f.z as uint] = divisor[f.z as uint] + na::one();
    }

    // ... and compute the mean
    for (n, divisor) in normals.iter_mut().zip(divisor.iter()) {
        *n = *n / *divisor
    }
}
