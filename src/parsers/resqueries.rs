//! Traits para solicitar información de los objetos de LIDER

use super::types::{Componente, ComponentesList, EdificioLIDER, PlantaLIDER};
use std::collections::HashMap;

const GRUPOSLIDER: [&str; 9] = [
    "Paredes Exteriores",
    "Cubiertas",
    "Suelos",
    "Puentes Térmicos",
    "Solar Ventanas",
    "Transmisión Ventanas",
    "Fuentes Internas",
    "Ventilación más Infiltración",
    "TOTAL",
];

/// Información que puede obtenerse de los distintos tipos de elementos
pub trait ResQueries {
    fn multiplicador(&self) -> i32;
    fn superficie(&self, ed: &EdificioLIDER) -> f32;
    fn calefaccion(&self, ed: &EdificioLIDER) -> f32;
    fn refrigeracion(&self, ed: &EdificioLIDER) -> f32;
    fn calefaccion_meses(&self, ed: &EdificioLIDER) -> Vec<f32>;
    fn refrigeracion_meses(&self, ed: &EdificioLIDER) -> Vec<f32>;
    fn demandas(&self) -> ComponentesList;
    fn grupos(&self) -> Vec<Componente>;
    fn componentes(&self) -> Vec<Componente>;
}

impl ResQueries for EdificioLIDER {
    fn multiplicador(&self) -> i32 {
        1
    }
    fn superficie(&self, ed: &EdificioLIDER) -> f32 {
        self.superficie
    }
    fn calefaccion(&self, ed: &EdificioLIDER) -> f32 {
        self.calefaccion
    }
    fn refrigeracion(&self, ed: &EdificioLIDER) -> f32 {
        self.refrigeracion
    }
    fn calefaccion_meses(&self, ed: &EdificioLIDER) -> Vec<f32> {
        self.calefaccion_meses.clone()
    }
    fn refrigeracion_meses(&self, ed: &EdificioLIDER) -> Vec<f32> {
        self.refrigeracion_meses.clone()
    }
    fn componentes(&self) -> Vec<Componente> {
        todo!()
    }
    fn grupos(&self) -> Vec<Componente> {
        //     @cached_property
        //     def grupos(self):
        //         """Flujos de calor de los grupos, para el edificio [kW/m²·año]

        //         Devuelve un diccionario indexado por grupo (p.e. u'Paredes exteriores')
        //         que contiene una tupla con las demandas de cada grupo:
        //             (calefacción +, calefacción -, calefacción neta,
        //              refrigeración +, refrigeración -, refrigeración neta)
        //         """
        //         dic = OrderedDict()
        //         for grupo in self.gruposlider:
        //             params = [self[planta].superficie *
        //                       numpy.array(self[planta].grupos[grupo])
        //                       for planta in self]
        //             plist = [sum(lst) for lst in zip(*params)]
        //             dic[grupo] = numpy.array(plist) / self.superficie
        //         return dic
        todo!()
    }

    fn demandas(&self) -> ComponentesList {
        //     @cached_property
        //     def demandas(self):
        //         """Demandas del edificio por grupos [kW/m²·año]

        //         Devuelve un diccionario con seis tuplas de calefacción +, calefacción -,
        //         calefacción neta, refrigeración +, refrigeración -, refrigeración neta
        //         que contienen el valor correspondiente para cada grupo del edificio.

        //         El orden de los valores corresponde al de los grupos en el diccionario
        //         self.grupos.keys().
        //         """
        //         d = OrderedDict()
        //         (d['cal+'], d['cal-'], d['cal'],
        //          d['ref+'], d['ref-'], d['ref']) = zip(*self.grupos.values())
        //         d['grupos'] = self.grupos.keys() # mismos elementos que self.gruposlider
        //         return d
        todo!()
    }
}

impl ResQueries for PlantaLIDER {
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
    fn grupos(&self) -> Vec<Componente> {
        let map = HashMap::<&str, Componente>::new();
        let grupos = Vec::new();
        for g in GRUPOSLIDER.iter() {
            let nombre = g.to_string();
            // TODO: seguir
        }
        grupos
        //     @cached_property
        //     def grupos(self):
        //         """Flujos de calor de los grupos, para la planta [kW/m²·año]

        //         Devuelve un diccionario indexado por grupo (p.e. u'Paredes exteriores')
        //         que contiene una tupla con las demandas de cada grupo:
        //             (calefacción +, calefacción -, calefacción neta,
        //              refrigeración +, refrigeración -, refrigeración neta)
        //         """
        //         #TODO: El rendimiento de esta parte es crítico, y depende mucho
        //         #TODO: de la creación de numpy.arrays y la suma de valores
        //         superficieplanta = self.superficie
        //         dic = OrderedDict()
        //         for grupo in self.gruposlider:
        //             params = [self[zona].superficie *
        //                       self[zona].multiplicador *
        //                       numpy.array(self[zona].grupos[grupo].values)
        //                       for zona in self]
        //             # XXX: Se podría hacer con numpy sumando arrays (que lo hace columna a columna)
        //             plist = [sum(lst) for lst in zip(*params)]
        //             dic[grupo] = numpy.array(plist) / superficieplanta
        //         return dic
    }

    /// Demandas de la planta por grupos [kW/m²·año]
    ///
    /// Devuelve una lista de componentes seis tuplas de nombres, calefacción +, calefacción -,
    /// calefacción neta, refrigeración +, refrigeración -, refrigeración neta
    /// que contienen el valor correspondiente para cada grupo de la planta.
    /// Listas de nombres, y componentes de flujos
    fn demandas(&self) -> ComponentesList {
        self.grupos()
            .iter()
            .fold(ComponentesList::default(), |acc, x| acc + x)
    }

    // /// Flujos de calor por componente (Hueco H1, muro M1...) [kWh/m²año]
    // pub componentes: HashMap<String, Componente>,
    // TODO: ????
    fn componentes(&self) -> Vec<Componente> {
        todo!()
    }
}
