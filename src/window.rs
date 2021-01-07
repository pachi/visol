///    Programa Visor para el Sistema de Observación de LIDER
///
///  En zonas muestra histograma de flujos y componentes. Encabezado de totales
///  en calef y refrig. También datos por meses (cal y ref)
///  En plantas muestra totales por planta e histograma por zonas de cal y ref.
///  
///  Copyright (C) 2014-20 Rafael Villar Burke <pachi@rvburke.com>
///
///  This program is free software; you can redistribute it and/or
///  modify it under the terms of the GNU General Public License
///  as published by the Free Software Foundation; either version 2
///  of the License, or (at your option) any later version.
///  
///  This program is distributed in the hope that it will be useful,
///  but WITHOUT ANY WARRANTY; without even the implied warranty of
///  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
///  GNU General Public License for more details.
///
///  You should have received a copy of the GNU General Public License
///  along with this program; if not, write to the Free Software
///  Foundation, Inc., 51 Franklin Street, Fifth Floor, Boston, MA
///  02110-1301, USA.
use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use cairo;
use chrono::prelude::*;
use gdk_pixbuf::Pixbuf;
use gio::prelude::*;
use glib::clone;
use gtk::prelude::*;

use crate::appstate::{AppState, TipoElemento};
use crate::config::Config;
use crate::graphs::histocomponentes::draw_histocomponentes;
use crate::graphs::histomeses::draw_histomeses;
use crate::graphs::horarioszona::draw_zonasgraph;
use crate::graphs::piechart::{draw_piechart, PieMode};
use crate::parsers::types::{type_to_str, TYPE_COMPONENTE, TYPE_EDIFICIO, TYPE_PLANTA, TYPE_ZONA};

// Inspeccionar elementos con CTRL+ SHIFT + D con la app lanzada

/// Crea ventana de aplicación y conecta interfaz con canales para enviar mensajes
///
/// - Recibe la aplicación, el estado de la aplicación y la configuración
pub fn build_ui(
    app: &gtk::Application,
    state: &Rc<RefCell<AppState>>,
    config: &Rc<RefCell<Config>>,
) {
    let ui: gtk::Builder = gtk::Builder::from_file("res/main.ui");
    let window: gtk::ApplicationWindow = ui.get_object("window").unwrap();
    let tv: gtk::TreeView = ui.get_object("treeview").unwrap();

    window.set_application(Some(app));

    let ui_treeview: gtk::TreeView = ui.get_object("treeview").unwrap();
    // Columna de icono (3 del modelo)
    let col = gtk::TreeViewColumn::new();
    col.set_title("Tipo");
    let cell = gtk::CellRendererPixbuf::new();
    col.pack_start(&cell, true);
    col.add_attribute(&cell, "pixbuf", 3);
    ui_treeview.append_column(&col);
    // Columna de texto (0 del modelo)
    let col = gtk::TreeViewColumn::new();
    col.set_title("Nombre");
    let cell = gtk::CellRendererText::new();
    col.pack_start(&cell, true);
    col.add_attribute(&cell, "text", 0);
    ui_treeview.append_column(&col);
    // Crea y conecta el modelo del treeview
    let store = gtk::TreeStore::new(&[
        String::static_type(), // nombre activo (edificio, planta, zona o componente)
        u8::static_type(),     // tipo
        String::static_type(), // zona (lo necesitamos para localizar un componente)
        Pixbuf::static_type(), // Pixbuf
    ]);
    ui_treeview.set_model(Some(&store));

    // Conecta señales -----------

    // Abre selector de archivos y carga datos
    let mnu_filechooser: gtk::ToolButton = ui.get_object("abrirbutton").unwrap();
    mnu_filechooser.connect_clicked(clone!(@weak state, @weak ui => move |_| {
        if let Some(filepath) = openfile() {
            loadfile(&filepath, state.clone(), ui.clone());
        }
        // Seleccionar edificio al recargar
        let tv: gtk::TreeView = ui.get_object("treeview").unwrap();
        tv.set_cursor::<gtk::TreeViewColumn>(&gtk::TreePath::from_indicesv(&[0]), None, false);
    }));

    // Activar pestaña de texto
    let mnu_showtext: gtk::ToggleToolButton = ui.get_object("showtext").unwrap();
    mnu_showtext.connect_toggled(clone!(@weak state, @strong ui => move |button| {
        let ui_swtext: gtk::ScrolledWindow = ui.get_object("scrolledwindowtext").unwrap();
        match button.get_property("active").unwrap().get::<bool>().unwrap().unwrap() {
            true => ui_swtext.show(),
            _ => ui_swtext.hide()
        }
    }));

    // Gráfica de datos horarios de zona
    let da_zonasgraph: gtk::DrawingArea = ui.get_object("zonasgraph").unwrap();
    da_zonasgraph.connect_draw(
        clone!(@weak state => @default-return Inhibit(false), move |widget, cr| {
            // TODO: ver qué datos se mandan y evitamos pasar state
            draw_zonasgraph(widget, cr, state);
            Inhibit(false)
        }),
    );

    // Histograma de componentes de demanda y demandas netas anuales
    let da_histoelementos: gtk::DrawingArea = ui.get_object("histoelementos").unwrap();
    da_histoelementos.connect_draw(
        clone!(@weak state => @default-return Inhibit(false), move |widget, cr| {
            // TODO: obtener del modelo y revisar nombres, etc, porque está copiado de los meses
            let cal_net = [-10.1, -3.4,-3.1,-3.6,17.1,-9.6,22.8,-21.2,-11.1];
            let ref_net = [2.9,2.0,-1.4,1.2,11.3,2.9,13.2,-12.6,19.5];
            draw_histocomponentes(widget, cr, &cal_net, &ref_net);
            Inhibit(true)
        }),
    );

    // Histograma de demanda mensual
    let da_histomeses: gtk::DrawingArea = ui.get_object("histomeses").unwrap();
    da_histomeses.connect_draw(
        clone!(@weak state => @default-return Inhibit(false), move |widget, cr| {
            // TODO: obtener del modelo, en el estado actual [f64;9]
            // XXX: esto es la demanda por componentes, no por meses!
            let calefaccion_meses = [-3.5, -2.3,-1.6,-0.0,-0.0,0.0,0.0,0.0,0.0,0.0,-0.5,-3.1];
            let refrigeracion_meses = [0.0,0.0,0.0,0.0,0.0,3.1,6.9,6.9,2.7,0.0,0.0,0.0];
            draw_histomeses(widget, cr, &calefaccion_meses, &refrigeracion_meses);
            Inhibit(true)
        }),
    );

    // Cal pos
    let da_calpos: gtk::DrawingArea = ui.get_object("pieglobalcalpos").unwrap();
    da_calpos.connect_draw(
        clone!(@weak state => @default-return Inhibit(false), move |widget, cr| {
            // TODO: obtener del modelo, en el estado actual
            let demandas = [0.0, 0.0, 3.2, 17.1, 0.1, 22.8, 7.2, 0.1];
            draw_piechart(widget, cr, &demandas, PieMode::CalPos);
            Inhibit(true)
        }),
    );

    // Cal neg
    let da_calneg: gtk::DrawingArea = ui.get_object("pieglobalcalneg").unwrap();
    da_calneg.connect_draw(
        clone!(@weak state => @default-return Inhibit(false), move |widget, cr| {
            // TODO: obtener del modelo, en el estado actual
            let demandas = [0.0, 3.2, 17.1, 0.1, 22.8, 7.2, 0.1, 0.0];
            draw_piechart(widget, cr, &demandas, PieMode::CalNeg);
            Inhibit(true)
        }),
    );

    // Ref pos
    let da_refpos: gtk::DrawingArea = ui.get_object("pieglobalrefpos").unwrap();
    da_refpos.connect_draw(
        clone!(@weak state => @default-return Inhibit(false), move |widget, cr| {
            // TODO: obtener del modelo, en el estado actual
            let demandas = [3.2, 17.1, 0.1, 22.8, 7.2, 0.1, 0.0, 0.0];
            draw_piechart(widget, cr, &demandas, PieMode::RefPos);
            Inhibit(true)
        }),
    );

    // Ref neg
    let da_refneg: gtk::DrawingArea = ui.get_object("pieglobalrefneg").unwrap();
    da_refneg.connect_draw(
        clone!(@weak state => @default-return Inhibit(false), move |widget, cr| {
            
            // TODO: obtener del modelo, en el estado actual
            let demandas = [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
            //let demandas = [17.1, 0.1, 22.8, 7.2, 0.1, 0.0, 0.0, 3.2];
            draw_piechart(widget, cr, &demandas, PieMode::RefNeg);
            gtk::Inhibit(false)
        }),
    );

    // Guarda pantallazo de la gráfica actual
    let mnu_screenshot: gtk::ToolButton = ui.get_object("savebutton").unwrap();
    mnu_screenshot.connect_clicked(clone!(@weak state, @weak config, @strong ui => move |_| {
        let nb: gtk::Notebook = ui.get_object("notebook").unwrap();
        let idx = nb.get_current_page();
        let container = nb.get_nth_page(idx).unwrap();
        let config = config.borrow();
        let out_dpi = config.out_dpi; // 100
        let out_fmt = &config.out_fmt; // '%Y%m%d_%H%M%S'
        let out_basename = &config.out_basename; // 'ViSol'
        let scale = out_dpi as f64 / 72.0;
        for child in container.downcast::<gtk::Container>().unwrap().get_children() {
            if child.is::<gtk::DrawingArea>() {
                // Dibuja widget en superficie
                let size = child.get_allocation();
                let surf = cairo::ImageSurface::create(cairo::Format::ARgb32, (size.width as f64 * scale) as i32, (size.height as f64 * scale) as i32).expect("No se ha podido crear la superficie cairo");
                let ctx = cairo::Context::new(&surf);
                ctx.scale(scale, scale);
                child.draw(&ctx);
                // Genera nombre de archivo
                let st = state.borrow();
                let timestamp = Local::now().format(out_fmt).to_string();
                let filename = format!("{}-{}-{}.png", timestamp, out_basename, st.filename().expect("Sin nombre de archivo para el modelo actual").display());
                let pathname = st.dirname().expect("Sin ruta para el modelo actual").join(filename);
                let mut outfile = std::fs::File::create(&pathname).expect("No se ha podido crear el archivo");
                // Guarda superficie en archivo y notifica a la app
                surf.write_to_png(&mut outfile).expect("No ");
                let sb: gtk::Statusbar = ui.get_object("statusbar").unwrap();
                sb.push(0, &format!("Guardando captura de pantalla: {}", pathname.display()));
                break
            }
        }
    }
    ));

    // Selecciona nueva fila al cambiar el cursor en la vista de árbol
    ui_treeview.connect_cursor_changed(clone!(@weak state, @strong ui => move |tv| {
        let selection = tv.get_selection();
        if let Some((model, iter)) = selection.get_selected() {
            let sb: gtk::Statusbar = ui.get_object("statusbar").unwrap();
            let labelzona: gtk::Label = ui.get_object("labelzona").unwrap();

            let nombre: String = model.get_value(&iter, 0).get().unwrap().unwrap();
            let tipo: u8 = model.get_value(&iter, 1).get_some().unwrap();
            // let zn = model.get_value(&iter, 3).get::<String>().unwrap().unwrap();
            let selected_type = match tipo {
                TYPE_EDIFICIO => TipoElemento::Edificio,
                TYPE_PLANTA => TipoElemento::Planta,
                TYPE_ZONA => TipoElemento::Zona,
                TYPE_COMPONENTE => TipoElemento::Componente,
                _ => TipoElemento::None,
            };

            let mut model = state.borrow_mut();
            model.curr_type = selected_type;
            model.curr_name = nombre.clone();
            let (mul, sup, cal, refr) = model.basicdata().unwrap();

            let mut txt1 = format!("<big><b>{}</b></big> ({})\n", nombre, type_to_str(tipo));
            match tipo {
                TYPE_EDIFICIO | TYPE_PLANTA | TYPE_ZONA => {
                    txt1.push_str(&format!("<i>{} x {:.2}m²</i>\n", mul, sup));
                    txt1.push_str(&format!("calefacción: {:6.1}<i>kWh/m²año</i>, ", cal));
                    txt1.push_str(&format!("refrigeración: {:6.1}<i>kWh/m²año</i>", refr));
                },
                _ => {
                    txt1.push('\n');
                }
            };
            sb.push(0, &format!("Seleccionado {}: {}", type_to_str(tipo), nombre));
            labelzona.set_property("label", &txt1).expect("Fallo al establecer etiqueta");
        }
    }));

    // Modifica el número de flujos activos en la vista de elementos

    // Botón de menú about
    let mnu_about: gtk::ToolButton = ui.get_object("aboutbutton").expect("aboutbutton not found");
    mnu_about.connect_clicked(clone!(@weak window => move |_| {
        show_about(&window);
    }));

    // Prepara gactions y conéctalas a la aplicación ("app") o la ventana principal ("win"). Hay macro en gio::prelude
    // Ver https://stackoverflow.com/questions/55344630/how-to-connect-buttons-to-actions-in-custom-simpleactiongroups-in-gtk-rs

    // win.quit
    let action = gio::SimpleAction::new("quit", None);
    action.connect_activate(clone!(@weak window => move |_, _| {
        window.close();
    }));
    window.add_action(&action);

    // win.cbelementos
    let action = gio::SimpleAction::new("cbelementos", None);
    action.connect_activate(clone!(@weak window => move |_, _| {
        // TODO: Ver antigua función cbelementos en lugar de esto
        // Determina si se añaden calpos, calneg, refpos, refneg en las gráficas
        window.close();
    }));
    window.add_action(&action);
    window.show_all();

    let mut testfile = std::env::current_dir().unwrap();
    testfile.push("data/test.res");
    loadfile(testfile, state.clone(), ui.clone());

    // Seleccionar edificio al arrancar
    tv.set_cursor::<gtk::TreeViewColumn>(&gtk::TreePath::from_indicesv(&[0]), None, false);
}

fn loadfile<P: AsRef<Path>>(path: P, state: Rc<RefCell<AppState>>, ui: gtk::Builder) {
    let sb: gtk::Statusbar = ui.get_object("statusbar").unwrap();
    let window: gtk::ApplicationWindow = ui.get_object("window").unwrap();

    let path = path.as_ref();
    if !path.exists() {
        sb.push(0, &format!("Error al leer archivo: {}", path.display()));
    } else {
        let mut state = state.borrow_mut();
        state.load_data(&Some(path.to_path_buf()));
        sb.push(0, &format!("Seleccionado archivo: {}", path.display()));
        let mut pth = path.display().to_string();
        let pth: String = pth.drain(..std::cmp::max(0, pth.len() - 40)).collect(); // Recortar a máx 40 caracteres
        window.set_title(&format!("ViSOL [... {}]", &pth));

        let e = state.edificio.as_ref().unwrap();

        // Contenido del control de texto
        let ui_tb: gtk::TextBuffer = ui.get_object("textbuffer").unwrap();
        ui_tb.set_text(&e.resdata.clone());

        // Árbol
        let tv: gtk::TreeView = ui.get_object("treeview").unwrap();
        tv.collapse_all();
        let ts = tv
            .get_model()
            .unwrap()
            .downcast::<gtk::TreeStore>()
            .unwrap();
        ts.clear();

        // # Modelo de plantas y zonas
        let edificio_icon = Pixbuf::from_file("./res/edificioicono.png").unwrap();
        let planta_icon = Pixbuf::from_file("./res/plantaicono.png").unwrap();
        let zona_icon = Pixbuf::from_file("./res/zonaicono.png").unwrap();
        let componente_icon = Pixbuf::from_file("./res/componenteicono.png").unwrap();

        // Empieza con el edificio
        let edificioiter = ts.insert_with_values(
            None,
            None,
            &[0, 1, 2, 3],
            &[&e.nombre, &TYPE_EDIFICIO, &"", &edificio_icon],
        );

        // Carga las plantas
        for planta in &e.plantas {
            let plantaiter = ts.insert_with_values(
                Some(&edificioiter),
                None,
                &[0, 1, 2, 3],
                &[&planta.nombre, &TYPE_PLANTA, &"", &planta_icon],
            );
            // Las zonas de las plantas
            for zona in &planta.zonas {
                let zonaiter = ts.insert_with_values(
                    Some(&plantaiter),
                    None,
                    &[0, 1, 2, 3],
                    &[&zona, &TYPE_ZONA, &zona, &zona_icon],
                );
                // Expande hasta el nivel de zonas
                tv.expand_to_path(ts.get_path(&zonaiter).as_ref().unwrap());
                // Carga los componentes de las zonas
                for componente in &e.zonas.get(zona).unwrap().componentes {
                    ts.insert_with_values(
                        Some(&zonaiter),
                        None,
                        &[0, 1, 2, 3],
                        &[
                            &componente.nombre,
                            &TYPE_COMPONENTE,
                            &zona,
                            &componente_icon,
                        ],
                    );
                }
            }
        }
        sb.push(0, &format!("Cargado modelo: {}", path.display()));
    }
}

/// Abre archivo de resultados
// Ver https://github.com/gtk-rs/examples/blob/master/src/bin/text_viewer.rs para no usar UI
fn openfile() -> Option<PathBuf> {
    let ui: gtk::Builder = gtk::Builder::from_file("res/filechooser.ui");
    let chooser: gtk::FileChooserDialog = ui
        .get_object("filechooserdialog")
        .expect("Couldn't get filechooserdialog");
    chooser.add_buttons(&[
        ("Open", gtk::ResponseType::Ok),
        ("Cancel", gtk::ResponseType::Cancel),
    ]);

    let res = if chooser.run() == gtk::ResponseType::Ok {
        chooser.get_filename()
    } else {
        None
    };
    chooser.close();
    res
}

/// Muestra ventana de créditos
fn show_about(window: &gtk::ApplicationWindow) {
    let builder: gtk::Builder = gtk::Builder::from_file("res/about.ui");
    let about_dialog: gtk::AboutDialog = builder
        .get_object("aboutdialog")
        .expect("aboutdialog not found");
    about_dialog.set_modal(true);
    about_dialog.set_transient_for(Some(window));
    about_dialog.show();
}
