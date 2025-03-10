use crate::math::Vector3;
use crate::object::CustomShape;

#[derive(Debug, Clone)]
pub struct Triangle {
    pub vertices: [Vector3; 3],
}
impl Triangle {
    /// creates a new triangle
    pub fn new(vertices: [Vector3; 3]) -> Self {
        Self { vertices }
    }
}
impl Triangle {
    /// generates the vectors representing the plane as (basis, direction 1, direction 2)
    fn plane_vectors(&self) -> (Vector3, Vector3, Vector3) {
        let basis = self.vertices[0];
        let dir1 = self.vertices[1] - basis;
        let dir2 = self.vertices[2] - basis;
        (basis, dir1, dir2)
    }
    fn plane_distance(&self, ray: (Vector3, Vector3)) -> f64 {
        let (ray_pos, dir) = ray;
        let dir = dir.norm();
        let normal = self.normal(Vector3::zeros());
        let self_pos = self.vertices[0];
        if dir.dot(normal) == 0.0 {
            return f64::INFINITY;
        }
        normal.dot(self_pos - ray_pos) / dir.dot(normal)
    }

    pub fn contains(&self, point: Vector3) -> bool {
        let (pos, r, s) = self.plane_vectors();
        let p = point - pos;
        // let lgs1 = ((r.x, s.x, p.x),
        //             (r.y, s.y, p.y));
        //
        // let factor1 = lgs1.1.0 / lgs1.0.0;
        // let lgs2 = ((1f64, s.x / (r.x), p.x / r.x),
        //             (0, s.y - factor1 * s.x, p.y - factor1 * p.x));
        //
        // let factor2 = lgs2.1.1 / lgs2.0.1;
        // let lgs3 = ((1f64, 0, lgs2.0.2 - factor2 * lgs2.1.2),
        //             (0, 1, lgs2.1.2 / lgs2.1.1));
        //
        // let a = lgs3.0.2;
        // let b = lgs3.1.2;
        let mut lgs1 = Vector3::new(r.x, s.x, p.x);
        let mut lgs2 = Vector3::new(r.y, s.y, p.y);
        let mut lgs3 = Vector3::new(r.z, s.z, p.z);

        if lgs1.x == 0.0 {
            // println!("{lgs1}, {lgs2}, {lgs3}");
            if lgs2.x == 0.0 {
                if lgs3.x == 0.0 {
                    eprintln!("can't handle LGS"); // TODO
                    return false;
                }
                (lgs3, lgs1) = (lgs1, lgs3);
            } else {
                (lgs1, lgs2) = (lgs2, lgs1)
            }
        }
        lgs1 /= lgs1.x;
        lgs2 -= lgs1 * (lgs2.x / lgs1.x);
        lgs3 -= lgs1 * (lgs3.x / lgs1.x);
        // print!("{:?} => ", (lgs1, lgs2, lgs3));
        // lgs2.x & lgs3.x = 0
        if lgs2.y == 0.0 {
            if lgs3.y == 0.0 {
                eprintln!("can't handle LGS"); // TODO
                return false;
            }
            (lgs2, lgs3) = (lgs3, lgs2);
        }
        lgs2 /= lgs2.y;
        lgs1 -= lgs1 * (lgs1.y / lgs2.y);
        lgs3 -= lgs3 * (lgs3.y / lgs2.y);

        let (a, b) = (lgs1.z, lgs2.z);

        // println!("{:?} => {:?}",(lgs1, lgs2, lgs3), (a, b));
        // print!("{a} {b}; ");
        0. <= a && a <= 1. && 0. <= b && b <= 1. && (a+b) <= 1. && (lgs3.z.abs() < 1e10)
    }
}
impl CustomShape for Triangle {
    fn normal(&self, _world_position: Vector3) -> Vector3 {
        let (_, a, b) = self.plane_vectors();
        a.cross(b).norm()
    }
    fn distance(&self, pos: Vector3, dir: Vector3) -> Option<f64> {
        if self.normal(Vector3::zeros()).dot(dir) >= 0.0 {
            // println!("triangle angled the wrong way");
            return None;
        }
        let distance = self.plane_distance((pos, dir)).abs();
        if distance == f64::INFINITY {
            return None
        }
        let hit_point = pos + dir * distance;
        if !self.contains(hit_point) {
            return None;
        }
        Some(distance)
    }
}