//! Histograma de componentes de demanda para un edificio, planta o zona
//!
//! Las pérdidas o ganancias se definen mediante las 8 categorías de HULC más el total

use gtk::WidgetExt;

use plotters::prelude::*;
use plotters_cairo::CairoBackend;

const LABELS: [&str;9] = [
    "Paredes Exteriores",
    "Cubiertas",
    "Suelos",
    "Puentes Térmicos",
    "Solar Ventanas",
    "Transmisión Ventanas",
    "Fuentes Internas",
    "Ventilación más Infiltración",
    "TOTAL",
];

// TODO: pintar gráficas en gtkdrawingarea:
// con https://github.com/38/plotters
// TODO: ver https://stackoverflow.com/questions/10250748/draw-an-image-on-drawing-area
// https://github.com/GuillaumeGomez/process-viewer/blob/master/src/graph.rs

/// Representa histograma de composición de demanda (demandas netas y por componentes): calpos, calneg, calnet, refpos, refneg, refnet
///
/// El eje horizontal representa los componentes de demanda y el eje vertical la demanda anual para el mismo [kWh/m²a]
pub fn draw_histocomponentes(widget: &gtk::DrawingArea, cr: &cairo::Context, cal_net: &[f64;9], ref_net: &[f64;9]) {
    let cal_min = cal_net.iter().map(|v| v.round() as i32).min().map(|m| m - 10).unwrap_or(-30_i32);
    let ref_min = ref_net.iter().map(|v| v.round() as i32).min().map(|m| m - 10).unwrap_or(-30_i32);
    let cal_max = cal_net.iter().map(|v| v.round() as i32).max().map(|m| m + 10).unwrap_or(30_i32);
    let ref_max = ref_net.iter().map(|v| v.round() as i32).max().map(|m| m + 10).unwrap_or(30_i32);
    let min = cal_min.min(ref_min);
    let max = cal_max.max(ref_max);


    let rect = widget.get_allocation();
    let root = CairoBackend::new(cr, (rect.width as u32, rect.height as u32))
        .unwrap()
        .into_drawing_area();
    root.fill(&WHITE).unwrap();
    //let root = root.margin(25, 25, 25, 25);

    let mut ctx = ChartBuilder::on(&root)
        .caption("Demandas por componente", ("sans-serif", 20))
        .y_label_area_size(65)
        .x_label_area_size(40)
        .margin(25)
        .build_cartesian_2d((0..17).into_segmented(), min..max)
        .unwrap();
    ctx.configure_mesh()
        .x_desc("Componente")
        .y_desc("Demanda [kWh/m²·año]")
        .axis_desc_style(("sans-serif", 15))
        .draw()
        .unwrap();

    ctx.draw_series((0..).map(|i| 2*i).zip(cal_net.iter()).map(|(x, y)| {
        let x0 = SegmentValue::Exact(x);
        let x1 = SegmentValue::Exact(x + 1);
        let mut bar = Rectangle::new([(x0, 0), (x1, y.round() as i32)], RED.filled());
        bar.set_margin(0, 0, 5, 5);
        bar
    }))
    .unwrap();
    ctx.draw_series((0..).map(|i| 2*i + 1).zip(ref_net.iter()).map(|(x, y)| {
        let x0 = SegmentValue::Exact(x);
        let x1 = SegmentValue::Exact(x + 1);
        let mut bar = Rectangle::new([(x0, 0), (x1, y.round() as i32)], BLUE.filled());
        bar.set_margin(0, 0, 5, 5);
        bar
    }))
    .unwrap();
}

// // to limit line "fuzziness"
// #[inline]
// fn rounder(x: f64) -> f64 {
//     let fract = x.fract();
//     if fract < 0.5 {
//         x.trunc() + 0.5
//     } else {
//         x.trunc() + 1.5
//     }
// }

// let x_start = 0.0;
// cr.set_source_rgb(0.5, 0.5, 0.5);
// // We always draw 10 lines (12 if we count the borders).
// let x_step = (width - x_start) / 12.;
// let mut current = width - width / 12.;
// if x_step < 0.1 {
//     cr.stroke();
//     return;
// }

// while current > x_start {
//     cr.move_to(rounder(current), 0.0);
//     cr.line_to(rounder(current), height);
//     current -= x_step;
// }
// let step = height / 10.0;
// current = step - 1.0;
// while current < height - 1. {
//     cr.move_to(x_start, rounder(current));
//     cr.line_to(width, rounder(current));
//     current += step;
// }
// cr.stroke();
