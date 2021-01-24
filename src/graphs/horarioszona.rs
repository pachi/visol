//! Histograma de demanda mensual para una zona o el edificio
//!
//! Las pérdidas o ganancias se definen mediante las 8 categorías de HULC más el total

use gtk::WidgetExt;

use super::{draw_watermark, linear_scale, rounder, MESES, NORMAL_SIZE, SMALL_SIZE, TITLE_SIZE};
use crate::parsers::bin::ZonaLider;

/// Dibuja gráfica con los datos horarios de zona
pub fn draw_zonasgraph(
    widget: &gtk::DrawingArea,
    cr: &cairo::Context,
    zonedata: Option<&ZonaLider>,
) {
    let title = "Valores diarios de zona";

    // Posiciones y cálculos previos
    let rect = widget.get_allocation();
    let widget_width = rect.width as f64;
    let widget_height = rect.height as f64;
    let htitle = 0.1 * widget_height;
    let subtitle_block_height = 0.05 * widget_height;
    let margin = 0.05 * widget_height;

    let x0 = 2.0 * margin;
    let x1 = widget_width - x0;
    let width = x1 - x0;
    let height = (0.9 * widget_height - 3.0 * margin) / 3.0 - subtitle_block_height;
    let ticksize = width / 10.0 / 12.0;
    let xscale = linear_scale(0.0, 365.0, x0, x1);

    cr.save();

    // Fondo
    cr.set_source_rgb(1.0, 1.0, 1.0);
    cr.rectangle(1.0, 1.0, widget_width, widget_height);
    cr.fill();

    // Título
    cr.select_font_face("Arial", cairo::FontSlant::Normal, cairo::FontWeight::Bold);
    cr.set_font_size(TITLE_SIZE);
    cr.set_source_rgb(0.5, 0.5, 0.5);
    let extents = cr.text_extents(title);
    cr.move_to(
        (widget_width - extents.width) / 2.0,
        0.5 * (htitle + extents.height),
    );
    cr.show_text(title);

    // En modos que no son de Zona dibujamos una nota
    if zonedata.is_none() {
        let txt = "Seleccione una zona";
        let te = cr.text_extents(txt);
        cr.move_to((widget_width - te.width) / 2.0, widget_height * 0.5);
        cr.show_text(txt);
        return;
    }

    // En modo Zona dibujamos los valores horarios
    let data = zonedata.unwrap();

    // ## Grafica 1 - Temperatura diaria (máxima, media, mínima)
    let y0 = htitle + subtitle_block_height;
    let y1 = y0 + height;
    let subtitle = "Temperatura diaria (máxima, media, mínima) [ºC]";
    draw_subtitle_and_box(cr, subtitle, subtitle_block_height, x0, y0, width, height);
    draw_months(&cr, x0, x1, y0, y1);

    // Valores remuestreados con media, máxima y mínima diaria
    let resampled_temp = data.t_real.chunks_exact(24);
    let t_mean: Vec<_> = resampled_temp
        .clone()
        .map(|chunk| chunk.iter().sum::<f32>() / 24.0)
        .collect();
    let t_min: Vec<_> = resampled_temp
        .clone()
        .map(|chunk| chunk.iter().fold(f32::INFINITY, |a, b| a.min(*b)))
        .collect();
    let t_max: Vec<_> = resampled_temp
        .clone()
        .map(|chunk| chunk.iter().fold(f32::NEG_INFINITY, |a, b| a.max(*b)))
        .collect();
    // Dominio de los datos de entrada
    let min_lim = t_min.iter().fold(f32::INFINITY, |a, b| a.min(*b)).ceil() - 3 as f32;
    let max_lim = t_max
        .iter()
        .fold(f32::NEG_INFINITY, |a, b| a.max(*b))
        .floor()
        + 3 as f32;
    let yscale = linear_scale(min_lim as f64, max_lim as f64, y1, y0);

    // Fondo T 17-28ºC
    cr.move_to(x0, yscale(28.0));
    cr.rectangle(x0, yscale(28.0), width, yscale(17.0) - yscale(28.0));
    cr.set_source_rgba(0.5, 0.5, 0.5, 0.15);
    cr.fill();
    // Fondo T 20-26ºC
    cr.move_to(x0, yscale(26.0));
    cr.rectangle(x0, yscale(26.0), width, yscale(20.0) - yscale(26.0));
    cr.set_source_rgba(0.5, 0.5, 0.5, 0.25);
    cr.fill();

    // Etiquetas Y
    let labels: Vec<(f64, String)> = [17.0, 20.0, 26.0, 28.0]
        .iter()
        .map(|v| (yscale(*v), format!("{:.0}", v)))
        .collect();
    ylabels(cr, labels.as_slice(), ticksize, x0, true);

    // Relleno de t_media con t_maxima (ir con t_media y volver con t_maxima)
    cr.set_source_rgba(1.0, 0.5, 0.5, 0.5);
    cr.move_to(x0, yscale(t_mean[0] as f64));
    t_mean
        .iter()
        .enumerate()
        .skip(1)
        .for_each(|(i, t)| cr.line_to(xscale(i as f64), yscale(*t as f64)));
    t_max
        .iter()
        .enumerate()
        .rev()
        .for_each(|(i, t)| cr.line_to(xscale(i as f64), yscale(*t as f64)));
    cr.fill();
    // Relleno de t_media con t_mínima (ir con t_media y volver con t_mínima)
    cr.set_source_rgba(0.5, 0.5, 1.0, 0.5);
    cr.move_to(x0, yscale(t_mean[0] as f64));
    t_mean
        .iter()
        .enumerate()
        .skip(1)
        .for_each(|(i, t)| cr.line_to(xscale(i as f64), yscale(*t as f64)));
    t_min
        .iter()
        .enumerate()
        .rev()
        .for_each(|(i, t)| cr.line_to(xscale(i as f64), yscale(*t as f64)));
    cr.fill();

    // Línea de t_mínima
    cr.set_line_width(0.5);
    cr.set_source_rgb(0.0, 0.0, 1.0);
    cr.move_to(x0, yscale(t_min[0] as f64));
    t_min
        .iter()
        .enumerate()
        .skip(1)
        .for_each(|(i, t)| cr.line_to(xscale(i as f64), yscale(*t as f64)));
    cr.stroke();
    // Línea de t_máxima
    cr.set_line_width(0.5);
    cr.set_source_rgb(1.0, 0.0, 0.0);
    cr.move_to(x0, yscale(t_max[0] as f64));
    t_max
        .iter()
        .enumerate()
        .skip(1)
        .for_each(|(i, t)| cr.line_to(xscale(i as f64), yscale(*t as f64)));
    cr.stroke();
    // Línea de t_media
    cr.set_line_width(1.0);
    cr.set_source_rgb(0.0, 0.0, 0.0);
    cr.move_to(x0, yscale(t_mean[0] as f64));
    t_mean
        .iter()
        .enumerate()
        .skip(1)
        .for_each(|(i, t)| cr.line_to(xscale(i as f64), yscale(*t as f64)));
    cr.stroke();

    // ## Gráfica 2 - Carga térmica diaria media (sensible, total (sen + lat)) W
    let y0 = y1 + margin + subtitle_block_height;
    let y1 = y0 + height;
    let subtitle = "Carga térmica diaria (sensible, total) [W]";
    draw_subtitle_and_box(cr, subtitle, subtitle_block_height, x0, y0, width, height);
    draw_months(&cr, x0, x1, y0, y1);

    let q_sen: Vec<_> = data
        .q_sen
        .chunks_exact(24)
        .clone()
        .map(|chunk| chunk.iter().sum::<f32>() / 24.0)
        .collect();

    let q_lat: Vec<_> = data
        .q_lat
        .chunks_exact(24)
        .clone()
        .map(|chunk| chunk.iter().sum::<f32>() / 24.0)
        .collect();

    let q_tot: Vec<f32> = q_sen.iter().zip(q_lat.iter()).map(|(a, b)| a + b).collect();
    let q_min = q_tot.iter().fold(f32::INFINITY, |a, b| a.min(*b));
    let q_max = q_tot.iter().fold(f32::NEG_INFINITY, |a, b| a.max(*b));
    let minmax_margin = (q_max - q_min).abs() * 0.1;

    let yscale = linear_scale(
        (q_min - minmax_margin).floor() as f64,
        (q_max + minmax_margin).ceil() as f64,
        y1,
        y0,
    );

    // Etiquetas Y
    let labels: Vec<(f64, String)> = [q_min as f64, 0.0, q_max as f64]
        .iter()
        .map(|v| (yscale(*v), format!("{:.1}", v)))
        .collect();
    ylabels(cr, labels.as_slice(), ticksize, x0, true);

    // Relleno de q_sen con 0
    cr.set_source_rgba(1.0, 0.5, 0.5, 0.5);
    cr.move_to(x0, rounder(yscale(0.0)));
    q_sen
        .iter()
        .enumerate()
        .for_each(|(i, v)| cr.line_to(xscale(i as f64), yscale(*v as f64)));
    cr.line_to(x1, rounder(yscale(0.0)));
    cr.fill();

    // Línea de q_sen
    cr.set_line_width(0.5);
    cr.set_source_rgb(1.0, 0.0, 0.0);
    cr.move_to(x0, yscale(t_mean[0] as f64));
    q_sen
        .iter()
        .enumerate()
        .skip(1)
        .for_each(|(i, t)| cr.line_to(xscale(i as f64), yscale(*t as f64)));
    cr.stroke();
    // Línea de q_tot
    cr.set_line_width(1.0);
    cr.set_source_rgb(0.0, 0.0, 0.0);
    cr.move_to(x0, yscale(t_min[0] as f64));
    q_tot
        .iter()
        .enumerate()
        .skip(1)
        .for_each(|(i, t)| cr.line_to(xscale(i as f64), yscale(*t as f64)));
    cr.stroke();

    // Línea de 0 W
    cr.set_line_width(0.5);
    cr.set_source_rgb(0.5, 0.5, 0.5);
    cr.move_to(x0, rounder(yscale(0.0)));
    cr.line_to(x1, rounder(yscale(0.0)));
    cr.stroke();

    // Gráfica 3 - Caudal diario de ventilación e infiltraciones
    let y0 = y1 + margin + subtitle_block_height;
    let y1 = y0 + height;
    let subtitle = "Caudal diario de ventilación e infiltraciones [m³/h; 1/h]";
    draw_subtitle_and_box(cr, subtitle, subtitle_block_height, x0, y0, width, height);
    draw_months(&cr, x0, x1, y0, y1);

    let volumen = data.volumen; // m3

    // Convertir kg/s a m3/h: caudal[m3/h] = caudal[kg/s] * 3600 s/h * 1.225 kg/m³
    let v_tot: Vec<_> = data
        .v_ventinf
        .chunks_exact(24)
        .clone()
        .map(|chunk| chunk.iter().sum::<f32>() / 24.0 * 3600.0 / 1.225)
        .collect();

    let v_min = v_tot.iter().fold(f32::INFINITY, |a, b| a.min(*b));
    let v_max = v_tot.iter().fold(f32::NEG_INFINITY, |a, b| a.max(*b));
    let minmax_margin = (v_max.max(1.0) - v_min.min(0.0)).abs() * 0.1;

    let yscale = linear_scale(
        (v_min.min(0.0) - minmax_margin).floor() as f64,
        (v_max.max(1.0) + minmax_margin).ceil() as f64,
        y1,
        y0,
    );

    // Etiquetas Y
    // m3/h
    let labels: Vec<(f64, String)> = [0.0, v_min as f64, v_max as f64]
        .iter()
        .map(|v| (yscale(*v), format!("{:.1}", v)))
        .collect();
    ylabels(cr, labels.as_slice(), ticksize, x0, true);
    // 1/h
    let labels: Vec<(f64, String)> = [0.0, v_min as f64, v_max as f64]
        .iter()
        .map(|v| (yscale(*v), format!("{:.1}", v / volumen as f64)))
        .collect();
    ylabels(cr, labels.as_slice(), ticksize, x1, false);

    // Relleno de v_tot con 0
    cr.set_source_rgba(0.5, 0.5, 1.0, 0.5);
    cr.move_to(x0, rounder(yscale(0.0)));
    v_tot
        .iter()
        .enumerate()
        .for_each(|(i, v)| cr.line_to(xscale(i as f64), yscale(*v as f64)));
    cr.line_to(x1, rounder(yscale(0.0)));
    cr.fill();

    // Línea de v_tot
    cr.set_line_width(1.0);
    cr.set_source_rgb(0.0, 0.0, 0.0);
    cr.move_to(x0, yscale(v_tot[0] as f64));
    v_tot
        .iter()
        .enumerate()
        .skip(1)
        .for_each(|(i, t)| cr.line_to(xscale(i as f64), yscale(*t as f64)));
    cr.stroke();
    // Línea de v = 0
    cr.set_line_width(0.5);
    cr.set_source_rgb(0.5, 0.5, 0.5);
    cr.move_to(x0, rounder(yscale(0.0)));
    cr.line_to(x1, rounder(yscale(0.0)));
    cr.stroke();

    draw_watermark(&cr, widget_width - widget_height * 0.05, htitle);

    cr.restore();
}

/// Líneas de separación de meses, etiquetas y ticks
/// (x0, y0), (x1, y1) son las coordenadas de la esquina sup. izq. e inf. derecha.
fn draw_months(cr: &cairo::Context, x0: f64, x1: f64, y0: f64, y1: f64) {
    let xstep = (x1 - x0) / 12.0;
    let ticksize: f64 = xstep / 10.0;
    cr.save();
    cr.select_font_face("Arial", cairo::FontSlant::Normal, cairo::FontWeight::Normal);
    cr.set_font_size(SMALL_SIZE);
    cr.set_line_width(0.5);
    cr.set_source_rgb(0.5, 0.5, 0.5);
    // Rótulos y ticks
    let ext = cr.text_extents("Ene");
    let th = ext.height;
    for (i, mes) in MESES.iter().enumerate() {
        cr.move_to(rounder(x0 + i as f64 * xstep), y1);
        cr.rel_line_to(0.0, ticksize);
        cr.stroke_preserve();
        cr.rel_move_to(0.0, ticksize + th);
        cr.show_text(mes);
    }
    cr.move_to(rounder(x1), y1);
    cr.rel_line_to(0.0, ticksize);
    cr.stroke();
    // Líneas de mes
    for i in 1..MESES.len() {
        cr.move_to(rounder(x0 + i as f64 * xstep), y0);
        cr.rel_line_to(0.0, y1 - y0);
        cr.stroke();
    }
    cr.restore();
}

/// Subtítulo y marco
///
/// subtitle_height es la altura que tiene el encabezado, por encima del recuadro
/// (x0, y0) es la esquina sup. izq de la gráfica
/// width, height es el ancho, alto del recuadro
fn draw_subtitle_and_box(
    cr: &cairo::Context,
    subtitle: &str,
    subtitle_height: f64,
    x0: f64,
    y0: f64,
    width: f64,
    height: f64,
) {
    cr.save();
    cr.set_font_size(NORMAL_SIZE);
    cr.set_line_width(0.5);
    cr.set_source_rgb(0.5, 0.5, 0.5);
    let ext = cr.text_extents(subtitle);
    cr.move_to(
        rounder(x0 + (width - ext.width) / 2.0),
        rounder(y0 - subtitle_height / 2.0),
    );
    cr.show_text(subtitle);
    cr.rectangle(x0, y0, width, height);
    cr.stroke();
    cr.restore();
}

/// Etiquetas eje Y
/// values: vector de (coord_y, etiqueta)
/// ticksize: tamaño del tick
/// x0: coordenada x del eje Y
/// left_axis indica si es un eje a la izquierda o a la derecha de la gráfica
fn ylabels(cr: &cairo::Context, values: &[(f64, String)], ticksize: f64, x0: f64, left_axis: bool) {
    cr.save();
    cr.select_font_face("Arial", cairo::FontSlant::Normal, cairo::FontWeight::Normal);
    cr.set_font_size(SMALL_SIZE);
    cr.set_line_width(0.5);
    cr.set_source_rgb(0.5, 0.5, 0.5);
    for (yval, label) in values {
        cr.move_to(x0, *yval);
        if left_axis {
            cr.rel_line_to(-ticksize, 0.0);
        } else {
            cr.rel_line_to(ticksize, 0.0);
        }
        cr.stroke_preserve();
        if left_axis {
            let ext = cr.text_extents(label);
            cr.rel_move_to(-(ticksize + ext.width), 0.0);
        } else {
            cr.rel_move_to(ticksize, 0.0);
        }
        cr.show_text(&label);
    }
    cr.restore();
}
