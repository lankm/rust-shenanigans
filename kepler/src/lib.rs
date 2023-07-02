/* README
 * This is a set of orbital parameters. Currently everything is floats, but
 * This should change to fixed point numbers due to the loss of precision.
 * Space has no center so it should not be more accurate depending on the time
 * or location.
 */
#![allow(non_snake_case)]
#![allow(unused)]

pub mod angle;

use std::{f64::consts::PI, char::MAX};

pub fn rot_x(pos: (f64, f64, f64), angle: f64) -> (f64, f64, f64) {
    let x     = pos.0;
    let mut y = pos.1;
    let mut z = pos.2;

    let r = y.hypot(z);         // radius from axis
    let mut theta = z.atan2(y); // angle from plane
    theta += angle;                  // applying rotating

    y = r*theta.cos();
    z = r*theta.sin();
    return ( x, y, z );
}
pub fn rot_y(pos: (f64, f64, f64), angle: f64) -> (f64, f64, f64) {
    let mut x = pos.0;
    let y     = pos.1;
    let mut z = pos.2;
        
    let r = x.hypot(z);         // radius from axis
    let mut theta = x.atan2(z); // angle from axis
    theta += angle;                  // applying rotating

    z = r*theta.cos();
    x = r*theta.sin();
    return ( x, y, z );
}
pub fn rot_z(pos: (f64, f64, f64), angle: f64) -> (f64, f64, f64) {
    let mut x = pos.0;
    let mut y = pos.1;          // coordinates
    let z     = pos.2;
        
    let r = x.hypot(y);         // radius from axis
    let mut theta = y.atan2(x); // angle from axis
    theta += angle;                  // applying rotating
    
    x = r*theta.cos();
    y = r*theta.sin();               // calculating change
    return ( x, y, z );
}

pub struct Orbit {
    pub e: f64, // eccentricity                     0-1
    pub a: f64, // semimajor axis
    pub b: f64, // semiminor axis

    pub i: f64, // inclination                      0-pi
    pub o: f64, // longitude of the ascending node  0-2pi
    pub w: f64, // argument of periapsis            0-2pi

    pub t0: f64, // time of periapsis passage
}
impl Orbit {
    pub fn new( e: f64, a: f64, i: f64, o: f64, w: f64, t0: f64 ) -> Orbit {
        return Orbit { 
            e, 
            a, 
            b: Orbit::b(e, a), 
            i, 
            o, 
            w,
            t0,
        }
    }

    /* E calculation
     * Because the result in undeterministic, more work went into this function.
     * Stable for all e. Yay!!
     * max average steps: 4.87125 (e = 1, M = 0-2PI)
     * min average steps: 0.00000 (e = 0)
     * max steps needed:  N/A     (e = 1, M = 0)
     * min steps needed:  0       (e = 0)
     * Very slowly starts to break if e=1 and M=0 as expected.
     */
    fn E(&self, M: f64) -> f64 {
        const PRECISION: f64 = 9e-16;   // min stable number
        const MAX_ITER: u32 = 100;      // if e = ~1 and M = ~0
        let mut E: f64 = M % (2.0*PI);  // initial estimate

        for i in 0..MAX_ITER {
            let E_next = M + self.e*E.sin(); // calculate next guess
            let difference = E_next-E;
            
            if difference.abs() < PRECISION {
                return E_next;
            } else {
                let step_mult = 1.0 / (1.0-self.e*E.cos()); // derivitive.
                E = E + step_mult*( difference )%(1.4);          // 1.4 causes the best results. Only god knows why.
            }
        }

        return E;
    }
    pub fn pos(&self, M: f64) -> (f64, f64, f64) {
        let E = self.E(M);
        let mut pos = self.pos_elliptic(E);
        pos = rot_z(pos, self.w); // apply argument of periapsis
        pos = rot_x(pos, self.i); // apply inclination
        pos = rot_z(pos, self.o); // apply longitude of the ascending node
        return (pos.0 as f64, pos.1 as f64, pos.2 as f64);
    }
    fn pos_elliptic(&self, E: f64) -> ( f64, f64, f64 ) {
        // reference direction is +x
        // 'up' is +z
        let x = self.a*(E.cos()-self.e);
        let y = self.b*E.sin();

        return ( x, y, 0.0 );
    }

    fn a(e: f64, b: f64) -> f64 {
        return b*( 1.0/(1.0-e*e) ).sqrt();
    }
    fn b(e: f64, a: f64) -> f64 {
        return a*(1.0-e*e).sqrt();
    }
    fn e(a: f64, b: f64) -> f64 {
        return (1.0-( (b*b)/(a*a) )).sqrt();
    }
    
    pub fn apoapsis(&self) -> f64 {
        return self.a - self.periapsis();
    }
    pub fn periapsis(&self) -> f64 {
        return ( self.a - self.a*self.e )/2.0
    }
}

pub struct Stat {
    pub total: f64,
    pub count: u64,
    pub max: f64,
    pub min: f64,
}
impl Stat {
    pub fn new() -> Stat{
        return Stat { total: 0.0, count: 0, max: f64::MIN, min : f64::MAX };
    }
    pub fn entry(&mut self, val: f64, count: Option<u64>) {
        self.total += val;
        self.count += count.unwrap_or(1);
        self.max = if val > self.max {val} else {self.max};
        self.min = if val < self.min {val} else {self.min};
    }
    pub fn mean(&self) -> f64 {
        return self.total / self.count as f64;
    } 
}