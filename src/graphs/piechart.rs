//! Gráfica de tarta de las ganancias o pérdidas de energía en temporadas de calefacción o refrigeración
//!
//! Las pérdidas o ganancias se definen mediante las 8 categorías de HULC más el total

use std::f64::consts::PI;

use gtk::WidgetExt;
use itertools::izip;

use crate::parsers::types::FlujosVec;

const COOLING_COLORS: [(f64, f64, f64); 8] = [
    (0.0, 1.0, 1.0),
    (0.0, 0.875, 1.0),
    (0.0, 0.75, 1.0),
    (0.0, 0.625, 1.0),
    (0.0, 0.5, 1.0),
    (0.0, 0.375, 1.0),
    (0.0, 0.25, 1.0),
    (0.0, 0.125, 1.0),
];

const HEATING_COLORS: [(f64, f64, f64); 8] = [
    (1.0, 1.0, 0.0),
    (1.0, 0.875, 0.0),
    (1.0, 0.75, 0.0),
    (1.0, 0.625, 0.0),
    (1.0, 0.5, 0.0),
    (1.0, 0.375, 0.0),
    (1.0, 0.25, 0.0),
    (1.0, 0.125, 0.0),
];

const PIE_LABELS: [&str; 8] = [
    "Muros",
    "Cubiertas",
    "Suelos",
    "PTs",
    "Solar huecos",
    "Transmisión huecos",
    "Fuentes internas",
    "Ventilación e infiltraciones",
];

/// Modo de visualización del gráfico de tarta
pub enum PieMode {
    /// Ganancias de la temporada de calefacción
    CalPos,
    /// Pérdidas de la temporada de calefacción
    CalNeg,
    /// Ganancias de la temporada de refrigeración
    RefPos,
    /// Pérdidas de la temporada de refrigeración
    RefNeg,
}

/// Datos de cada valor
struct Point {
    /// Etiqueta
    label: String,
    /// Valor
    value: f64,
    /// Porcentaje, como cadena ("13.1%")
    value_pct: String,
    /// Ángulo inicial (radianes) 0 en eje X, pi/2 en eje Y.
    start_angle: f64,
    /// Ángluo final (radianes)
    end_angle: f64,
    /// Ángulo medio (radianes)
    mid_angle: f64,
    /// ¿El ángulo medio apunta al eje X positivo?
    is_right: bool,
}

/// Genera datos para la representación de la gráfica
///
/// Devuelve lista ordenada de elementos Point {label, value, value_pct, start_angle, end_angle}
fn build_data(demandas: &[f64]) -> Vec<Point> {
    let demanda_total: f64 = demandas.iter().map(|v: &f64| v.abs()).sum();
    let demandas_pct = demandas.iter().map(|demanda| {
        if demanda_total != 0.0 {
            format!("{:.1}%", 100.0 * demanda.abs() / demanda_total)
        } else {
            "-".to_string()
        }
    });

    // Datos para representar
    let mut data: Vec<Point> = izip!(&PIE_LABELS, demandas.iter().cloned(), demandas_pct)
        .map(|(label, value, ref value_pct)| Point {
            label: label.to_string(),
            value,
            value_pct: value_pct.to_string(),
            start_angle: 0.0,
            end_angle: 0.0,
            mid_angle: 0.0,
            is_right: true,
        })
        .collect();
    data.sort_by(|a, b| a.value.abs().partial_cmp(&b.value.abs()).unwrap());
    let angles: Vec<(f64, f64)> = data
        .iter()
        .map(|d| 2.0 * PI * d.value / demanda_total)
        .fold(vec![0.0], |mut acc, angle| {
            let last = acc.last().copied().unwrap();
            acc.push(last + angle);
            acc
        })
        .windows(2)
        .map(|win| (win[0], win[1]))
        .collect();
    data.iter_mut()
        .zip(angles)
        .for_each(|(d, (start_angle, end_angle))| {
            d.start_angle = start_angle;
            d.end_angle = end_angle;
            d.mid_angle = 0.5 * (start_angle + end_angle);
            d.is_right = (d.mid_angle < PI / 2.0) || (d.mid_angle > 3.0 * PI / 2.0);
        });
    data
}

/// Dibuja gráfica de tarta para CalPos, CalNeg, RefPos y RefNeg
pub fn draw_piechart(
    widget: &gtk::DrawingArea,
    cr: &cairo::Context,
    flujos: &FlujosVec,
    mode: PieMode,
) {
    let (title, colores, demandas) = match mode {
        PieMode::CalPos => (
            "Ganancias térmicas, periodo de calefacción",
            HEATING_COLORS,
            &flujos.calpos,
        ),
        PieMode::CalNeg => (
            "Pérdidas térmicas, periodo de calefacción",
            HEATING_COLORS,
            &flujos.calneg,
        ),
        PieMode::RefPos => (
            "Ganancias térmicas, periodo de refrigeración",
            COOLING_COLORS,
            &flujos.refpos,
        ),
        PieMode::RefNeg => (
            "Pérdidas térmicas, periodo de refrigeración",
            COOLING_COLORS,
            &flujos.refneg,
        ),
    };

    // Si los datos tienen 9 valores es que incluyen al final el total... y lo eliminamos
    let len = demandas.len();
    let demandas = if len == 8 {
        &demandas
    } else {
        &demandas[..len - 1]
    };

    let demandas = demandas.iter().map(|v| v.abs() as f64).collect::<Vec<_>>();
    let mut data = build_data(&demandas);
    let demanda_total: f64 = demandas.iter().map(|v| v.abs()).sum();

    // Posiciones
    let rect = widget.get_allocation();
    let width = rect.width as f64;
    let height = rect.height as f64;
    let htitulo = 0.1 * height;
    let hgrafica = 0.9 * height;
    let wgrafica = 1.0 * width;
    let mut fontsize = 14.0;
    let radius = 0.8 * (wgrafica).min(hgrafica) / 2.0;

    // Inciamos dibujo guardando contexto
    cr.save();

    // Calculamos tamaño de fuente para no salirnos del borde de la imagen
    cr.select_font_face("Arial", cairo::FontSlant::Normal, cairo::FontWeight::Normal);
    cr.set_font_size(fontsize);
    let textmargin = 5.0; // separación de flecha y texto
    let textlen = cr
        .text_extents(PIE_LABELS.iter().max_by_key(|x| x.len()).unwrap_or(&"-"))
        .width
        + 2.0 * textmargin;
    let textmaxwidth = 0.5 * wgrafica - 1.1 * radius;
    fontsize = fontsize.min(fontsize * textmaxwidth / textlen);

    // Posición del centro, dejando espacio para el título y dos líneas de texto
    let (ox, oy) = (0.5 * wgrafica, htitulo + 0.5 * hgrafica - 2.0 * fontsize);

    // Fondo
    cr.set_source_rgb(1.0, 1.0, 1.0);
    cr.new_path();
    cr.rectangle(0.0, 0.0, width, height);
    cr.fill();

    // Título
    cr.select_font_face("Arial", cairo::FontSlant::Normal, cairo::FontWeight::Bold);
    cr.set_font_size(20.0);
    cr.set_source_rgb(0.5, 0.5, 0.5);
    let extents = cr.text_extents(title);
    cr.move_to(ox - extents.width / 2.0, 0.5 * (htitulo + extents.height));
    cr.show_text(title);

    // Caso con demanda total nula
    if demanda_total < 0.01 {
        cr.set_source_rgb(0.5, 0.5, 0.5);
        cr.set_line_width(0.5);
        cr.set_font_size(14.0);
        cr.move_to(ox + radius, oy);
        cr.arc(ox, oy, radius, 0.0, 2.0 * PI);
        cr.stroke();
        let txt = "Sin datos de demanda o demanda casi nula";
        let extents = cr.text_extents(txt);
        cr.move_to(ox - extents.width / 2.0, oy - extents.height / 2.0);
        cr.show_text(txt);
        cr.restore();
        return;
    }

    // Cuñas del círculo y radios
    for (point, (r, g, b)) in data.iter().zip(colores.iter()) {
        // Cuñas
        cr.set_source_rgb(*r, *g, *b);
        cr.move_to(ox, oy);
        cr.line_to(
            ox + radius * point.start_angle.cos(),
            oy + radius * point.start_angle.sin(),
        );
        cr.arc(ox, oy, radius, point.start_angle, point.end_angle);
        cr.close_path();
        cr.stroke_preserve();
        cr.fill();
        // Radios
        cr.set_source_rgb(1.0, 1.0, 1.0);
        cr.set_line_width(0.5);
        cr.move_to(ox, oy);
        cr.line_to(
            ox + radius * point.start_angle.cos(),
            oy + radius * point.start_angle.sin(),
        );
        cr.stroke();
    }

    // Leyendas
    // Reordenamos las cuñas de arriba abajo (eje Y positivo hacia abajo) para colocar etiquetas
    data.sort_by(|a, b| {
        (a.mid_angle.sin())
            .partial_cmp(&(b.mid_angle.sin()))
            .unwrap()
    });

    // Omite puntos con menos del 0.01%
    let skip_point = |p: &Point| (p.end_angle - p.start_angle) < 0.01 * 2.0 * PI / 100.0;

    // Posiciones del texto a cada lado, descontando los % < 0.01%
    let txt_width = textmaxwidth - 40.0; // ancho disponible y margen de 20px por cada lado
    let numlabels_right: i32 = data.iter().filter(|p| p.is_right && !skip_point(p)).count() as i32;
    let numlabels_left: i32 = data
        .iter()
        .filter(|p| !p.is_right && !skip_point(p))
        .count() as i32;

    let layout = widget.create_pango_layout(Some("Prueba"));
    let fontdesc = pango::FontDescription::from_string("Arial Normal 10.5");
    layout.set_font_description(Some(&fontdesc));
    layout.set_width(pango::units_from_double(txt_width.round()));

    let (_, line_height) = layout.get_pixel_size();
    let expected_height_left = ((numlabels_left * 3 - 1) * line_height) as f64;
    let expected_height_right = ((numlabels_right * 3 - 1) * line_height) as f64;

    let txt_xpos_right = wgrafica - txt_width;
    let txt_xpos_left = 20.0;
    let mut txt_ypos_left = oy - expected_height_left / 2.0;
    let mut txt_ypos_right = oy - expected_height_right / 2.0;

    cr.set_source_rgb(0.0, 0.0, 0.0);
    cr.set_line_width(0.5);
    cr.select_font_face("Arial", cairo::FontSlant::Normal, cairo::FontWeight::Normal);
    cr.set_font_size(fontsize);
    for point in &data {
        let Point {
            label,
            value,
            value_pct,
            start_angle,
            end_angle,
            mid_angle,
            is_right,
        } = point;

        // Omite valores < 0.01 %
        if skip_point(point) {
            continue;
        };

        // Porcentajes, solo si hay hueco
        let extents = cr.text_extents(&value_pct);
        let available_height = (2.0 / 3.0 * radius * (end_angle - start_angle).sin()).abs();
        if available_height > 1.1 * extents.height {
            cr.move_to(
                ox + 2.0 / 3.0 * radius * mid_angle.cos() - 0.5 * extents.width,
                oy + 2.0 / 3.0 * radius * mid_angle.sin() + 0.5 * extents.height,
            );
            cr.show_text(&value_pct);
        }

        // Líneas
        let x_start_lead = ox + 1.02 * radius * mid_angle.cos();
        let y_start_lead = oy + 1.02 * radius * mid_angle.sin();
        cr.set_source_rgb(0.5, 0.5, 0.5);
        cr.move_to(x_start_lead, y_start_lead);
        if *is_right {
            cr.line_to(ox + radius + 10.0, y_start_lead);
            cr.line_to(txt_xpos_right - 10.0, txt_ypos_right + line_height as f64);
        } else {
            cr.line_to(ox - radius - 10.0, y_start_lead);
            cr.line_to(textmaxwidth - 10.0, txt_ypos_left + line_height as f64);
        };
        cr.stroke();

        // Textos
        cr.set_source_rgb(0.0, 0.0, 0.0);
        layout.set_text(&format!("{}\n{:.2} kWh/m2·a ({})", label, value, value_pct));
        if *is_right {
            cr.move_to(txt_xpos_right, txt_ypos_right);
            txt_ypos_right += line_height as f64 * 3.0;
        } else {
            cr.move_to(txt_xpos_left, txt_ypos_left);
            txt_ypos_left += line_height as f64 * 3.0;
        };
        pangocairo::show_layout(cr, &layout);
    }

    // Restauramos contexto
    cr.restore();
}
