use serde::{Deserialize, Serialize};

pub trait Scaled {
    fn scalar(&self) -> f64;
}

pub trait Calculable {
    fn calc(&self, x: f64) -> f64;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Criteria {
    pub pressure_alt: f64,
    pub temp_c: f64,
    pub take_off_weight: f64,
    pub headwind: f64
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PerformanceData {
    pub ground_roll: i64,
    pub vr: i64,
}


pub fn performance(iso_temp_c: f64, pressure_alt: f64, take_off_weight: f64, wind: f64) -> PerformanceData {
    let iso_temp_f = lin(1.8, 32.0, iso_temp_c);
    // First calc
    let curves = [QuadCurve {
        scalar: 0.0,
        a: -184.484, 
        b: 0.024538,
        c: 283.3
    }, QuadCurve {
        scalar: 2000.0,
        a: -148.028,
        b: 0.0362252,
        c: 665.469
    }, QuadCurve {
        scalar: 4000.0,
        a: -216.325,
        b: 0.0318901,
        c: 392.791
    }, QuadCurve {
        scalar: 6000.0,
        a: -235.315,
        b: 0.0353866,
        c: 434.767
    }, QuadCurve {
        scalar: 7000.0,
        a: -175.482, 
        b: 0.051501,
        c: 1064.07
    }];

    let (nearest_low, nearest_high) = search_for_nearest_curves(&curves, pressure_alt);

    let val1 = nearest_low.calc(iso_temp_f);
    let val2 = nearest_high.calc(iso_temp_f);
    let init_roll = interpolate_linear(val1, val2, scale(nearest_low.scalar(), nearest_high.scalar(), pressure_alt));
    let weight_scalar = scale(2000.0, 2750.0, take_off_weight);
    let vr = interpolate_linear(60.0, 71.0, weight_scalar);

    // Second calc
    let weight_curves = [QuadCurve {
        scalar: 3300.0,
        a: -4043.7,
        b: 0.000163585,
        c: -4250.13
    }, QuadCurve {
        scalar: 3000.0,
        a: -473.892,
        b: 0.000325273,
        c: -380.715
    }, QuadCurve {
        scalar: 2500.0,
        a: 25.0,
        b: 0.0003333333,
        c: 24.7917
    }, QuadCurve {
        scalar: 2050.0,
        a: 383.947,
        b: 0.000324786,
        c: 231.78
    }, QuadCurve {
        scalar: 1710.0,
        a: -379.118,
        b: 0.000203106,
        c: -278.691
    }];

    let (first, second) = search_for_nearest_curves(&weight_curves, init_roll);
    let val3 = first.calc(take_off_weight);
    let val4 = second.calc(take_off_weight);
    let adj_roll = interpolate_linear(val3, val4, scale(first.scalar(), second.scalar(), init_roll));

    // Third calc
    let lines = [Line {
        scalar: 1275.0,
        a: -17.0,
        b: 1275.0,
    }, Line {
        scalar: 1725.0,
        a: -22.0,
        b: 1725.0,
    }, Line {
        scalar: 2240.0,
        a: -26.0,
        b: 2240.0,
    }, Line {
        scalar: 2750.0,
        a: -34.0,
        b: 2750.0,
    }, Line {
        scalar: 3350.0,
        a: -40.0,
        b: 3350.0,
    }];

    let (first_l, second_l) = search_for_nearest_curves(&lines, adj_roll);
    let val5 = first_l.calc(wind);
    let val6 = second_l.calc(wind);
    let final_roll = interpolate_linear(val5, val6, scale(first_l.scalar(), second_l.scalar(), adj_roll));
    return PerformanceData { ground_roll: final_roll.round() as i64, vr: vr.round() as i64};
}

fn search_for_nearest_curves<T: Scaled>(curves: &[T], scalar: f64) -> (&T, &T) {

    if curves.len() < 2 {
        panic!("Cannot interpolate fewer than 2 weight_curves")
    }

    let mut smallest = &curves[0];
    let mut second_smallest = &curves[1];
    for x in curves {
        if (scalar - x.scalar()).abs() < (scalar - smallest.scalar()).abs() {
            second_smallest = smallest;
            smallest = &x;
        }
    }
    (smallest, second_smallest)
}

impl Calculable for Line {
    fn calc(&self, x: f64) -> f64 {
        lin(self.a, self.b, x)
    }
}

#[derive(Debug)]
#[derive(PartialEq)]
pub struct Line {
    pub scalar: f64,
    pub a: f64,
    pub b: f64,
}

impl Scaled for Line {
    fn scalar(&self) -> f64 {
        self.scalar
    }
}

pub fn interpolate_linear(first: f64, second: f64, scalar: f64) -> f64 {
    (second - first) * scalar + first
}

pub fn scale(first: f64, second: f64, val: f64) -> f64 {
    (val - first) / (second - first)
}

pub fn lin(a: f64, b: f64, x: f64) -> f64 {
    a * x + b
}

#[derive(Debug)]
#[derive(PartialEq)]
pub struct QuadCurve {
    pub scalar: f64,
    pub a: f64,
    pub b: f64,
    pub c: f64,
}

impl Scaled for QuadCurve {
    fn scalar(&self) -> f64 {
        self.scalar
    }
}

impl Calculable for QuadCurve {
    fn calc(&self, x: f64) -> f64 {
        self.b * (x - self.a).powi(2) + self.c
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SMALLEST: QuadCurve = QuadCurve { scalar: -3.0, a: 1.0, b: 1.0, c: 1.0 };
    const MIDDLE: QuadCurve = QuadCurve { scalar: -2.0, a: 1.0, b: 1.0, c: 1.0 };
    const LARGEST: QuadCurve = QuadCurve { scalar: 4.0, a: 1.0, b: 1.0, c: 1.0 };
    const X_LARGEST: QuadCurve = QuadCurve { scalar: 400.0, a: 1.0, b: 1.0, c: 1.0 };

    #[test]
    fn test_search_for_nearest_curves_order() {
        let ordered = [SMALLEST, MIDDLE, LARGEST];
        let (first, second) = search_for_nearest_curves(&ordered, -6.0);
        assert_eq!(first, &SMALLEST);
        assert_eq!(second, &MIDDLE);
    }

    #[test]
    fn test_search_for_nearest_curves_two_in_wrong_order() {
        let ordered = [LARGEST, SMALLEST];
        let (first, second) = search_for_nearest_curves(&ordered, -6.0);
        assert_eq!(first, &SMALLEST);
        assert_eq!(second, &LARGEST);
    }

    #[test]
    fn test_search_for_nearest_curves_unordered() {
        let ordered = [MIDDLE, LARGEST, SMALLEST, X_LARGEST];
        let (first, second) = search_for_nearest_curves(&ordered, -6.0);
        assert_eq!(first, &SMALLEST);
        assert_eq!(second, &MIDDLE);
    }

    #[test]
    fn test_search_for_nearest_curves_reverse() {
        let ordered = [X_LARGEST, LARGEST, MIDDLE, SMALLEST];
        let (first, second) = search_for_nearest_curves(&ordered, -6.0);
        assert_eq!(first, &SMALLEST);
        assert_eq!(second, &MIDDLE);
    }

    #[test]
    fn test_scale_in_range() {
        assert_eq!(scale(5.0, 10.0, 7.5), 0.5);
    }

    #[test]
    fn test_scale_out_of_range() {
        assert_eq!(scale(5.0, 10.0, 2.5), -0.5);
    }

    #[test]
    fn test_quad() {
        let curve = QuadCurve {
            scalar: 1.0,
            a: -4044.0, 
            b: 0.0001636, 
            c: -7550.0
        };
        assert_eq!(curve.calc(2750.0), 1.5201295999995637);
    }
}