//! Histograma de demanda mensual para una zona o el edificio
//!
//! Las pérdidas o ganancias se definen mediante las 8 categorías de HULC más el total

use std::cell::RefCell;
use std::rc::Rc;

use gtk::WidgetExt;

use plotters::prelude::*;
use plotters_cairo::CairoBackend;

use crate::appstate::{AppState, TipoElemento};

// XXX: Ver ejemplos en https://medium.com/journey-to-rust/drawing-in-gtk-in-rust-part-1-4a401eecc4e0
// https://github.com/GuillaumeGomez/process-viewer
// https://docs.rs/plotters/0.3.0/plotters/
// https://plotters-rs.github.io/book/basic/basic_data_plotting.html

/// Dibuja gráfica con los datos horarios de zona
pub fn draw_zonasgraph(widget: &gtk::DrawingArea, cr: &cairo::Context, state: Rc<RefCell<AppState>>) {
    let state = state.borrow();
    let mode = state.curr_type;

    let rect = widget.get_allocation();
    let width = rect.width as f64;
    let height = rect.height as f64;
    // println!("Dibuja zonasgraph en ancho, alto: {}, {}", rect.width, rect.height);

    cr.save();
    if mode != TipoElemento::Zona {
        // En modos que no son de Zona dibujamos una nota
        let txt = "Seleccione una zona";

        cr.set_source_rgb(1.0, 1.0, 1.0);
        cr.rectangle(1.0, 1.0, width, height);
        cr.fill();
        cr.set_font_size(18.0);
        cr.set_source_rgb(0.0, 0.0, 0.0);
        // Centramos el texto
        let te = cr.text_extents(txt);
        cr.move_to((width - te.width) / 2.0, height * 0.5);
        cr.show_text(txt);
    } else {
        // TODO: En modo Zona dibujamos los valores horarios
        // Ver ejemplo de dos escalas en https://github.com/38/plotters/blob/master/examples/two-scales.rs
        // https://gtk-rs.org/docs/cairo/struct.Context.html#method.text_extents
        // cr.set_source_pixbuf(&pixbuf, 0f64, 0f64);
        // cr.paint();
        let root = CairoBackend::new(cr, (rect.width as u32, rect.height as u32))
            .unwrap()
            .into_drawing_area();
        root.fill(&WHITE).unwrap();
        //let root = root.margin(25, 25, 25, 25);

        let mut ctx = ChartBuilder::on(&root)
            .caption("This is a test", ("sans-serif", 20))
            .y_label_area_size(65)
            .x_label_area_size(40)
            .margin(25)
            .build_cartesian_2d(-10.0..100.0, -30.0..300.0)
            .unwrap();
        // .set_secondary_coord(0f32..10f32, -1.0f32..1.0f32);

        ctx.configure_mesh()
            .x_desc("X label")
            .y_desc("Y label")
            // .y_label_formatter(&|x| format!("{:e}", x))
            .axis_desc_style(("sans-serif", 15))
            .draw()
            .unwrap();

        // Eje secundario
        // ctx
        //     .configure_secondary_axes()
        //     .y_desc("Linear Scale")
        //     .draw()?;

        ctx.draw_series(LineSeries::new(
            (-10..=100).map(|x| {
                let x = x as f64;
                (x, 3.0 * x)
            }),
            &GREEN,
        ))
        .unwrap();

        // ctx
        // .draw_secondary_series(LineSeries::new(
        //     (0..=100).map(|x| (x as f32 / 10.0, (x as f32 / 5.0).sin())),
        //     &RED,
        // ))?
        // .label("y = sin(2x)")
        // .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &RED));

        // ctx
        //     .configure_series_labels()
        //     .background_style(&RGBColor(128, 128, 128))
        //     .draw()?;
    }
    cr.restore();
}
