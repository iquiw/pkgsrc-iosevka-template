use std::collections::HashMap;
use std::error::Error;
use std::fs::{self, File};
use std::io::Read;
use std::path::PathBuf;
use std::process::Command;

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

fn usage() {
    eprintln!("usage: iosevka-template (render|makesum)");
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = std::env::args();

    if let Some(cmd) = args.nth(1) {
        match cmd.as_ref() {
            "makesum" => makesum(),
            "render" => render(),
            _ => Ok(usage()),
        }
    } else {
        Ok(usage())
    }
}

fn makesum() -> Result<(), Box<dyn Error>> {
    let data = read_data()?;
    for (key, _) in data.iter() {
        let path_dir = get_package_dir(&key)?;
        let mut cmd = Command::new("make")
            .arg("makesum")
            .current_dir(&path_dir)
            .spawn()?;
        let exit = cmd.wait()?;
        println!("{}: makesum exited with {}", key, exit);
    }
    Ok(())
}

fn render() -> Result<(), Box<dyn Error>> {
    let data = read_data()?;
    let mut reg = Handlebars::new();

    let files = &["Makefile", "DESCR", "PLIST"];

    for file in files {
        regfile!(reg, file);
    }

    for (key, val) in data.iter() {
        let path_dir = get_package_dir(&key)?;
        for file in files {
            let mut path = path_dir.clone();
            path.push(file);
            let output = File::create(&path)?;
            reg.render_to_write(file, val, output)?;
            println!("Rendered {}", path.display());
        }
    }
    Ok(())
}

fn read_data() -> Result<HashMap<String, Variant>, Box<dyn Error>> {
    let mut f = File::open("data.toml")?;
    let mut s = String::new();

    f.read_to_string(&mut s)?;
    Ok(toml::from_str(&s)?)
}

fn get_package_dir(name: &str) -> Result<PathBuf, Box<dyn Error>> {
    let mut path_dir = PathBuf::from("..");
    path_dir.push(name);
    if !path_dir.is_dir() {
        fs::create_dir(&path_dir)?;
    }
    Ok(path_dir)
}
