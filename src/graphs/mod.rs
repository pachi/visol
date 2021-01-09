pub mod piechart;
pub mod histomeses;
pub mod histocomponentes;
pub mod horarioszona;

/// Traduce del dominio [x1, x2] al rango [x1, x2]
pub fn linear_scale(domx1: f64, domx2: f64, rangex1: f64, rangex2: f64) -> impl Fn(f64) -> f64 {
    let denom = domx2 - domx1;
    assert!(denom.abs() > f64::EPSILON);
    let m = (rangex2 - rangex1) / denom;
    move |x: f64| (x - domx1) * m + rangex1
}
