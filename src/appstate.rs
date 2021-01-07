use crate::parsers::{bin::BinData, res::EdificioLIDER};
use std::{
    ffi::OsString,
    path::{Path, PathBuf},
};

/// Tipo de elemento activo
#[allow(unused)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum TipoElemento {
    Edificio = 0,
    Planta = 1,
    Zona = 2,
    Componente = 3,
    None = 4
}

impl Default for TipoElemento {
    fn default() -> Self {
        Self::Edificio
    }
}


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
    /// Tipo de elemento activo
    pub curr_type: TipoElemento,
    /// Nombre del elemento activo
    pub curr_name: String,
}

impl AppState {
    /// Crea estado inicial de la aplicación
    pub fn new() -> Self {
        Self {
            // TODO: usar aquí path por defecto
            ..Default::default()
        }
    }

    /// Nombre del archivo activo, sin extensión
    pub fn filename(&self) -> Option<&Path> {
        self.respath
            .as_ref()
            .map_or(None, |v| v.file_stem().map(Path::new))
    }
    /// Directorio del archivo activo
    pub fn dirname(&self) -> Option<&Path> {
        self.respath.as_ref().map_or(None, |v| v.parent())
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
                        Some(PathBuf::from(respathdir.join(&binfile)))
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
        if self.curr_type == TipoElemento::None {return None};
        if let Some(edificio) = self.edificio.as_ref() {
            Some(edificio.basicdata(self.curr_type as u8, &self.curr_name))
        } else { None }
    }
}
