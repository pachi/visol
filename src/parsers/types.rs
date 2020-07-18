//! Tipos para la representación de archivos de resultados de LIDER
//! - EdificioLIDER
//! - PlantaLIDER
//! - ZonaLIDER
//! - Componente

use crate::utils::Error;
use std::{
    collections::HashMap,
    ops::{Add, Mul},
};

// const CONCEPTOSLIDER: [&str; 9] = [
//     "Paredes Exteriores",
//     "Cubiertas",
//     "Suelos",
//     "Puentes Térmicos",
//     "Solar Ventanas",
//     "Transmisión Ventanas",
//     "Fuentes Internas",
//     "Ventilación más Infiltración",
//     "TOTAL",
// ];

/// Tipos de objetos y estados de la aplicación
pub const TYPE_EDIFICIO: u8 = 0;
pub const TYPE_PLANTA: u8 = 1;
pub const TYPE_ZONA: u8 = 2;
pub const TYPE_COMPONENTE: u8 = 3;

/// Conversión de tipo / modo a str
pub fn type_to_str(mode: u8) -> &'static str {
    match mode {
        TYPE_EDIFICIO => "EDIFICIO",
        TYPE_PLANTA => "PLANTA",
        TYPE_ZONA => "ZONA",
        TYPE_COMPONENTE => "COMPONENTE",
        _ => "",
    }
}

#[derive(Clone, Debug, Default)]
/// Edificio en LIDER
pub struct EdificioLIDER {
    /// Nombre del edificio
    pub nombre: String,
    /// Superficie del edificio [m²]
    pub superficie: f32,
    /// Demanda anual de calefacción del edificio [kWh/m²/año]
    pub calefaccion: f32,
    /// Demanda anual de refrigeración del edificio  [kWh/m²/año]
    pub refrigeracion: f32,
    /// Demandas mensuales de calefacción del edificio [kWh/m²/mes]
    pub calefaccion_meses: Vec<f32>,
    /// Demandas mensuales de refrigeración del edificio [kWh/m²/mes]
    pub refrigeracion_meses: Vec<f32>,
    /// Contenido del archivo .RES del edificio
    pub resdata: String,
    /// Plantas del edificio
    pub plantas: Vec<PlantaLIDER>,
    /// Zonas del edificio
    pub zonas: HashMap<String, ZonaLIDER>,
}

impl EdificioLIDER {
    /// Devuelve parámetros básicos del objeto de nombre y zona dados
    /// (multiplicador, superficie, calefaccion, refrigeracion)
    pub fn basicdata(&self, mode: u8, nombre: &str) -> (i32, f32, f32, f32) {
        match mode {
            TYPE_EDIFICIO => (1, self.superficie, self.calefaccion, self.refrigeracion),
            TYPE_PLANTA => {
                let planta = self.plantas.iter().find(|p| p.nombre == nombre).unwrap();
                (
                    1,
                    planta.superficie(self),
                    planta.calefaccion(self),
                    planta.refrigeracion(self),
                )
            }
            TYPE_ZONA => {
                let zona = self.zonas.get(nombre).unwrap();
                (
                    zona.multiplicador,
                    zona.superficie,
                    zona.calefaccion,
                    zona.refrigeracion,
                )
            }
            _ => (0, 0.0, 0.0, 0.0),
        }
    }

    /// Flujos por conceptos del edificio [kW/m²·año]
    /// Se obtienen a partir de los de las plantas, ponderando por superficies
    fn conceptos(&self, ed: &EdificioLIDER) -> Conceptos {
        let mut conceptos = Conceptos::default();
        if self.superficie.abs() < f32::EPSILON {
            return conceptos;
        };

        for planta in &self.plantas {
            let p_conc = planta.conceptos(ed);
            let p_sup = planta.superficie(ed);
            conceptos = conceptos + p_sup * p_conc;
        }
        conceptos * (1.0 / self.superficie)
    }

    pub fn minmaxgrupos(&self) -> (Componente, Componente) {
        //     def minmaxgrupos(self):
        //         """Flujo máximo y mínimo de grupos en todas las zonas del edificio  [kW/m²·año]"""
        //         zonas = self.zonas
        //         names = self.gruposlider
        //         pmin = min(min(zona.grupos[name].values) for zona in zonas for name in names)
        //         pmax = max(max(zona.grupos[name].values) for zona in zonas for name in names)
        //         return pmin, pmax
        todo!()
    }

    pub fn minmaxmeses(&self) -> (Componente, Componente) {
        //     def minmaxmeses(self):
        //         """Mínimo y máximo en demanda del edificio [kW/m²·año]

        //         Corresponde al mínimo y máximo de las zonas, ya que las plantas y edificio
        //         solamente tienen que tener valores más bajos por m².
        //         """
        //         zonas = self.zonas
        //         _min = min(min(zona.calefaccion_meses) for zona in zonas)
        //         _max = max(max(zona.refrigeracion_meses) for zona in zonas)
        //         return _min, _max
        todo!()
    }
}

#[derive(Clone, Debug, Default)]
/// Planta de LIDER
/// Contiene un conjunto de zonas.
pub struct PlantaLIDER {
    /// Nombre de la planta
    pub nombre: String,
    // TODO: multiplicador de planta??
    // TODO: comprobar si se usa el multiplicador de planta (no parece usarse en el .RES) y se traslada a las zonas
    // pub multiplicador: i32,
    /// Zonas de la planta
    pub zonas: Vec<String>,
}

impl PlantaLIDER {
    pub fn from_name(nombre: &str) -> Self {
        Self {
            nombre: nombre.to_string(),
            ..Default::default()
        }
    }

    /// Multiplicador de planta
    /// TODO: comprobar que es 1 y que el multiplicador se lleva a las zonas
    fn multiplicador(&self) -> i32 {
        1
    }

    /// Superficie de la planta en m² [m²]
    fn superficie(&self, ed: &EdificioLIDER) -> f32 {
        let zonedata = &ed.zonas;
        self.zonas
            .iter()
            .map(|zona| {
                let z = zonedata.get(zona).unwrap();
                z.superficie * z.multiplicador as f32
            })
            .sum()
    }

    /// Demanda anual de calefacción por m² [kWh/m²·año]
    fn calefaccion(&self, ed: &EdificioLIDER) -> f32 {
        self.calefaccion_meses(ed).iter().sum()
    }

    /// Demanda de calefacción por meses, agregando las de las zonas (en proporción a su superficie) kWh/m2
    fn calefaccion_meses(&self, ed: &EdificioLIDER) -> Vec<f32> {
        let zonedata = &ed.zonas;
        self.zonas
            .iter()
            .map(|z| {
                let z = zonedata.get(z).unwrap();
                z.calefaccion_meses
                    .iter()
                    .map(|dcal_i| dcal_i * z.superficie)
                    .collect()
            })
            .fold(vec![0.0f32; 12], |acc, x: Vec<f32>| {
                acc.iter().zip(x).map(|(a, b)| a + b).collect()
            })
            .iter()
            .map(|m| m / self.superficie(ed))
            .collect()
    }

    /// Demanda anual de refrigeración por m² [kWh/m²·año]
    fn refrigeracion(&self, ed: &EdificioLIDER) -> f32 {
        self.refrigeracion_meses(ed).iter().sum()
    }

    /// Demandas de refrigeración mensuales por m² [kWh/m²·mes]
    fn refrigeracion_meses(&self, ed: &EdificioLIDER) -> Vec<f32> {
        let zonedata = &ed.zonas;
        self.zonas
            .iter()
            .map(|z| {
                let z = zonedata.get(z).unwrap();
                z.refrigeracion_meses
                    .iter()
                    .map(|dref_i| dref_i * z.superficie)
                    .collect()
            })
            .fold(vec![0.0f32; 12], |acc, x: Vec<f32>| {
                acc.iter().zip(x).map(|(a, b)| a + b).collect()
            })
            .iter()
            .map(|m| m / self.superficie(ed))
            .collect()
    }

    /// Devuelve los flujos de calor de los grupos de las zonas de la planta
    /// Como los conceptos se dan en valor absoluto para cada zona, al convertirlo a datos de planta
    /// hay que ponderar por superficie (y tener en cuenta los multiplicadores)
    fn conceptos(&self, ed: &EdificioLIDER) -> Conceptos {
        let mut conceptos = Conceptos::default();
        let supplanta = self.superficie(&ed);

        if supplanta.abs() < f32::EPSILON {
            return conceptos;
        };

        for zona in &self.zonas {
            let z = ed.zonas.get(zona).unwrap();
            conceptos = conceptos + ((z.multiplicador as f32 * z.superficie) * z.conceptos);
        }
        conceptos * (1.0 / supplanta)
    }
}

#[derive(Clone, Debug, Default)]
pub struct ZonaLIDER {
    /// Nombre de la zona
    pub nombre: String,
    /// Nombre de la planta a la que pertenece la zona
    pub planta: String,
    /// Superficie de la zona [m²]
    pub superficie: f32,
    /// Número de zonas iguales en la planta
    pub multiplicador: i32,
    /// Demanda anual de calefacción de la zona [kWh/m²/año]
    pub calefaccion: f32,
    /// Demanda anual de refrigeración de la zona [kWh/m²/año]
    pub refrigeracion: f32,
    /// Demanda mensual de calefacción de la zona [kWh/m²/mes]
    pub calefaccion_meses: Vec<f32>,
    /// Demanda mensual de refrigeración de la zona [kWh/m²/mes]
    pub refrigeracion_meses: Vec<f32>,
    /// Flujos de calor por grupo (Paredes exteriores, Cubiertas...) [kWh/año]
    /// (e.g. "'Paredes Exteriores': (0.0, 1.2, 1.2, 0.0, -1.0, -1.0)")
    pub conceptos: Conceptos, //Vec<Componente>,
    /// Flujos de calor por componente (elementos constructivos) [kWh/año]
    pub componentes: Vec<Componente>,
}

impl ZonaLIDER {
    /// Crea nueva zona a partir de datos básicos
    pub fn from_name(nombre: String) -> Self {
        Self {
            nombre,
            planta: String::new(),
            superficie: 0.0,
            multiplicador: 1,
            calefaccion: 0.0,
            refrigeracion: 0.0,
            calefaccion_meses: vec![0.0; 12],
            refrigeracion_meses: vec![0.0; 12],
            conceptos: Conceptos::default(),
            componentes: Vec::new(),
        }
    }
}

// ----------------------------------------------------------------------------------------

/// Flujos a través de un elemento
#[derive(Debug, Copy, Clone, PartialEq, Default)]
pub struct Flujos {
    /// Flujo positivo (ganancias) de energía en temporada de calefacción [kWh/año]
    pub calpos: f32,
    /// Flujo negativo (pérdidas) de energía en temporada de calefacción [kWh/año]
    pub calneg: f32,
    /// Flujo neto de energía en temporada de calefacción [kWh/año] (ganancias - pérdidas)
    pub calnet: f32,
    /// Flujo positivo (ganancias) de energía en temporada de refrigeración [kWh/año]
    pub refpos: f32,
    /// Flujo negativo (pérdidas) de energía en temporada de refrigeración [kWh/año]
    pub refneg: f32,
    /// Flujo neto de energía en temporada de refrigeración [kWh/año] (ganancias - pérdidas)
    pub refnet: f32,
}

impl std::str::FromStr for Flujos {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let v: Vec<f32> = s
            .split(',')
            .map(|v| v.trim().parse::<f32>())
            .collect::<Result<Vec<f32>, _>>()?;
        if v.len() != 6 {
            return Err(format!("Formato de flujos erróneo: {}", s))?;
        };
        Ok(Self {
            calpos: v[0],
            calneg: v[1],
            calnet: v[2],
            refpos: v[3],
            refneg: v[4],
            refnet: v[5],
        })
    }
}

impl Add<&Flujos> for &Flujos {
    type Output = Flujos;

    fn add(self, other: &Flujos) -> Self::Output {
        Self::Output {
            calpos: self.calpos + other.calpos,
            calneg: self.calneg + other.calneg,
            calnet: self.calnet + other.calnet,
            refpos: self.refpos + other.refpos,
            refneg: self.refneg + other.refneg,
            refnet: self.refnet + other.refnet,
        }
    }
}

impl Mul<f32> for &Flujos {
    type Output = Flujos;

    fn mul(self, other: f32) -> Self::Output {
        Self::Output {
            calpos: self.calpos * other,
            calneg: self.calneg * other,
            calnet: self.calnet * other,
            refpos: self.refpos * other,
            refneg: self.refneg * other,
            refnet: self.refnet * other,
        }
    }
}

/// Conceptos de agrupación de flujos
#[derive(Debug, Copy, Clone, PartialEq, Default)]
pub struct Conceptos {
    /// Paredes exteriores
    pub pext: Flujos,
    /// Cubiertas
    pub cub: Flujos,
    /// Suelos
    pub suelos: Flujos,
    /// Puentes térmicos
    pub pts: Flujos,
    /// Huecos, transmisión solar
    pub huecos_solar: Flujos,
    /// Huecos, transmisión
    pub huecos_trans: Flujos,
    /// Fuentes internas
    pub fint: Flujos,
    /// Ventilación e infiltraciones
    pub vent: Flujos,
    /// Total conceptos
    pub total: Flujos,
}

impl Conceptos {
    /// Parsea concepto desde lista de 6 líneas de componentes:
    ///     concepto, calpos, calneg, calnet, refpos, refneg, refnet
    pub fn from_vec(vec: Vec<&str>) -> Result<Self, Error> {
        if vec.len() != 9 {
            return Err(format!("Lista de conceptos de tamaño inesperado {:?}", vec))?;
        }
        let lst: Vec<(&str, Flujos)> = vec
            .iter()
            .map(|l| {
                let mut vals = l.splitn(2, ',').map(str::trim);
                let name = vals.next();
                let flujos = vals.next().map(str::parse::<Flujos>);
                match (name, flujos) {
                    (Some(name), Some(Ok(flujos))) => Ok((name.trim_matches('\"'), flujos)),
                    _ => Err(format!(
                        "Error en formato de números de concepto: {:?}",
                        vec
                    )),
                }
            })
            .collect::<Result<_, _>>()?;
        let mut res = Self::default();
        for (name, flujos) in lst {
            match name {
                "Paredes Exteriores" => res.pext = flujos,
                "Cubiertas" => res.cub = flujos,
                "Suelos" => res.suelos = flujos,
                "Puentes Térmicos" => res.pts = flujos,
                "Solar Ventanas" => res.huecos_solar = flujos,
                "Transmisión Ventanas" => res.huecos_trans = flujos,
                "Fuentes Internas" => res.fint = flujos,
                "Infiltración" => res.vent = flujos,
                "TOTAL" => res.total = flujos,
                _ => Err(format!("Error al procesar concepto {}", name))?,
            }
        }
        Ok(res)
    }
}

// Ver https://stackoverflow.com/questions/28005134/how-do-i-implement-the-add-trait-for-a-reference-to-a-struct
// para la implementación completa de operaciones

impl Add<&Conceptos> for &Conceptos {
    type Output = Conceptos;

    fn add(self, other: &Conceptos) -> Self::Output {
        Self::Output {
            pext: &self.pext + &other.pext,
            cub: &self.cub + &other.cub,
            suelos: &self.suelos + &other.suelos,
            pts: &self.pts + &other.pts,
            huecos_solar: &self.huecos_solar + &other.huecos_solar,
            huecos_trans: &self.huecos_trans + &other.huecos_trans,
            fint: &self.fint + &other.fint,
            vent: &self.vent + &other.vent,
            total: &self.total + &other.total,
        }
    }
}

impl Add<Conceptos> for Conceptos {
    type Output = Conceptos;

    fn add(self, other: Conceptos) -> Self::Output {
        &self + &other
    }
}

impl Add<&Conceptos> for Conceptos {
    type Output = Conceptos;

    fn add(self, other: &Conceptos) -> Self::Output {
        &self + other
    }
}

impl Add<Conceptos> for &Conceptos {
    type Output = Conceptos;

    fn add(self, other: Conceptos) -> Self::Output {
        self + &other
    }
}

impl Mul<f32> for &Conceptos {
    type Output = Conceptos;

    fn mul(self, other: f32) -> Self::Output {
        Self::Output {
            pext: &self.pext * other,
            cub: &self.cub * other,
            suelos: &self.suelos * other,
            pts: &self.pts * other,
            huecos_solar: &self.huecos_solar * other,
            huecos_trans: &self.huecos_trans * other,
            fint: &self.fint * other,
            vent: &self.vent * other,
            total: &self.total * other,
        }
    }
}

impl Mul<f32> for Conceptos {
    type Output = Conceptos;

    fn mul(self, other: f32) -> Self::Output {
        &self * other
    }
}

impl Mul<&Conceptos> for f32 {
    type Output = Conceptos;

    fn mul(self, other: &Conceptos) -> Self::Output {
        other * self
    }
}

impl Mul<Conceptos> for f32 {
    type Output = Conceptos;

    fn mul(self, other: Conceptos) -> Self::Output {
        &other * self
    }
}

#[derive(Debug, Clone, Default)]
/// Componente de demanda de LIDER
/// Puede usarse para definir el comportamiento de elementos constructivos o grupos de demanda
pub struct Componente {
    /// Nombre del componente
    pub nombre: String,
    /// Flujos de energía de componente [kWh/año]
    /// - positivo (ganancias) de energía en temporada de calefacción [kWh/año]
    /// - negativo (pérdidas) de energía en temporada de calefacción [kWh/año]
    /// - neto de energía en temporada de calefacción [kWh/año] (ganancias - pérdidas)
    /// - positivo (ganancias) de energía en temporada de refrigeración [kWh/año]
    /// - negativo (pérdidas) de energía en temporada de refrigeración [kWh/año]
    /// - neto de energía en temporada de refrigeración [kWh/año] (ganancias - pérdidas)
    pub flujos: Flujos,
}

impl std::str::FromStr for Componente {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut data = s.splitn(2, ',').map(str::trim);
        let nombre = data.next().map(|s| s.trim_matches('\"').to_string());
        let flujos = data.next().map(|v| v.parse::<Flujos>());
        match (nombre, flujos) {
            (Some(nombre), Some(Ok(flujos))) => Ok(Self { nombre, flujos }),
            _ => Err(format!("Formato de componente erróneo: {}", s))?,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_components() {
        let c1 = Componente {
            nombre: "uno".to_string(),
            flujos: Flujos {
                calpos: 1.0,
                calneg: 2.0,
                calnet: 3.0,
                refpos: 4.0,
                refneg: 5.0,
                refnet: 6.0,
            },
        };
        let c2 = Componente {
            nombre: "dos".to_string(),
            flujos: Flujos {
                calpos: 1.0,
                calneg: 2.0,
                calnet: 3.0,
                refpos: 4.0,
                refneg: 5.0,
                refnet: 6.0,
            },
        };
    }
}
