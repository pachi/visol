//! Histograma de demanda mensual para una zona o el edificio
//!
//! Las pérdidas o ganancias se definen mediante las 8 categorías de HULC más el total

use std::f64::consts::PI;

use gtk::WidgetExt;

// Pintar gráficas en gtkdrawingarea:
// Ejemplos en: https://stackoverflow.com/questions/10250748/draw-an-image-on-drawing-area
// https://github.com/GuillaumeGomez/process-viewer/blob/master/src/graph.rs

/// Meses del año
const MESES: [&str; 12] = [
    "Ene", "Feb", "Mar", "Abr", "May", "Jun", "Jul", "Ago", "Sep", "Oct", "Nov", "Dic",
];

/// Traduce del dominio [x1, x2] al rango [x1, x2]
fn linear_scale(domx1: f64, domx2: f64, rangex1: f64, rangex2: f64) -> impl Fn(f64) -> f64 {
    let denom = domx2 - domx1;
    assert!(denom.abs() > f64::EPSILON);
    let m = (rangex2 - rangex1) / denom;
    move |x: f64| (x - domx1) * m + rangex1
}

/// Representa histograma de demanda mensual para una zona o el edificio
///
/// Se incluye la demanda de calefacción (neg) y refrigeración (pos).
///
/// El eje horizontal representa los periodos [meses] y el eje vertical la demanda existente [kWh/m²mes]
/// No está disponible para componentes
pub fn draw_histomeses(
    widget: &gtk::DrawingArea,
    cr: &cairo::Context,
    calefaccion_meses: &[f32],
    refrigeracion_meses: &[f32],
    min: f32,
    max: f32,
) {
    assert!(calefaccion_meses.len() == 12);
    assert!(refrigeracion_meses.len() == 12);

    // Posiciones
    let min = (min / 10.0 - 1.0).round() * 10.0;
    let max = (max / 10.0 + 1.0).round() * 10.0;
    let rect = widget.get_allocation();
    let width = rect.width as f64;
    let height = rect.height as f64;
    let htitulo = 0.1 * height;
    let margin = 0.05 * height;
    let hgrafica = 0.9 * height - 2.0 * margin;
    let wgrafica = width - 4.0 * margin;
    let (og_x, og_y) = (3.0 * margin, 0.1 * height); // Esquina sup. izq.
    let (eg_x, eg_y) = (og_x + wgrafica, og_y + hgrafica); // Esquina inf. der.
    let stepx = wgrafica / 12.0;
    let stepy = hgrafica / (max - min).abs() as f64;
    let title_size = 20.0;
    let normal_size = 14.0;
    let small_size = 11.0;
    let title = "Demanda neta mensual";
    let xlabel = "Mes";
    let ylabel = "Demanda [kWh/m²mes]";
    let ticksize = stepx / 10.0;
    // Escalas lineales de X e Y sobre la gráfica
    let scalex = linear_scale(0.0, 12.0, og_x, eg_x);
    let scaley = linear_scale(min as f64, max as f64, eg_y, og_y);
    let x0 = scalex(0.0);
    let y0 = scaley(0.0);

    // Inciamos dibujo guardando contexto
    cr.save();

    // Fondo
    cr.set_source_rgb(1.0, 1.0, 1.0);
    cr.new_path();
    cr.rectangle(0.0, 0.0, width, height);
    cr.fill();

    // Título
    cr.select_font_face("Arial", cairo::FontSlant::Normal, cairo::FontWeight::Bold);
    cr.set_font_size(title_size);
    cr.set_source_rgb(0.5, 0.5, 0.5);
    let extents = cr.text_extents(title);
    cr.move_to(
        (width - extents.width) / 2.0,
        0.5 * (htitulo + extents.height),
    );
    cr.show_text(title);

    // Rótulos de ejes
    cr.select_font_face("Arial", cairo::FontSlant::Normal, cairo::FontWeight::Normal);

    // YLabel
    cr.set_font_size(normal_size);
    cr.set_source_rgb(0.5, 0.5, 0.5);
    let extents = cr.text_extents(ylabel);
    cr.move_to(margin, og_y + (hgrafica + extents.width) / 2.0);
    cr.save();
    cr.rotate(-PI / 2.0);
    cr.show_text(ylabel);
    cr.restore();
    // XLabel
    let extents = cr.text_extents(xlabel);
    cr.move_to((width - extents.width) / 2.0, height - margin / 2.0);
    cr.show_text(xlabel);
    // Meses
    cr.set_line_width(1.0);
    cr.set_font_size(small_size);
    let extents = cr.text_extents("Sep");
    let (labelw, labelh) = (extents.width, extents.height);

    let mut xpos = og_x + (stepx - labelw) / 2.0;
    let ypos = eg_y + labelh * 2.0;
    cr.move_to(xpos, ypos);
    for label in &MESES {
        cr.show_text(label);
        xpos = xpos + stepx;
        cr.move_to(xpos, ypos);
    }
    // Ticks en x
    cr.set_line_width(1.0);
    cr.set_source_rgb(0.0, 0.0, 0.0);
    for i in 0..13 {
        cr.move_to(og_x + (i as f64) * stepx, eg_y);
        cr.rel_line_to(0.0, ticksize);
        cr.stroke();
    }

    // Línea y = 0
    cr.set_line_width(2.0);
    cr.move_to(x0, y0);
    cr.rel_line_to(wgrafica, 0.0);
    cr.stroke();

    // Líneas de y cada 10 kWh/m2·a
    for i in 0.. {
        let y = og_y + i as f64 * stepy * 10.0;
        if y > eg_y {
            break;
        }
        cr.set_line_width(1.0);
        cr.set_source_rgb(0.0, 0.0, 0.0);
        let txt = format!("{:.0}", max - i as f32 * 10.0);
        let txt_ext = cr.text_extents(&txt);
        cr.move_to(og_x - 2.0 * ticksize - txt_ext.width, y);
        cr.show_text(&txt);
        cr.move_to(og_x - ticksize, y);
        cr.rel_line_to(ticksize, 0.0);
        cr.stroke_preserve();
        cr.set_line_width(0.5);
        cr.set_source_rgb(0.5, 0.5, 0.5);
        cr.rel_line_to(wgrafica, 0.0);
        cr.stroke()
    }

    // Barras calefacción
    for (i, cal) in calefaccion_meses.iter().enumerate() {
        cr.new_path();
        let x = scalex(i as f64);
        let y = scaley(*cal as f64);
        let height = y - y0;
        cr.rectangle(x, y, stepx, -height);
        cr.set_source_rgb(1.0, 0.0, 0.0);
        cr.fill_preserve();
        cr.set_source_rgb(0.0, 0.0, 0.0);
        cr.stroke();
        if cal.abs() >= f32::EPSILON {
            let txt = format!("{:.1}", cal);
            let txt_ext = cr.text_extents(&txt);
            let x_txt = scalex(i as f64 + 0.5) - txt_ext.width / 2.0;
            let y_txt = y + txt_ext.height * 1.5;
            cr.move_to(x_txt, y_txt);
            cr.show_text(&txt);
        }
    }
    // Barras refrigeración
    for (i, refr) in refrigeracion_meses.iter().enumerate() {
        cr.new_path();
        let x = scalex(i as f64);
        let y = scaley(*refr as f64);
        let height = y - y0;
        cr.rectangle(x, y, stepx, -height);
        cr.set_source_rgb(0.0, 0.0, 1.0);
        cr.fill_preserve();
        cr.set_source_rgb(0.0, 0.0, 0.0);
        cr.stroke();
        if refr.abs() > f32::EPSILON {
            let txt = format!("{:.1}", refr);
            let txt_ext = cr.text_extents(&txt);
            let x_txt = scalex(i as f64 + 0.5) - txt_ext.width / 2.0;
            let y_txt = y - txt_ext.height * 0.5;
            cr.move_to(x_txt, y_txt);
            cr.show_text(&txt);
        }
    }
    // Restauramos contexto
    cr.restore();

}
