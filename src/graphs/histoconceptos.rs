//! Histograma de componentes de demanda para un edificio, planta o zona
//!
//! Las pérdidas o ganancias se definen mediante las 8 categorías de HULC más el total

use std::f64::consts::PI;

use gtk::WidgetExt;

use crate::parsers::types::FlujosVec;

use super::linear_scale;

const COLOR_RED: (f64, f64, f64) = (1.0, 0.0, 0.0);
const COLOR_RED2: (f64, f64, f64) = (1.0, 0.4, 0.4);
const COLOR_RED3: (f64, f64, f64) = (1.0, 0.6, 0.6);
const COLOR_BLUE: (f64, f64, f64) = (0.0, 0.0, 1.0);
const COLOR_BLUE2: (f64, f64, f64) = (0.4, 0.4, 1.0);
const COLOR_BLUE3: (f64, f64, f64) = (0.6, 0.6, 1.0);

/// Representa histograma de composición de demanda (demandas netas y por componentes): calpos, calneg, calnet, refpos, refneg, refnet
///
/// El eje horizontal representa los componentes de demanda y el eje vertical la demanda anual para el mismo [kWh/m²a]
/// TODO: ver cómo mostrar detalle (igual no hacer detalle con tanta granularidad)
pub fn draw_histoconceptos(
    widget: &gtk::DrawingArea,
    cr: &cairo::Context,
    cur_name: &str,
    flujos: &FlujosVec,
    min: f32,
    max: f32,
    show_detail: bool,
) {
    assert!(flujos.calnet.len() == 9 || flujos.calnet.len() == 1);
    assert!(flujos.refnet.len() == 9 || flujos.refnet.len() == 1);

    let xtitles = if flujos.calnet.len() == 9 {
        vec![
            "Paredes exteriores",
            "Cubiertas",
            "Suelos",
            "Puentes térmicos",
            "Solar ventanas",
            "Transmisión ventanas",
            "Fuentes internas",
            "Ventilación e infiltración",
            "TOTAL",
        ]
    } else {
        vec![cur_name]
    };

    let series = match show_detail {
        false => vec![(&flujos.calnet, COLOR_RED), (&flujos.refnet, COLOR_BLUE)],
        true => vec![
            (&flujos.calnet, COLOR_RED),
            (&flujos.calpos, COLOR_RED2),
            (&flujos.calneg, COLOR_RED3),
            (&flujos.refnet, COLOR_BLUE),
            (&flujos.refpos, COLOR_BLUE2),
            (&flujos.refneg, COLOR_BLUE3),
        ],
    };

    let min = ((min / 10.0 - 1.0).round() * 10.0) as f64;
    let max = ((max / 10.0 + 1.0).round() * 10.0) as f64;

    let title_size = 20.0;
    let normal_size = 14.0;
    let small_size = 11.0;
    let title = "Demandas por componente";
    let ylabel = "Demanda [kWh/m²·año]";
    let numseries = series.len() as f64;

    // Posiciones
    let rect = widget.get_allocation();
    let width = rect.width as f64;
    let height = rect.height as f64;
    let htitulo = 0.1 * height;
    let margin = 0.05 * height;
    let hgrafica = 0.9 * height - 3.0 * margin;
    let wgrafica = width - 4.0 * margin;
    let (og_x, og_y) = (3.0 * margin, 0.1 * height); // Esquina sup. izq.
    let (eg_x, eg_y) = (og_x + wgrafica, og_y + hgrafica); // Esquina inf. der.
    let stepx = wgrafica / xtitles.len() as f64;
    let stepy = hgrafica / (max - min).abs();
    let ticksize = wgrafica / 100.0;
    // Escalas lineales de X e Y sobre la gráfica
    let scalex = linear_scale(0.0, xtitles.len() as f64, og_x, eg_x);
    let scaley = linear_scale(min, max, eg_y, og_y);
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

    // Etiquetas de componentes
    cr.set_line_width(1.0);
    cr.set_font_size(small_size);

    let layout = widget.create_pango_layout(None);
    let fontdesc = pango::FontDescription::from_string(&format!("Arial Normal {}", small_size));
    layout.set_font_description(Some(&fontdesc));
    layout.set_alignment(pango::Alignment::Center);
    layout.set_width(pango::units_from_double((stepx * 0.9).round()));

    for (i, label) in xtitles.iter().enumerate() {
        layout.set_text(label);
        let xpos = og_x + (i as f64 + 0.05) * stepx;
        let ypos = eg_y + ticksize * 2.0;
        cr.move_to(xpos, ypos);
        pangocairo::show_layout(cr, &layout);
    }

    // Ticks en x
    cr.set_line_width(1.0);
    cr.set_source_rgb(0.0, 0.0, 0.0);
    for i in 0..=xtitles.len() {
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
        let txt = format!("{:.0}", max - i as f64 * 10.0);
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

    // Barras de las series
    for (i_serie, (vals, color)) in series.iter().enumerate(){
        let i_serie = i_serie as f64;
        for (i_concepto, val) in vals.iter().enumerate() {
            // barra
            cr.set_source_rgb(color.0, color.1, color.2);
            cr.new_path();
            let x = scalex(i_concepto as f64 + i_serie / numseries);
            let y = scaley(*val as f64);
            let height = y - y0;
            cr.rectangle(x, y, stepx / numseries, -height);
            cr.fill_preserve();
            // etiquetas
            cr.set_source_rgb(0.0, 0.0, 0.0);
            cr.stroke();
            if val.abs() >= f32::EPSILON {
                let txt = format!("{:.1}", val);
                let txt_ext = cr.text_extents(&txt);
                let x_txt =
                    scalex(i_concepto as f64 + (0.5 + i_serie) / numseries) - txt_ext.width / 2.0;
                let y_txt = if *val < 0.0 {
                    y + txt_ext.height * 1.5
                } else {
                    y - txt_ext.height * 0.5
                };
                cr.move_to(x_txt, y_txt);
                cr.show_text(&txt);
            }
        }
    }

    // Restauramos contexto
    cr.restore();
}
