//! Analizador de archivos de resultados de LIDER

pub use crate::parsers::types::{EdificioLIDER, PlantaLIDER, ZonaLIDER};
use crate::utils::read_latin1_file;
use crate::utils::Error;
use std::str::Lines;
use std::{collections::HashMap, path::Path};

impl EdificioLIDER {
    pub fn from_file<S: AsRef<Path>>(path: S) -> Result<EdificioLIDER, Error> {
        eprintln!("Parsear resfile: {}", path.as_ref().display());

        let mut edificio = EdificioLIDER::default();
        let resdata = read_latin1_file(path)?;

        let mut lines = &mut resdata.lines();
        while let Some(line) = lines.next() {
            let line = line.trim();
            // Comentarios y líneas en blanco
            if line.starts_with("#") || line.is_empty() {
                continue;
            }
            // Plantas del edificio ----------------------------------
            else if line.starts_with("Numero de plantas") {
                find_plantas_y_zonas(&mut lines, &mut edificio)?;
            }
            // Resultados a nivel de edificio ------------------------
            else if line.starts_with("RESULTADOS A NIVEL EDIFICIO") {
                // Demandas del edificio --------------
                find_demandas_generales_edificio(&mut lines, &mut edificio)?;
                find_cal_mensual_edificio(&mut lines, &mut edificio)?;
                find_ref_mensual_edificio(&mut lines, &mut edificio)?;

                // Zonas del edificio --------------
                // Datos generales de zonas
                let zonelist = find_datos_generales_zonas(&mut lines, &mut edificio)?;
                // Calefacción mensual por zonas
                find_cal_mensual_zonas(&mut lines, &mut edificio, &zonelist)?;
                // Refrigeración mensual por zonas
                find_ref_mensual_zonas(&mut lines, &mut edificio, &zonelist)?;
            };
        }
        edificio.resdata = resdata;
        Ok(edificio)
    }
}

/// Localiza conjunto de plantas y zonas
fn find_plantas_y_zonas(lines: &mut Lines, edificio: &mut EdificioLIDER) -> Result<(), Error> {
    let numplantas: i32 = lines
        .next()
        .ok_or_else(|| "No se encuentra el número de plantas del edificio")?
        .parse()?;
    // XXX: no guardamos el número de plantas del edificio. Basta contarlas en la lista de plantas
    // edificio.numplantas = numplantas;

    let mut plantas = Vec::<PlantaLIDER>::with_capacity(numplantas as usize);
    let mut zonas = HashMap::<String, ZonaLIDER>::new();

    for _ in 0..numplantas {
        let pname = lines
            .find(|l| l.starts_with("\"P"))
            .ok_or_else(|| "No se encuentran todas las plantas del edificio")?
            .trim()
            .trim_matches('\"');
        let mut planta = PlantaLIDER::from_name(pname);

        // Parsing de zonas de la planta ------------------------
        lines.find(|l| l.starts_with("Numero de zonas"));
        let numzonas: i32 = lines
            .next()
            .ok_or_else(|| format!("No se encuentra el número de zonas de la planta {}", pname))?
            .parse()?;
        planta.zonas = Vec::<String>::with_capacity(numzonas as usize);
        for i in 0..numzonas {
            let (_znumero, znombre) = match lines
                .find(|l| l.starts_with("Zona "))
                .ok_or_else(|| format!("No se encuentra la zona {} de la planta {}", i, pname))?
                .split(',')
                .collect::<Vec<&str>>()
                .as_slice()
            {
                [num, name] => (
                    num, //num.trim_start_matches("Zona ").parse::<i32>()?,
                    name.trim().trim_matches('\"').to_string(),
                ),
                _ => Err(format!(
                    "Formato incorrecto de zona {} de la planta {}",
                    i, pname
                ))?,
            };
            let zsuperficie = lines
                .next()
                .ok_or_else(|| format!("No se encuentra la superficie de la zona {}", znombre))?
                .parse::<f32>()?;
            let mut zona = ZonaLIDER::from_name(znombre.to_string());
            // zona.numero = znumero; // No lo guardamos
            zona.planta = pname.to_string();
            zona.superficie = zsuperficie;

            // Parsing de grupos de demanda de la zona ----------------
            lines.find(|l| l.starts_with("Concepto, Cal_positivo"));
            // 9 grupos de demanda (Paredes Exteriores, Cubiertas, Suelos, ...)
            let mut grupos = Vec::with_capacity(9);
            for _ in 0..9 {
                let valores = lines.next().ok_or_else(|| {
                    format!(
                        "No se encuentran los grupos de demanda de la zona {}",
                        znombre
                    )
                })?;
                grupos.push(valores.parse()?)
            }
            zona.grupos = grupos;

            // Parsing de componentes de demanda de la zona ------------
            lines.find(|l| l.starts_with("Numero de Componentes"));
            let numcomponentes: i32 = lines
                .next()
                .ok_or_else(|| {
                    format!(
                        "No se encuentra el número de componentes de la zona {}",
                        znombre
                    )
                })?
                .parse()?;
            lines.find(|l| l.starts_with("Componente, Cal_positivo"));
            zona.componentes = Vec::with_capacity(numcomponentes as usize);
            for _ in 0..numcomponentes {
                let valores = lines.next().ok_or_else(|| {
                    format!(
                        "No se encuentran las demandas de componentes de la zona {}",
                        znombre
                    )
                })?;
                zona.componentes.push(valores.parse()?);
            }
            planta.zonas.push(zona.nombre.clone());
            zonas.insert(zona.nombre.clone(), zona);
        }
        plantas.push(planta);
    }
    edificio.plantas = plantas;
    edificio.zonas = zonas;

    Ok(())
}

/// Localiza demandas anuales y mensuales de calefacción y refrigeración del edificio
fn find_demandas_generales_edificio(
    lines: &mut Lines,
    edificio: &mut EdificioLIDER,
) -> Result<(), Error> {
    // Cal, ref, mensual
    lines.find(|l| {
        l.starts_with("Calefacción, Refrigeración anual") || l.starts_with("Calefacción anual")
    });
    match lines
        .next()
        .ok_or_else(|| "Formato incorrecto: datos generales")?
        .split(',')
        .map(|v| v.trim().parse::<f32>())
        .collect::<Result<Vec<f32>, _>>()?
        .as_slice()
    {
        [cal, refr] => {
            edificio.calefaccion = *cal;
            edificio.refrigeracion = *refr;
        }
        res => {
            return Err(format!(
                "Formato incorrecto de datos a nivel de edificio: {:?}",
                res
            ))?
        }
    };
    Ok(())
}

/// Localiza demandas mensuales de calefacción del edificio
fn find_cal_mensual_edificio(lines: &mut Lines, edificio: &mut EdificioLIDER) -> Result<(), Error> {
    // Cal meses
    lines.find(|l| l.starts_with("Calefacción mensual"));
    let cal_meses = lines
        .next()
        .ok_or_else(|| "Formato incorrecto: calefacción por meses")?
        .split(',')
        .map(|v| v.trim().parse::<f32>())
        .collect::<Result<Vec<f32>, _>>()?;
    edificio.calefaccion_meses = cal_meses;

    Ok(())
}

/// Localiza demandas mensuales de refrigeración del edificio
fn find_ref_mensual_edificio(lines: &mut Lines, edificio: &mut EdificioLIDER) -> Result<(), Error> {
    // Ref meses
    lines.find(|l| l.starts_with("Refrigeración mensual"));
    let ref_meses = lines
        .next()
        .ok_or_else(|| "Formato incorrecto: refrigeración por meses")?
        .split(',')
        .map(|v| v.trim().parse::<f32>())
        .collect::<Result<Vec<f32>, _>>()?;
    edificio.refrigeracion_meses = ref_meses;
    Ok(())
}

/// Localiza datos de refrigeración mensual de zonas
fn find_datos_generales_zonas(
    lines: &mut Lines,
    edificio: &mut EdificioLIDER,
) -> Result<Vec<String>, Error> {
    let mut zonelist = Vec::<String>::new();

    lines.find(|l| l.starts_with("Numero de zonas"));
    let numzonas = lines
        .next()
        .ok_or_else(|| "Formato incorrecto: zonas del edificio")?
        .parse::<i32>()?;
    // XXX: no guardamos el número de zonas del edificio. Es la suma de zonas de las plantas
    // edificio.numzonas = numzonas;

    // Recopilamos datos de zonas, y luego habrá que asignarlos a la ZonaLIDER correspondiente en las plantas
    lines.find(|l| l.starts_with("Nombre, m2, multiplicador"));

    for _ in 0..numzonas {
        let valores = lines
            .next()
            .ok_or_else(|| "Formato incorrecto: datos de zonas")?
            .split(',')
            .collect::<Vec<&str>>();
        if valores.len() != 5 {
            return Err("Número incorrecto de valores en datos de zonas")?;
        };
        let numbers = valores[1..]
            .iter()
            .map(|v| v.trim().parse::<f32>())
            .collect::<Result<Vec<f32>, _>>()?;
        let (nombre, superficie, multiplicador, calefaccion, refrigeracion) = (
            valores[0].trim().trim_matches('\"'),
            numbers[0],
            numbers[1],
            numbers[2],
            numbers[3],
        );
        edificio.zonas.entry(nombre.to_string()).and_modify(|zona| {
            zona.superficie = superficie;
            zona.multiplicador = multiplicador as i32;
            zona.calefaccion = calefaccion;
            zona.refrigeracion = refrigeracion;
        });
        zonelist.push(nombre.to_string());
    }

    // TOTAL
    match lines
        .find(|l| l.starts_with("TOTAL"))
        .ok_or_else(|| "Formato incorrectos: sin total de zonas")?
        .split(',')
        .collect::<Vec<_>>()
        .as_slice()
    {
        [_, superficie, _, _] => edificio.superficie = superficie.trim().parse::<f32>()?,
        res => return Err(format!("Formato incorrecto en total de zonas: {:?}", res))?,
    };
    Ok(zonelist)
}

/// Localiza datos de refrigeración mensual de zonas
fn find_cal_mensual_zonas(
    lines: &mut Lines,
    edificio: &mut EdificioLIDER,
    zonelist: &Vec<String>,
) -> Result<(), Error> {
    lines.find(|l| l.starts_with("Calefacción mensual por zonas"));
    for nombre in zonelist {
        let valores = lines
            .next()
            .ok_or_else(|| "Formato incorrecto: datos de calefacción mensual por zonas")?
            .split(',')
            .map(|v| v.trim().parse::<f32>())
            .collect::<Result<Vec<f32>, _>>()?;
        edificio
            .zonas
            .entry(nombre.clone())
            .and_modify(|e| e.calefaccion_meses = valores);
    }

    Ok(())
}

/// Localiza datos de refrigeración mensual de zonas
fn find_ref_mensual_zonas(
    lines: &mut Lines,
    edificio: &mut EdificioLIDER,
    zonelist: &Vec<String>,
) -> Result<(), Error> {
    lines.find(|l| l.starts_with("Refrigeración mensual por zonas"));
    for nombre in zonelist {
        let valores = lines
            .next()
            .ok_or_else(|| "Formato incorrecto: datos de refrigeración mensual por zonas")?
            .split(',')
            .map(|v| v.trim().parse::<f32>())
            .collect::<Result<Vec<f32>, _>>()?;
        edificio
            .zonas
            .entry(nombre.clone())
            .and_modify(|e| e.refrigeracion_meses = valores);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_res_testfile() {
        let mut testfile = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        testfile.push("./src/data/test.res");
        let res = EdificioLIDER::from_file(testfile).unwrap();
        assert_eq!(res.plantas.len(), 2);
        assert_eq!(res.plantas[0].zonas.len(), 4);
        assert_eq!(res.plantas[1].zonas.len(), 6);
        assert_eq!(1, 1);
    }
}
