use crate::math::mat::Mat3x3;
use crate::math::vector::Vector3;

impl Mat3x3 {
    /// creates a new matrix
    pub fn new(x: Vector3, y: Vector3, z: Vector3) -> Self {
        Mat3x3 { x, y, z}
    }
    /// calculates the inverse of the matrix
    pub fn inverse(&self) -> Self {
        let adjugate = self.adjugate();
        let det = self.determinant();
        adjugate / det
    }
    /// transposes the matrix
    pub fn transpose(&self) -> Self {
        let x = Vector3::new(self.x.x, self.y.x, self.z.x);
        let y = Vector3::new(self.x.y, self.y.y, self.z.y);
        let z = Vector3::new(self.x.z, self.y.z, self.z.z);
        Self::new(x,y,z)
    }
    /// calculates the determinant of the matrix
    pub fn determinant(&self) -> f64 {
        // comments assume the following Matrix:
        // [[a,b,c]
        //  [d,e,f]
        //  [g,h,i]]

        // a * e * i
        let sum1 = self.x.x * self.y.y * self.z.z;
        // b * f * g
        let sum2 = self.x.y * self.y.z * self.z.x;
        // c * d * h
        let sum3 = self.x.z * self.y.x * self.z.y;

        // g * e * c
        let sub1 = self.z.x * self.y.y * self.x.z;
        // h * f * a
        let sub2 = self.z.y * self.y.z * self.x.x;
        // i * d * b
        let sub3 = self.z.z * self.y.x * self.x.y;
        (sum1 + sum2 + sum3) - (sub1 + sub2 + sub3)
    }
    /// calculates the adjugate of the matrix
    pub fn adjugate(&self) -> Self {
        let a = self.x.x;
        let b = self.x.y;
        let c = self.x.z;
        let d = self.y.x;
        let e = self.y.y;
        let f = self.y.z;
        let g = self.z.x;
        let h = self.z.y;
        let i = self.z.z;
        let cofactor_x = Vector3::new(
            e * i - f * h,
            c * h - b * i,
            b * f - c * e,
        );
        let cofactor_y = Vector3::new(
            f * g - d * i,
            a * i - c * g,
            c * d - a * f,
        );
        let cofactor_z = Vector3::new(
            d * h - e * g,
            b * g - a * h,
            a * e - b * d,
        );
        Mat3x3::new(cofactor_x, cofactor_y, cofactor_z)
    }
}