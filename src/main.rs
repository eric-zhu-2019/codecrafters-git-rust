#[allow(unused_imports)]
use std::env;
#[allow(unused_imports)]
use std::fs::{self, File};
use std::io::{BufReader, Stdout};
use std::io::{Write, Read};

use clap::ArgAction;
use clap::Command;
use clap::arg;
use flate2::read::ZlibDecoder;
type ZReader = BufReader<ZlibDecoder<File>>;

fn read_until(reader: &mut ZReader, delim: u8) -> std::io::Result<Vec<u8>> {
    let mut buf = Vec::new();
    let mut b = [0; 1];
    loop {
        reader.read(&mut b)?;
        if b[0] == delim {
            break;
        }
        buf.push(b[0]);
    }
    Ok(buf)
}

#[allow(dead_code)]
fn deflat_file(path: &str, out: &mut Stdout, _pretty: bool) -> std::io::Result<()> {
    
    let file = fs::File::open(path)?;
    let g = ZlibDecoder::new(file);

    let mut zreader = BufReader::new(g);
    let header = read_until(&mut zreader, b'\0')?;
    match String::from_utf8_lossy(&header[..]).split_once(" ") {
        Some(("blob", size)) => {
            let mut blob = vec![0; size.trim().parse::<usize>().unwrap()];
            let _n = zreader.read(&mut blob)?;
            out.write_all(&mut blob).unwrap();
        }
        _ => panic!("not a blob"),

    }
    Ok(())
}

fn main() {

    let mut appcmd = Command::new("mygit")
        .subcommand(Command::new("init").about("init the git directory"))
        .subcommand(Command::new("cat-file").about("cat object with hash")
                    .arg(arg!(pretty: -p "pretty print").required(false).action(ArgAction::SetTrue))
                    .arg(arg!(<HASH> "cat object with hash").required(true)));

    let cmds = appcmd.clone().get_matches();

    match cmds.subcommand() {
        Some(("init", _)) => { 
            fs::create_dir(".git").unwrap();
            fs::create_dir(".git/objects").unwrap();
            fs::create_dir(".git/refs").unwrap();
            fs::write(".git/HEAD", "ref: refs/heads/master\n").unwrap();
            println!("Initialized git directory");
        }
        Some(("cat-file", args)) => {
            let hash = args.get_one::<String>("HASH").unwrap();
            let pretty = args.get_flag("pretty");
            let path = format!(".git/objects/{}/{}", &hash[..2], &hash[2..]);
            let mut out = std::io::stdout();
            deflat_file(&path, &mut out, pretty).unwrap();
        }
        _ => {
            println!("No subcommand was used");
            appcmd.print_help().unwrap();
        }
    }

}
