#[allow(unused_imports)]
use std::env;
#[allow(unused_imports)]
use std::fs;
use std::io::BufReader;

use clap::Command;
use clap::arg;
use flate2::read::ZlibDecoder;

#[allow(dead_code)]
fn deflat_file(path: &str) -> std::io::Result<()> {
    let file = fs::File::open(path)?;
    let mut out = std::io::stdout();
    let mut reader = BufReader::new(file);
    let mut g = ZlibDecoder::new(&mut reader);
    std::io::copy(&mut g, &mut out)?;
    Ok(())
}

fn main() {

    let mut appcmd = Command::new("mygit")
        .subcommand(Command::new("init").about("init the git directory"))
        .subcommand(Command::new("cat-file").about("cat object with hash")
                    .arg(arg!(-p <HASH> "cat object with hash").required(true)));

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
            let path = format!(".git/objects/{}/{}", &hash[..2], &hash[2..]);
            deflat_file(&path).unwrap();
        }
        _ => {
            println!("No subcommand was used");
            appcmd.print_help().unwrap();
        }
    }

}
