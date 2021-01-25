use crate::{APP_NAME, APP_VERSION};

pub mod histoconceptos;
pub mod histomeses;
pub mod horarioszona;
pub mod piechart;

const TITLE_SIZE: f64 = 20.0;
const NORMAL_SIZE: f64 = 14.0;
const SMALL_SIZE: f64 = 11.0;
/// Meses del año
const MESES: [&str; 12] = [
    "Ene", "Feb", "Mar", "Abr", "May", "Jun", "Jul", "Ago", "Sep", "Oct", "Nov", "Dic",
];

/// Rounder to limit line "fuzziness"
#[inline]
pub fn rounder(x: f64) -> f64 {
    let fract = x.fract();
    if fract < 0.5 {
        x.trunc() + 0.5
    } else {
        x.trunc() + 1.5
    }
}

/// Traduce del dominio [x1, x2] al rango [x1, x2]
pub fn linear_scale(domx1: f64, domx2: f64, rangex1: f64, rangex2: f64) -> impl Fn(f64) -> f64 {
    let denom = domx2 - domx1;
    assert!(denom.abs() > f64::EPSILON);
    let m = (rangex2 - rangex1) / denom;
    move |x: f64| (x - domx1) * m + rangex1
}

/// Devuelve una lista con tick_count números "bonitos" para una escala que va del valor low al valor high
pub fn nice_range(low: f64, high: f64, tick_count: usize) -> Vec<f64> {
    let range = high - low;
    let unrounded_ticksize = range / (tick_count - 1) as f64;
    let x = (unrounded_ticksize.log10() - 1.0).ceil();
    let pow10x = 10.0_f64.powi(x as i32);
    let rounded_tickrange = (unrounded_ticksize / pow10x).ceil() * pow10x;
    let newlower = rounded_tickrange * (low / rounded_tickrange).round();
    let mut range = vec![];
    for i in 0..(tick_count) {
        range.push(newlower + i as f64 * rounded_tickrange);
    }
    range
}

/// Dibuja marca de agua de la aplicación
pub fn draw_watermark(cr: &cairo::Context, x: f64, y: f64) {
    cr.select_font_face("Arial", cairo::FontSlant::Normal, cairo::FontWeight::Normal);
    cr.set_font_size(SMALL_SIZE);
    cr.set_source_rgb(0.2, 0.2, 0.2);
    let mark = format!(
        "{} v.{} ({})",
        APP_NAME,
        APP_VERSION,
        chrono::Local::today().format("%d-%m-%Y").to_string()
    );
    let ext = cr.text_extents(&mark);
    cr.move_to(x - ext.width, y - 0.25 * ext.height);
    cr.show_text(&mark);
}
