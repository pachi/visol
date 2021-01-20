//! Lectura de archivos binarios de datos de zonas de LIDERE (.bin)
//!
//! La estructura del formato está documentada en el archivo "esto2_nucleo.jar",
//! en el archivo LeeZonasLIDER_2.h

// use std::io::prelude::*;
use std::{collections::BTreeMap, convert::{TryFrom, TryInto}};
use std::io::{BufReader, Read};
use std::{fs::File, path::Path};

type Error = Box<dyn std::error::Error + 'static>;

// TODO: probar a hacer type BinData = Vec<ZonaLider> ya que no necesitamos el numzonas, con len()
#[derive(Debug, Clone)]
pub struct BinData {
    /// Número de zonas
    pub numzonas: u32,
    /// Datos de zonas
    pub zonas: BTreeMap<String, ZonaLider>,
}

// 285896 bytes
const ZONEDATADISKSIZE: usize = std::mem::size_of::<ZonaLiderFFI>();

impl BinData {
    /// Convierte archivo .bin a estructura BinData
    /// Los primeros 4 bytes (i32) contienen el número de zonas
    /// El resto del archivo contiene ese número de estructuras ZonaLiderFFI
    pub fn from_file<S: AsRef<Path>>(path: S) -> Result<Self, Error> {
        let mut file = BufReader::new(File::open(path)?);
        // Lee número de zonas numzonas
        let buf = &mut [0u8; 4];
        file.read_exact(buf)?;
        let numzonas = u32::from_le_bytes(*buf);
        // Lee vector de ZonaLiderFFI con numzonas elementos
        let numbytes = numzonas as usize * ZONEDATADISKSIZE;
        let mut bytezones = Vec::<ZonaLiderFFI>::with_capacity(numzonas as usize);
        unsafe {
            let buffer =
                std::slice::from_raw_parts_mut(bytezones.as_mut_ptr() as *mut u8, numbytes);
            file.read_exact(buffer)?;
            bytezones.set_len(numzonas as usize);
        }
        // Convierte desde vector de ZonaLiderFFI a HashMap de ZonaLider
        let mut zonas = BTreeMap::<String, ZonaLider>::new();
        for bytezone in bytezones.iter() {
            let zona: ZonaLider = bytezone.try_into()?;
            zonas.insert(zona.nombre.clone(), zona);
        }
        // Devolvemos BinData
        Ok(Self { numzonas, zonas })
    }
}

/// Número de horas en un año
const NHORAS: usize = 8760;

/// Número máximo de zonas adyacentes a una zona
const MAXADJZONAS: usize = 100;

#[derive(Clone)]
pub struct ZonaLider {
    ///nombreZona: Nombre de la zona
    pub nombre: String,
    /// Area: Superficie de la zona [m2]
    pub area: f32,
    /// Volumen: Volumen de la zona []m3]
    pub volumen: f32,
    /// multiplicador: Multiplicador de la zona
    pub multiplicador: i32,
    /// p: Factores de respuesta de la zona (p)
    ///    ante ganancia térmica
    ///    (cálculo de la carga sensible sobre los equipos con RTS)
    pub p: Vec<f32>,
    /// g: Factores de respuesta (g) de la zona
    ///    ante cambio de la temperatura
    ///    (cálculo de la carga sensible sobre los equipos con RTS)
    pub g: Vec<f32>,
    /// localAdyacente: Nombres de las zonas adyacentes
    pub adyacentes: Vec<String>,
    /// UAint: UA con las zonas adyacentes [W/K]
    pub ua_int: Vec<f32>,
    /// UAext: UA con el exterior [W/K]
    pub ua_ext: f32,
    /// daCal: 1|0 para on|off de demanda de calefacción
    pub da_cal: Vec<i32>,
    /// daRef: 1|0 para on|off de demanda de refrigeración
    pub da_ref: Vec<i32>,
    /// QS: Carga sensible de la zona [W?]
    pub q_sen: Vec<f32>,
    /// QL: Carga latente de la zona [W?]
    pub q_lat: Vec<f32>,
    /// Treal: Temperatura del local [ºC]
    pub t_real: Vec<f32>,
    /// Tmax: Temperatura de consigna alta [ºC]
    pub t_max: Vec<f32>,
    /// Tmin: Temperatura de consigna baja [ºC]
    pub t_min: Vec<f32>,
    /// Vventinf: Caudal másico de ventilación e infiltración [kg/s?]
    pub v_ventinf: Vec<f32>,
}

impl Default for ZonaLider {
    fn default() -> Self {
        ZonaLider {
            nombre: String::new(),
            area: 0.0f32,
            volumen: 0.0f32,
            multiplicador: 0i32,
            p: vec![0f32; 2],
            g: vec![0f32; 24],
            adyacentes: vec![],
            ua_int: vec![],
            ua_ext: 0f32,
            da_cal: vec![0i32; NHORAS],
            da_ref: vec![0i32; NHORAS],
            q_sen: vec![0.0; NHORAS],
            q_lat: vec![0.0; NHORAS],
            t_real: vec![0.0; NHORAS],
            t_max: vec![0.0; NHORAS],
            t_min: vec![0.0; NHORAS],
            v_ventinf: vec![0.0; NHORAS],
        }
    }
}

/// Formatea vector para mostrar solo los 3 primeros y últimos valores
macro_rules! format_vec_lider {
    ($v:expr) => {
        &format_args!(
            "[{}, {}, {} ... {}, {}, {}]",
            $v[0],
            $v[1],
            $v[2],
            $v[$v.len() - 3],
            $v[$v.len() - 2],
            $v[$v.len() - 1]
        )
    };
}

impl std::fmt::Debug for ZonaLider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ZonaLider")
            .field("nombre", &self.nombre)
            .field("volumen", &self.volumen)
            .field("multiplicador", &self.multiplicador)
            .field("p", &self.p)
            .field("g", &self.g)
            .field("adyacentes", &self.adyacentes)
            .field("ua_int", &self.ua_int)
            .field("ua_ext", &self.ua_ext)
            // Mostrar solo los 3 primeros y últimos valores de datos horarios
            .field("da_cal", format_vec_lider!(&self.da_cal))
            .field("da_ref", format_vec_lider!(&self.da_ref))
            .field("q_sen", format_vec_lider!(&self.q_sen))
            .field("q_lat", format_vec_lider!(&self.q_lat))
            .field("t_real", format_vec_lider!(&self.t_real))
            .field("t_max", format_vec_lider!(&self.t_max))
            .field("t_min", format_vec_lider!(&self.t_min))
            .field("v_ventinf", format_vec_lider!(&self.v_ventinf))
            .finish()
    }
}

/// Lee nombre de zona a partir de cadena de bytes
/// La cadena está rellena con \0 con una longitud fija
/// Eliminamos las comillas que rodean el nombre (convención HULC)
fn read_zonename_from_u8(u8vec: &[u8]) -> String {
    let nul_range_end = u8vec
        .iter()
        .position(|&c| c == b'\0')
        .unwrap_or(u8vec.len());
    String::from_utf8(u8vec[0..nul_range_end].to_vec())
        .unwrap()
        .trim_matches('\"')
        .to_string()
}

impl TryFrom<&ZonaLiderFFI> for ZonaLider {
    type Error = Error;

    fn try_from(z: &ZonaLiderFFI) -> Result<Self, Self::Error> {
        let num_adyacentes = (z.num_adyacentes + 1) as usize; // va de 0 a localNextZones
        let adyacentes: Vec<String> = z.adyacentes[0..num_adyacentes]
            .iter()
            .map(|s| read_zonename_from_u8(s))
            .collect();
        Ok(Self {
            nombre: read_zonename_from_u8(&z.nombre),
            area: z.area,
            volumen: z.volumen,
            multiplicador: z.multiplicador,
            p: z.p.to_vec(),
            g: z.g.to_vec(),
            adyacentes,
            ua_int: z.ua_int[0..num_adyacentes].to_vec(),
            ua_ext: z.ua_ext,
            da_cal: z.da_cal.to_vec(),
            da_ref: z.da_ref.to_vec(),
            q_sen: z.q_sen.to_vec(),
            q_lat: z.q_lat.to_vec(),
            t_real: z.t_real.to_vec(),
            t_max: z.t_max.to_vec(),
            t_min: z.t_max.to_vec(),
            v_ventinf: z.v_ventinf.to_vec(),
        })
    }
}

#[repr(C)]
/// Estructura ZonaLider en archivo .bin
///
/// La estructura del formato está documentada en el archivo "esto2_nucleo.jar",
/// en el archivo LeeZonasLIDER_2.h
///
/// Incluye un entero (4 bytes) con el número de estructuras, que se describen a continuación
///
///     struct zonaLIDER {
///       char nombreZona[50];
///       float Area;
///       float Volumen;
///       int multiplicador;
///       float p[2];
///       float g[24];
///       int numLocalesAdyacentes;
///       float UAext;
///       float UAint[100];
///       char localAdyacente[100][50];
///       int daCal[8760];
///       int daRef[8760];
///       float QS[8760];
///       float QL[8760];
///       float Treal[8760];
///       float Tmax[8760];
///       float Tmin[8760];
///       float Vventinf[8760];
///     };
/// Ver descripción de los factores p y g en:
///     IDAE, "Guía técnica. Procedimientos y aspectos de la simulación de instalaciones
///     térmicas en edificios", pp.50-51 y Anexo 6.

pub struct ZonaLiderFFI {
    /// nombreZona: Nombre de la zona.
    /// char nombreZona[50];
    pub nombre: [u8; 50],
    /// Area: Superficie de la zona [m2]
    /// float Area;
    pub area: f32,
    /// Volumen: Volumen de la zona []m3]
    /// float Volumen;
    pub volumen: f32,
    /// multiplicador: Multiplicador de la zona
    /// int multiplicador;
    pub multiplicador: i32,
    /// p: Factores de respuesta de la zona (p)
    ///    ante ganancia térmica
    ///    (cálculo de la carga sensible sobre los equipos con RTS)
    /// float p[2];
    pub p: [f32; 2],
    /// g: Factores de respuesta (g) de la zona
    ///    ante cambio de la temperatura
    ///    (cálculo de la carga sensible sobre los equipos con RTS)
    /// float g[24];
    pub g: [f32; 24],
    /// numLocalesAdyacentes: Número de zonas adyacentes (de 0 a numLocalesAdyacentes)
    /// int numLocalesAdyacentes;
    pub num_adyacentes: i32,
    /// UAext: UA con el exterior [W/K]
    /// float UAext;
    pub ua_ext: f32,
    /// UAint: UA con las zonas adyacentes [W/K]
    /// float UAint[100];
    pub ua_int: [f32; MAXADJZONAS],
    /// localAdyacente: Nombres de las zonas adyacentes
    /// char localAdyacente[100][50];
    pub adyacentes: [[u8; 50]; MAXADJZONAS],
    /// daCal: 1|0 para on|off de demanda de calefacción
    /// int daCal[nHoras];
    pub da_cal: [i32; NHORAS],
    /// daRef: 1|0 para on|off de demanda de refrigeración
    /// int daRef[nHoras];
    pub da_ref: [i32; NHORAS],
    /// QS: Carga sensible de la zona [W?]
    /// float QS[nHoras];
    pub q_sen: [f32; NHORAS],
    /// QL: Carga latente de la zona [W?]
    /// float QL[nHoras];
    pub q_lat: [f32; NHORAS],
    /// Treal: Temperatura del local [ºC]
    /// float Treal[nHoras];
    pub t_real: [f32; NHORAS],
    /// Tmax: Temperatura de consigna alta [ºC]
    /// float Tmax[nHoras];
    pub t_max: [f32; NHORAS],
    /// Tmin: Temperatura de consigna baja [ºC]
    /// float Tmin[nHoras];
    pub t_min: [f32; NHORAS],
    /// Vventinf: Caudal másico de ventilación e infiltración [kg/s?]
    /// float Vventinf[nHoras];
    pub v_ventinf: [f32; NHORAS],
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_bin_testfile() {
        let mut testfile = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        testfile.push("./src/data/test.bin");
        eprintln!("Parsear binfile: {}", testfile.display());
        let res = BinData::from_file(testfile).unwrap();
        assert_eq!(10, res.numzonas);
        let zone0 = &res.zonas["P01_E01"];
        assert_eq!(zone0.nombre, "P01_E01");
        assert_eq!(zone0.area, 25.0373287200928);
        assert_eq!(zone0.multiplicador, 1);
        assert_eq!(zone0.volumen, 62.5933227539062);
        assert_eq!(zone0.p, &[1.0, -0.9554443359375]);
        assert_eq!(
            zone0.g,
            &[
                186.3824,
                -194.9448,
                3.184631,
                1.8466333,
                1.2096132,
                0.83193725,
                0.5903781,
                0.42930642,
                0.31856796,
                0.24052902,
                0.18438642,
                0.14326464,
                0.11267228,
                0.08959523,
                0.0719719,
                0.058364194,
                0.047740776,
                0.03937716,
                0.032728218,
                0.027403418,
                0.023101456,
                0.019599736,
                0.016730765,
                0.014363277
            ]
        );
        assert_eq!(
            zone0.adyacentes,
            &["P01_E02", "P01_E04", "P02_E01", "P02_E03", "P02_E05"]
        );
        assert_eq!(
            zone0.ua_int,
            &[26.382734, 13.981942, 7.7620816, 8.6559, 1.0642923]
        );
        assert_eq!(zone0.ua_ext, 28.203636);
        assert_eq!(
            &zone0.v_ventinf[0..6],
            &[
                0.015308376,
                0.01476276,
                0.014628861,
                0.014651622,
                0.014688805,
                0.014711768
            ]
        );
        // println!("Struct 0: {:?}", res);
    }
}
