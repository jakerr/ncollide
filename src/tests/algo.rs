use rand;
use na::{Vec1, Vec2, Vec3, Vec4, Vec5, Vec6};
use na;
use narrow_phase::algorithm::johnson_simplex::{JohnsonSimplex, RecursionTemplate};
use narrow_phase::algorithm::simplex::Simplex;
use narrow_phase::algorithm::gjk;
use narrow_phase::closest_points;
use narrow_phase::algorithm::brute_force_simplex::BruteForceSimplex;
use shape::Ball;
use shape::AnnotatedPoint;
use support_map;
use math::{Scalar, Point, Vect};

macro_rules! test_johnson_simplex_impl(
    ($t: ty, $n: ty) => ( {
        let recursion = RecursionTemplate::new(na::dim::<$t>());

        for d in range(0u, na::dim::<$t>() + 1) {
            for i in range(1u, 200 / (d + 1)) {
                // note that this fails with lower precision
                let mut v1: $t = rand::random();
                v1 = v1 - (0.5 as $n);
                v1 = v1 * (i as $n);

                let mut splx1 = JohnsonSimplex::new(recursion.clone());
                splx1.reset(v1.clone());

                let mut splx2 = BruteForceSimplex::new();
                splx2.reset(v1.clone());

                d.times(|| {
                    let mut v: $t = rand::random();
                    v = v - (0.5 as $n);
                    v = v * (i as $n);

                    splx1.add_point(v.clone());
                    splx2.add_point(v);
                });

                let proj2 = splx2.project_origin();
                let proj1 = splx1.project_origin();

                assert!(na::approx_eq(&proj1, &proj2));
            }
        }
    }
  )
)

macro_rules! test_gjk_ball_ball_impl(
    ($t: ty, $n: ty) => ( {
        let recursion   = RecursionTemplate::new(na::dim::<$t>());

        200.times(|| {
            let r1 = 10.0 as $n * rand::random();
            let r2 = 10.0 as $n * rand::random();

            let mut c1: $t = rand::random();
            c1 = c1 - (0.5 as $n);
            c1 = c1 * (100.0 as $n);

            let mut c2: $t = rand::random();
            c2 = c2 - (0.5 as $n);
            c2 = c2 * (100.0 as $n);

            let b1 = Ball::new(r1);
            let b2 = Ball::new(r2);

            let (p1, p2) = closest_points::ball_ball(&c1, &b1, &c2, &b2);

            // FIXME: a bit verbose…
            let cso_point   = support_map::cso_support_point_without_margin(&c1, &b1, &c2, &b2, rand::random());
            let mut simplex: JohnsonSimplex<$n, AnnotatedPoint<$t>> = JohnsonSimplex::new(recursion.clone());

            simplex.reset(cso_point);

            let pts_johnson = gjk::closest_points(&c1, &b1, &c2, &b2, &mut simplex);

            match pts_johnson {
                Some((jp1, jp2)) => assert!(na::approx_eq(&jp1, &p1) && na::approx_eq(&jp2, &p2),
                "found: " + jp1.to_str() + " " + jp2.to_str()
                + " but expected: " + p1.to_str() + p2.to_str()),
                None => assert!(na::dist(&p1, &p2)) <= r1 + r2)
            }
        });
    }
  )
)

#[test]
fn test_gjk_ball_ball_1d() {
    test_gjk_ball_ball_impl!(Vec1<f64>, f64)
}

#[test]
fn test_gjk_ball_ball_2d() {
    test_gjk_ball_ball_impl!(Vec2<f64>, f64)
}

#[test]
fn test_gjk_ball_ball_3d() {
    test_gjk_ball_ball_impl!(Vec3<f64>, f64)
}

#[test]
fn test_gjk_ball_ball_4d() {
    test_gjk_ball_ball_impl!(Vec4<f64>, f64);
}

#[test]
fn test_gjk_ball_ball_5d() {
    test_gjk_ball_ball_impl!(Vec5<f64>, f64);
}

#[test]
fn test_gjk_ball_ball_6d() {
    test_gjk_ball_ball_impl!(Vec6<f64>, f64);
}

#[test]
fn test_johnson_simplex_1d() {
    test_johnson_simplex_impl!(Vec1<f64>, f64);
}

#[test]
fn test_johnson_simplex_2d() {
    test_johnson_simplex_impl!(Vec2<f64>, f64);
}

#[test]
fn test_johnson_simplex_3d() {
    test_johnson_simplex_impl!(Vec3<f64>, f64);
}

#[test]
fn test_johnson_simplex_4d() {
    test_johnson_simplex_impl!(Vec4<f64>, f64);
}

#[test]
fn test_johnson_simplex_5d() {
    test_johnson_simplex_impl!(Vec5<f64>, f64);
}

#[test]
fn test_johnson_simplex_6d() {
    test_johnson_simplex_impl!(Vec6<f64>, f64);
}
