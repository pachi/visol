// TODO: implementar serialización y / o lectura desde disco ui/visol.cfg

//! Configuración de la aplicación ViSOL

/// Datos de configuración de la aplicación
#[derive(Debug, Clone)]
pub struct Config {
    /// Límite automático de la demanda
    pub autolimits: bool,
    /// Límite superior de las escalas
    pub maxlimit: i32,
    /// Límite inferior de las escalas
    pub minlimit: i32,
    /// Resolución de salida de los pantallazos
    pub out_dpi: i32,
    /// Formato de fecha/hora de los pantallazos
    pub out_fmt: String,
    /// Nombre base de los pantallazos
    pub out_basename: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            autolimits: true,
            maxlimit: 50,
            minlimit: -150,
            out_dpi: 150,
            out_fmt: "%Y%m%d_%H%M%S".into(),
            out_basename: "ViSol".into(),
        }
    }
}
