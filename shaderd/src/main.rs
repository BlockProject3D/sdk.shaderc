use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use bpx::sd::formatting::{Format, IndentType};
use bpx::shader::ShaderPack;
use bpx::shader::symbol::{FLAG_ASSEMBLY, FLAG_DOMAIN_STAGE, FLAG_EXTENDED_DATA, FLAG_EXTERNAL, FLAG_GEOMETRY_STAGE, FLAG_HULL_STAGE, FLAG_INTERNAL, FLAG_PIXEL_STAGE, FLAG_REGISTER, FLAG_VERTEX_STAGE};
use clap::{Arg, Command};

enum Error {
    Io(std::io::Error),
    Bpx(bpx::shader::error::Error)
}

fn disassemble(path: &Path, table: bool) -> Result<(), Error>
{
    let file = File::open(path).map_err(Error::Io)?;
    let shader = ShaderPack::open(BufReader::new(file)).map_err(Error::Bpx)?;
    println!("Linked assembly: {:#X}", shader.get_settings().assembly_hash);
    let symbols = shader.symbols().map_err(Error::Bpx)?;
    let shaders = shader.shaders();
    println!("Number of symbols: {}", symbols.len());
    println!("Target API: {:?}", shader.get_settings().target);
    println!("Shader type: {:?}", shader.get_settings().ty);
    println!("Number of shader stages: {}", shaders.len());
    println!();
    if table {
        println!("Symbol table:");
        for sym in &symbols {
            let name = symbols.load_name(sym).map_err(Error::Bpx)?;
            println!("    * {}: {:?}", name, sym.ty);
        }
    }
    Ok(())
}

fn flags_to_string(flags: u16) -> String {
    let mut str = String::new();
    if flags & FLAG_REGISTER != 0 {
        str += "Register | "
    }
    if flags & FLAG_EXTENDED_DATA != 0 {
        str += "ExtendedData | "
    }
    if flags & FLAG_ASSEMBLY != 0 {
        str += "Assembly | "
    }
    if flags & FLAG_INTERNAL != 0 {
        str += "Internal | "
    }
    if flags & FLAG_EXTERNAL != 0 {
        str += "External | "
    }
    if flags & FLAG_DOMAIN_STAGE != 0 {
        str += "DomainStage | "
    }
    if flags & FLAG_VERTEX_STAGE != 0 {
        str += "VertexStage | "
    }
    if flags & FLAG_HULL_STAGE != 0 {
        str += "HullStage | "
    }
    if flags & FLAG_PIXEL_STAGE != 0 {
        str += "PixelStage | "
    }
    if flags & FLAG_GEOMETRY_STAGE != 0 {
        str += "GeometryStage | "
    }
    if !str.is_empty() {
        str.truncate(str.len() - 3);
    }
    str
}

fn show_symbol(path: &Path, name: &str) -> Result<(), Error>
{
    let file = File::open(path).map_err(Error::Io)?;
    let shader = ShaderPack::open(BufReader::new(file)).map_err(Error::Bpx)?;
    let symbols = shader.symbols().map_err(Error::Bpx)?;
    for sym in &symbols {
        if symbols.load_name(sym).map_err(Error::Bpx)? == name {
            println!("==> Basic <==");
            println!("Name: {}", symbols.load_name(sym).map_err(Error::Bpx)?);
            println!("Type: {:?}", sym.ty);
            if sym.flags & FLAG_REGISTER != 0 {
                println!("Register: {}", sym.register)
            }
            println!("Flags: {}", flags_to_string(sym.flags));
            if sym.flags & FLAG_EXTENDED_DATA != 0 {
                println!();
                println!("==> Extended data <==");
                let obj = symbols.load_extended_data(sym).map_err(Error::Bpx)?;
                println!("{}", obj.format(IndentType::Spaces, 4));
            }
            return Ok(())
        }
    }
    Ok(())
}

fn main() {
    let matches = Command::new("shaderd")
        .author("BlockProject 3D")
        .about("BlockProject 3D SDK - Shader Disassembler")
        .version("1.0.0")
        .args([
            Arg::new("dump").short('d').long("dump")
                .help("Dump all shader stage code to separate files"),
            Arg::new("symbol").short('s').long("symbol").takes_value(true)
                .help("Show all data about a symbol"),
            Arg::new("table").short('t').long("table")
                .help("Show symbol table"),
            Arg::new("shader").takes_value(true).allow_invalid_utf8(true).required(true)
                .help("Shader pack file to disassemble")
        ]).get_matches();
    let path = matches.value_of_os("shader").map(Path::new).unwrap();
    let data = if let Some(name) = matches.value_of("symbol") {
        show_symbol(path, name)
    } else {
        disassemble(path, matches.is_present("table"))
    };
    if let Err(e) = data {
        match e {
            Error::Io(e) => eprintln!("An io error has occured: {}", e),
            Error::Bpx(e) => eprintln!("A BPX error has occured: {}", e)
        }
        std::process::exit(1);
    }
}
