pub use crate::parsers::types::TipoObjeto;
use crate::parsers::{bin::BinData, res::EdificioLIDER, types::FlujosVec};
use std::{
    convert::From,
    ffi::OsString,
    path::{Path, PathBuf},
};

// const image_buffer_path: &str = "/tmp/automata_buffer.png";
#[derive(Debug, Default, Clone)]
pub struct AppState {
    /// Ruta completa al archivo de datos de HULC
    pub respath: Option<PathBuf>,
    /// Datos del edificio
    pub edificio: Option<EdificioLIDER>,
    /// Ruta completa al archivo bin
    pub binpath: Option<PathBuf>,
    /// Datos del archivo .bin
    pub bindata: Option<BinData>,
    /// Tipo de objeto activo
    pub curr_obj_type: TipoObjeto,
    /// Nombre del objeto activo (Edificio, Planta, Zona, Elemento)
    pub curr_name: String,
    /// Nombre de la zona activa (es neceario para localizar un elemento de esa zona)
    pub curr_zone: String,
    /// Muestra detalle de componentes (cal+, cal-, ref+, ref-, además de calnet y refnet)
    pub show_detail: bool,
}

impl AppState {
    /// Crea estado inicial de la aplicación
    /// ¿Tendría sentido usar un path por defecto
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    /// Nombre del archivo activo, sin extensión
    pub fn filename(&self) -> Option<&Path> {
        self.respath
            .as_ref()
            .and_then(|v| v.file_stem().map(Path::new))
    }
    /// Directorio del archivo activo
    pub fn dirname(&self) -> Option<&Path> {
        self.respath.as_ref().and_then(|v| v.parent())
    }

    /// Localiza archivo bin en directorio de proyecto
    ///
    /// Probamos primero a ver si hay un bin con el mismo nombre que el res,
    /// luego uno con ResumenRCC_nombrearchivores.bin y finalmente el primero que encuentre.
    fn find_bin(&self) -> Option<PathBuf> {
        let respathdir = self.dirname().unwrap();
        let files = respathdir.read_dir().unwrap();
        let binfiles = files
            .filter_map(Result::ok)
            .filter(|d| d.path().extension().map(|e| e.to_str().unwrap_or("")) == Some("bin"))
            .map(|e| e.path().file_name().map(|s| s.to_os_string()))
            .collect::<Option<Vec<OsString>>>();
        match binfiles {
            Some(binfiles) => {
                match self.filename() {
                    Some(filename) => {
                        let samename = filename.with_extension(".bin").into_os_string();

                        let mut rccname = OsString::from("ResumenRCC_");
                        rccname.push(&samename);

                        let binfile = if binfiles.contains(&samename) {
                            // Caso 1: nombre del archivo .res pero con extensión .bin
                            samename
                        } else if binfiles.contains(&rccname) {
                            // Caso 2: ResumenRCC_ + archivores + .bin
                            rccname
                        } else {
                            // Caso 3: primer .bin encontrado
                            binfiles.get(0).unwrap().clone()
                        };
                        Some(respathdir.join(&binfile))
                    }
                    None => None,
                }
            }
            None => None,
        }
    }

    /// Carga archivo activo y datos asociados
    pub fn load_data(&mut self, path: &Option<PathBuf>) {
        if path != &self.respath {
            match path {
                None => {
                    self.respath = None;
                    self.edificio = None;
                    self.binpath = None;
                    self.bindata = None;
                }
                Some(pth) => {
                    if pth.exists() {
                        self.respath = path.clone();
                        self.edificio = EdificioLIDER::from_file(pth).ok();
                        // println!("Cargado edificio: {:#?}", &self.edificio);
                        if let Some(binpath) = self.find_bin() {
                            self.bindata = BinData::from_file(&binpath).ok();
                            self.binpath = Some(binpath);
                        } else {
                            self.bindata = None;
                            self.binpath = None;
                        }
                    }
                }
            };
        }
    }

    /// Devuelve parámetros básicos del objeto de nombre y zona dados
    /// (multiplicador, superficie, calefaccion, refrigeracion)
    pub fn basicdata(&self) -> Option<(i32, f32, f32, f32)> {
        if self.curr_obj_type == TipoObjeto::None {
            return None;
        };
        self.edificio
            .as_ref()
            .map(|e| e.basicdata(self.curr_obj_type as u8, &self.curr_name))
    }

    /// Datos mensuales de demanda de calefacción y refrigeración
    /// No está definido para elementos constructivos o sin edificio definido
    pub fn calref_monthly_data(&self) -> (Vec<f32>, Vec<f32>) {
        match self.curr_obj_type {
            TipoObjeto::Edificio => self
                .edificio
                .as_ref()
                .map(|e| (e.calefaccion_meses.clone(), e.refrigeracion_meses.clone())),
            TipoObjeto::Planta => self.edificio.as_ref().and_then(|e| {
                e.plantas
                    .iter()
                    .find(|p| p.nombre == self.curr_name)
                    .map(|p| (p.calefaccion_meses(e), p.refrigeracion_meses(e)))
            }),
            TipoObjeto::Zona => self.edificio.as_ref().and_then(|e| {
                e.zonas
                    .get(&self.curr_name)
                    .map(|z| (z.calefaccion_meses.clone(), z.refrigeracion_meses.clone()))
            }),
            TipoObjeto::Elemento | TipoObjeto::None => None,
        }
        .unwrap_or((vec![0.0; 12], vec![0.0; 12]))
    }

    /// Valores de flujos de calor por conceptos
    /// Cuando no hay selección se devuelve todo a cero
    pub fn concepts_data(&self) -> FlujosVec {
        match self.curr_obj_type {
            TipoObjeto::Edificio => self
                .edificio
                .as_ref()
                .map(EdificioLIDER::conceptos)
                .map(|c| c.to_flows()),
            TipoObjeto::Planta => self
                .edificio
                .as_ref()
                .and_then(|e| {
                    e.plantas
                        .iter()
                        .find(|p| p.nombre == self.curr_name)
                        .map(|p| p.conceptos(e))
                })
                .map(|c| c.to_flows()),
            TipoObjeto::Zona => self
                .edificio
                .as_ref()
                .and_then(|e| e.zonas.get(&self.curr_name).map(|z| z.conceptos))
                .map(|c| c.to_flows()),
            TipoObjeto::Elemento => self
                .edificio
                .as_ref()
                .and_then(|e| {
                    e.zonas.get(&self.curr_zone).and_then(|z| {
                        z.elementos
                            .iter()
                            .find(|el| el.nombre == self.curr_name)
                            .map(|el| el.flujos)
                    })
                })
                .map(|c| c.to_flows()),
            TipoObjeto::None => None,
        }
        .unwrap_or_default()
    }
}
