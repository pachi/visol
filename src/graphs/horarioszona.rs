//! Histograma de demanda mensual para una zona o el edificio
//!
//! Las pérdidas o ganancias se definen mediante las 8 categorías de HULC más el total

use gtk::WidgetExt;

use super::{NORMAL_SIZE, TITLE_SIZE, draw_watermark, linear_scale};
use crate::parsers::bin::ZonaLider;

// XXX: Ver ejemplos en https://medium.com/journey-to-rust/drawing-in-gtk-in-rust-part-1-4a401eecc4e0
// https://github.com/GuillaumeGomez/process-viewer
// https://docs.rs/plotters/0.3.0/plotters/
// https://plotters-rs.github.io/book/basic/basic_data_plotting.html

/// Dibuja gráfica con los datos horarios de zona
pub fn draw_zonasgraph(
    widget: &gtk::DrawingArea,
    cr: &cairo::Context,
    zonedata: Option<&ZonaLider>,
) {
    let title = "Valores diarios de zona";

    // Posiciones
    let rect = widget.get_allocation();
    let width = rect.width as f64;
    let height = rect.height as f64;
    let htitulo = 0.1 * height;
    let hsubtitulo = 0.05 * height;
    let margin = 0.05 * height;
    let hgrafica = 0.9 * height - 2.0 * margin;
    let wgrafica = width - 4.0 * margin;

    cr.save();

    // Fondo
    cr.set_source_rgb(1.0, 1.0, 1.0);
    cr.rectangle(1.0, 1.0, width, height);
    cr.fill();

    // Título
    cr.select_font_face("Arial", cairo::FontSlant::Normal, cairo::FontWeight::Bold);
    cr.set_font_size(TITLE_SIZE);
    cr.set_source_rgb(0.5, 0.5, 0.5);
    let extents = cr.text_extents(title);
    cr.move_to(
        (width - extents.width) / 2.0,
        0.5 * (htitulo + extents.height),
    );
    cr.show_text(title);

    if zonedata.is_none() {
        // En modos que no son de Zona dibujamos una nota
        let txt = "Seleccione una zona";
        let te = cr.text_extents(txt);
        cr.move_to((width - te.width) / 2.0, height * 0.5);
        cr.show_text(txt);
    } else {
        // En modo Zona dibujamos los valores horarios
        let data = zonedata.unwrap();

        let g_x0 = 2.0 * margin;
        let g_x1 = 2.0 * margin + wgrafica;
        let g_width = g_x1 - g_x0;
        let gheight = (hgrafica - margin) / 3.0 - hsubtitulo;

        cr.set_font_size(NORMAL_SIZE);
        cr.set_line_width(0.5);

        // Grafica 1 - Temperatura diaria (máxima, media, mínima)
        let g1_text = "Temperatura diaria (máxima, media, mínima)";
        let g1_y0 = htitulo + hsubtitulo;
        let g1_y1 = g1_y0 + gheight;

        // Subtítulo y marco
        cr.set_source_rgb(0.5, 0.5, 0.5);
        let ext = cr.text_extents(&g1_text);
        cr.move_to((width - ext.width) / 2.0, g1_y0 - hsubtitulo / 2.0);
        cr.show_text(g1_text);
        cr.rectangle(g_x0, g1_y0, g_width, gheight);
        cr.stroke();
        // Valores remuestreados con media, máxima y mínima diaria
        let resampled_temp = data.t_real.chunks_exact(24);
        let t_mean: Vec<_> = resampled_temp
            .clone()
            .map(|chunk| chunk.iter().sum::<f32>() / 24.0).collect();
        let t_min: Vec<_> = resampled_temp
            .clone()
            .map(|chunk| chunk.iter().fold(f32::INFINITY, |a, b| a.min(*b))).collect();
        let t_max: Vec<_> = resampled_temp
            .clone()
            .map(|chunk| chunk.iter().fold(f32::NEG_INFINITY, |a, b| a.max(*b))).collect();
        // Dominio de los datos de entrada
        let min_lim = t_min.iter().fold(f32::INFINITY, |a, b| a.min(*b)).ceil() - 3 as f32;
        let max_lim = t_max
            .iter()
            .fold(f32::NEG_INFINITY, |a, b| a.max(*b))
            .floor()
            + 3 as f32;
        
        let xscale = linear_scale(0 as f64, 365 as f64, g_x0, g_x0 + g_width);
        let yscale = linear_scale(min_lim as f64, max_lim as f64, g1_y1, g1_y0);

        cr.set_line_width(2.0);
        cr.move_to(xscale(0.0), yscale(t_mean[0] as f64));
        t_mean.iter().enumerate().skip(1).for_each(|(i, t)| cr.line_to(xscale(i as f64), yscale(*t as f64)));
        cr.stroke();
        
        cr.set_line_width(0.5);
        cr.set_source_rgb(0.0, 0.0, 1.0);
        cr.move_to(xscale(0.0), yscale(t_min[0] as f64));
        t_min.iter().enumerate().skip(1).for_each(|(i, t)| cr.line_to(xscale(i as f64), yscale(*t as f64)));
        cr.stroke();

        cr.set_line_width(0.5);
        cr.set_source_rgb(1.0, 0.0, 0.0);
        cr.move_to(xscale(0.0), yscale(t_max[0] as f64));
        t_max.iter().enumerate().skip(1).for_each(|(i, t)| cr.line_to(xscale(i as f64), yscale(*t as f64)));
        cr.stroke();

        // TODO: hacer rellenos entre líneas: haciendo ida y vuelta con t_mean y t_max.reversed() y cerrando...

        // Gráfica 2 - Carga térmica diaria (sensible, latente, total)
        let g2_text = "Carga térmica diaria (sensible, latente, total)";
        let g2_y0 = g1_y1 + margin + hsubtitulo;
        let g2_y1 = g2_y0 + gheight;

        // Subtítulo y marco
        cr.set_source_rgb(0.5, 0.5, 0.5);
        let ext = cr.text_extents(&g1_text);
        cr.move_to((width - ext.width) / 2.0, g2_y0 - hsubtitulo / 2.0);
        cr.show_text(g2_text);
        cr.rectangle(g_x0, g2_y0, g_width, gheight);
        cr.stroke();

        // Línea de 0
        // cr.set_line_width(3.0);
        // cr.move_to(xscale(0.0), yscale(0.0));
        // cr.rel_line_to(g_width, 0.0);
        // cr.stroke();



        // Gráfica 3 - Caudal diario de ventilación e infiltraciones
        let g3_text = "Caudal diario de ventilación e infiltraciones";
        let g3_y0 = g2_y1 + margin + hsubtitulo;
        let g3_y1 = g3_y0 + gheight;

        let ext = cr.text_extents(&g1_text);
        cr.move_to((width - ext.width) / 2.0, g3_y0 - hsubtitulo / 2.0);
        cr.show_text(g3_text);
        cr.rectangle(g_x0, g3_y0, g_width, gheight);
        cr.stroke();
    }

    draw_watermark(&cr, width - height * 0.05, htitulo);

    cr.restore();
}
