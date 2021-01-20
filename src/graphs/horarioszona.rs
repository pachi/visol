//! Histograma de demanda mensual para una zona o el edificio
//!
//! Las pérdidas o ganancias se definen mediante las 8 categorías de HULC más el total

use std::cell::RefCell;
use std::rc::Rc;

use gtk::WidgetExt;

use crate::appstate::{AppState, TipoObjeto};

use super::{TITLE_SIZE, draw_watermark};

// XXX: Ver ejemplos en https://medium.com/journey-to-rust/drawing-in-gtk-in-rust-part-1-4a401eecc4e0
// https://github.com/GuillaumeGomez/process-viewer
// https://docs.rs/plotters/0.3.0/plotters/
// https://plotters-rs.github.io/book/basic/basic_data_plotting.html

/// Dibuja gráfica con los datos horarios de zona
pub fn draw_zonasgraph(widget: &gtk::DrawingArea, cr: &cairo::Context, state: Rc<RefCell<AppState>>) {
    let state = state.borrow();
    let mode = state.curr_obj_type;

    let title = "Valores diarios de zona";

    // Posiciones
    let rect = widget.get_allocation();
    let width = rect.width as f64;
    let height = rect.height as f64;
    let htitulo = 0.1 * height;
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

    if mode != TipoObjeto::Zona {
        // En modos que no son de Zona dibujamos una nota
        let txt = "Seleccione una zona";
        cr.set_font_size(18.0);
        cr.set_source_rgb(0.0, 0.0, 0.0);
        // Centramos el texto
        let te = cr.text_extents(txt);
        cr.move_to((width - te.width) / 2.0, height * 0.5);
        cr.show_text(txt);
    } else {
        // TODO: En modo Zona dibujamos los valores horarios

    }

    draw_watermark(&cr, width - height * 0.05, htitulo);

    cr.restore();
}
