use std::collections::HashMap;
use std::error::Error;
use std::fs::{self, File};
use std::io::Read;
use std::path::PathBuf;

use handlebars::Handlebars;
use serde::{Deserialize, Serialize};
use toml;

#[derive(Debug, Deserialize, Serialize)]
struct Variant {
    variant: String,
    name: String,
    number: String,
    description: String,
}

macro_rules! regfile {
    ($reg:ident, $name:expr) => {
        let mut pb = PathBuf::from("templates");
        pb.push($name);
        $reg.register_template_file($name, &pb)?;
    };
}

fn main() -> Result<(), Box<Error>> {
    let data = read_data()?;
    let mut reg = Handlebars::new();

    let files = &["Makefile", "DESCR", "PLIST"];

    for file in files {
        regfile!(reg, file);
    }

    for (key, val) in data.iter() {
        for file in files {
            let mut path = PathBuf::from("..");
            path.push(key);
            if !path.is_dir() {
                fs::create_dir(&path)?;
            }
            path.push(file);
            let output = File::create(&path)?;
            reg.render_to_write(file, val, output)?;
            println!("Rendered {}", path.display());
        }
    }
    Ok(())
}

fn read_data() -> Result<HashMap<String, Variant>, Box<Error>> {
    let mut f = File::open("data.toml")?;
    let mut s = String::new();

    f.read_to_string(&mut s)?;
    Ok(toml::from_str(&s)?)
}
