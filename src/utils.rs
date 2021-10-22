/* -*- coding: utf-8 -*-

Copyright (c) 2018 Rafael Villar Burke <pachi@ietcc.csic.es>

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in
all copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
*/

// Utilidades varias

use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

use encoding::all::ISO_8859_1;
use encoding::{DecoderTrap, Encoding};

pub type Error = Box<dyn std::error::Error + 'static>;

/// Comprueba el directorio de ejecución, para detectar modo de desarrollo
pub fn check_current_dir() {
    // Cambiar el directorio actual si se arranca en directorio de proyecto para poder testear
    let cargo_manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let mut current_dir = std::env::current_dir().expect("Cannot read current dir");

    // println!("Ruta de cargo manifest: {}", cargo_manifest_dir.display());
    // println!(
    //     "Ruta del ejecutable: {}",
    //     std::env::current_exe().unwrap().display()
    // );
    // println!("Directorio actual inicial: {}", current_dir.display());

    // Si estamos arrancando en modo desarrollador cambiamos al subdirectorio ./src
    if current_dir == cargo_manifest_dir {
        current_dir.push("./src");
        std::env::set_current_dir(&current_dir).expect("Couldn't change current dir");
        // println!("Directorio actual final: {}", &current_dir.display());
    };
}

/// Lee a una cadena un archivo en latin1
pub fn read_latin1_file<T: AsRef<Path>>(path: T) -> Result<String, Error> {
    let buf = {
        let mut buf = Vec::new();
        File::open(path.as_ref())?
            .read_to_end(&mut buf)
            .map_err(|_| "No se ha podido leer el archivo")?;
        buf
    };

    match ISO_8859_1.decode(&buf, DecoderTrap::Replace) {
        Ok(utf8buf) => Ok(utf8buf),
        _ => return Err(format!(
            "Error de codificación del archivo {}",
            path.as_ref().display()
        ).into()),
    }
}
