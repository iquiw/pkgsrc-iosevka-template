use std::collections::HashMap;
use std::error::Error;
use std::fs::{self, File};
use std::io::Read;
use std::path::PathBuf;
use std::process::Command;

use handlebars::Handlebars;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
struct Variant {
    variant: String,
    name: String,
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
    eprintln!("usage: iosevka-template (make|pkglint|render)");
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = std::env::args();

    if let Some(cmd) = args.nth(1) {
        match cmd.as_ref() {
            "make" => make(args.collect()),
            "pkglint" => pkglint(),
            "render" => render(),
            _ => {
                usage();
                Ok(())
            }
        }
    } else {
        usage();
        Ok(())
    }
}

fn make(args: Vec<String>) -> Result<(), Box<dyn Error>> {
    let data = read_data()?;
    let mut cmd = Command::new("make");
    let cmd_str = format!("make {}", args.join(" "));
    cmd.args(args);
    for (key, _) in data.iter() {
        let path_dir = get_package_dir(key)?;
        let exit = cmd.current_dir(&path_dir).spawn()?.wait()?;
        println!("{}: {} exited with {}", key, cmd_str, exit);
    }
    Ok(())
}

fn pkglint() -> Result<(), Box<dyn Error>> {
    let data = read_data()?;
    let mut cmd = Command::new("pkglint");
    cmd.arg("-Call").arg("-Wall");
    for (key, _) in data.iter() {
        let path_dir = get_package_dir(key)?;
        eprint!("{}: ", key);
        let _ = cmd.current_dir(&path_dir).spawn()?.wait()?;
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
        let path_dir = get_package_dir(key)?;
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
