//! Tipos para la representación de archivos de resultados de LIDER
//! - EdificioLIDER
//! - PlantaLIDER
//! - ZonaLIDER
//! - Componente

use crate::parsers::resqueries::ResQueries;
use crate::utils::Error;
use std::{collections::HashMap, ops::Add};

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
    pub grupos: Vec<Componente>,
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
            grupos: Vec::new(),
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

// TODO: convertir componente a nombre + Flujos

#[derive(Debug, Clone, Default)]
/// Componente de demanda de LIDER
/// Puede usarse para definir el comportamiento de elementos constructivos o grupos de demanda
pub struct Componente {
    /// Nombre del componente
    pub nombre: String,
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

/// Componente del edificio en LIDER
///
///     nombre - Nombre del componente / elemento
///     calpos - Flujo positivo de energía en temporada de calefacción [kWh/año]
///     calneg - Flujo negativo de energía en temporada de calefacción [kWh/año]
///     calnet - Flujo neto de energía en temporada de calefacción [kWh/año]
///     refpos - Flujo positivo de energía en temporada de refrigeración [kWh/año]
///     refneg - Flujo negativo de energía en temporada de refrigeración [kWh/año]
///     refnet - Flujo neto de energía en temporada de refrigeración [kWh/año]
impl Componente {
    /// Constructor de nuevo componente / elemento a partir de sus datos
    pub fn new(
        nombre: String,
        calpos: f32,
        calneg: f32,
        calnet: f32,
        refpos: f32,
        refneg: f32,
        refnet: f32,
    ) -> Self {
        Self {
            nombre,
            calpos,
            calneg,
            calnet,
            refpos,
            refneg,
            refnet,
        }
    }

    /// Devuelve valores de flujos y nombre como listas
    /// TODO: ver dónde se usa y si podemos elminarlo
    pub fn demandas(&self) -> ComponentesList {
        ComponentesList {
            nombre: vec![self.nombre.clone()],
            calpos: vec![self.calpos],
            calneg: vec![self.calneg],
            calnet: vec![self.calnet],
            refpos: vec![self.refpos],
            refneg: vec![self.refneg],
            refnet: vec![self.refnet],
        }
    }
}

impl std::str::FromStr for Componente {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let data: Vec<&str> = s.split(',').map(str::trim).collect();
        if data.len() != 7 {
            return Err(format!("Formato de componente erróneo: {}", s))?;
        };
        let nombre = data[0].trim_matches('\"').to_string();
        let v = data[1..7]
            .iter()
            .map(|v| v.parse::<f32>())
            .collect::<Result<Vec<f32>, _>>()?;
        Ok(Self {
            nombre,
            calpos: v[0],
            calneg: v[1],
            calnet: v[2],
            refpos: v[3],
            refneg: v[4],
            refnet: v[5],
        })
    }
}

impl Add<&Componente> for &Componente {
    type Output = ComponentesList;

    fn add(self, other: &Componente) -> Self::Output {
        Self::Output {
            nombre: vec![self.nombre.clone(), other.nombre.clone()],
            calpos: vec![self.calpos, other.calpos],
            calneg: vec![self.calneg, other.calneg],
            calnet: vec![self.calnet, other.calnet],
            refpos: vec![self.refpos, other.refpos],
            refneg: vec![self.refneg, other.refneg],
            refnet: vec![self.refnet, other.refnet],
        }
    }
}

/// Lista de nombres y flujos de calor para varios elementos o componentes
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ComponentesList {
    /// Nombres de los elementos / componentes
    nombre: Vec<String>,
    /// Flujos positivos (ganancias) de energía en temporada de calefacción [kWh/año]
    calpos: Vec<f32>,
    /// Flujos negativos (pérdidas) de energía en temporada de calefacción [kWh/año]
    calneg: Vec<f32>,
    /// Flujos netos de energía en temporada de calefacción [kWh/año] (ganancias - pérdidas)
    calnet: Vec<f32>,
    /// Flujos positivos (ganancias) de energía en temporada de refrigeración [kWh/año]
    refpos: Vec<f32>,
    /// Flujos negativos (pérdidas) de energía en temporada de refrigeración [kWh/año]
    refneg: Vec<f32>,
    /// Flujos netos de energía en temporada de refrigeración [kWh/año] (ganancias - pérdidas)
    refnet: Vec<f32>,
}

impl Add<&Componente> for ComponentesList {
    type Output = Self;

    fn add(self, other: &Componente) -> Self::Output {
        let mut res = self.clone();
        res.nombre.push(other.nombre.clone());
        res.calpos.push(other.calpos);
        res.calneg.push(other.calneg);
        res.calnet.push(other.calnet);
        res.refpos.push(other.refpos);
        res.refneg.push(other.refneg);
        res.refnet.push(other.refnet);
        res
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_components() {
        let c1 = Componente {
            nombre: "uno".to_string(),
            calpos: 1.0,
            calneg: 2.0,
            calnet: 3.0,
            refpos: 4.0,
            refneg: 5.0,
            refnet: 6.0,
        };
        let c2 = Componente {
            nombre: "dos".to_string(),
            calpos: 1.0,
            calneg: 2.0,
            calnet: 3.0,
            refpos: 4.0,
            refneg: 5.0,
            refnet: 6.0,
        };
        assert_eq!(
            ComponentesList {
                nombre: vec!["uno".to_string(), "dos".to_string()],
                calpos: vec![1.0, 1.0],
                calneg: vec![2.0, 2.0],
                calnet: vec![3.0, 3.0],
                refpos: vec![4.0, 4.0],
                refneg: vec![5.0, 5.0],
                refnet: vec![6.0, 6.0]
            },
            &c1 + &c2
        )
    }
}
