#[allow(unused_imports)]
use std::env;
use std::fs::remove_file;
#[allow(unused_imports)]
use std::fs::{self, File};
use std::io::{BufReader, Stdout, copy};
use std::io::{Write, Read};
use std::path::{Path, PathBuf};

use anyhow::Result;
use clap::ArgAction;
use clap::Command;
use clap::arg;
use flate2::Compression;
use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use sha1::{Digest, Sha1};

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

fn init_git() -> Result<()> {
    fs::create_dir(".git")?;
    fs::create_dir(".git/objects")?;
    fs::create_dir(".git/refs")?;
    fs::write(".git/HEAD", "ref: refs/heads/master\n")?;
    println!("Initialized git directory");
    Ok(())
}

fn zlib_compress(src_file: &PathBuf, tgt_file: &PathBuf) -> Result<()> {
    let mut src_file = fs::File::open(&src_file)?;
    let mut zlib_compressor = ZlibEncoder::new(Vec::new(), Compression::default());
    let mut buf = [0; 1024*16];
    let mut tgt_file = fs::File::create(&tgt_file)?;
    loop {
        match src_file.read(&mut buf) {
            Ok(0) => {
                if let Ok(w) = zlib_compressor.finish() {
                    tgt_file.write_all(w.as_slice())?;
                }
                break;
            },
            Ok(n) => {
                zlib_compressor.write(&buf[..n])?;
            }
            Err(e) => {
                return Err(e.into());
            }
        }
    }
    
    Ok(())
}

fn concat_blob_header(srcfile: &str) -> Result<PathBuf> {
    let sz = PathBuf::from(srcfile).metadata().unwrap().len();
    let blob = format!("blob {}\0", sz);
    let tmpdir = std::env::temp_dir();
    let tmpfile = tmpdir.join("tmp-blob-file");
    let _ = remove_file(&tmpfile);
    let mut objfile = fs::OpenOptions::new()
        .append(true)
        .create(true)
        .open(&tmpfile)?;
    objfile.write_all(blob.as_bytes())?;
    let mut src_file = fs::File::open(&srcfile)?;
    copy(&mut src_file, &mut objfile)?;
    Ok(tmpfile)
}

fn hash_file(objfile: &PathBuf) -> Result<String> {
    let mut hasher = Sha1::new();
    let mut buf = [0; 1024*16];
    let mut file = fs::File::open(&objfile)?;

    loop {
        match file.read(&mut buf) {
            Ok(0) => {
                let k = hasher.finalize();
                let hex_string = hex::encode(k);
                return Ok(hex_string);
            },
            Ok(n) => {
                hasher.update(&buf[..n]);
            }
            Err(e) => {
                return Err(anyhow::anyhow!("Error reading file: {}, {:?}", objfile.file_name().unwrap().to_str().unwrap(), e.get_ref().unwrap()));
            }
        }
    }
}

// hash(blob <size of original file>\0<file content>)
// zlib-compress (bloc <size>\0<file content>)
fn hash_obj(filepath: &str, dowrite: bool) -> Result<()> {

    // concat "blob size\0" with file content
    if let Ok(tmpfile) = concat_blob_header(filepath) {
        if let Ok(hex_string) = hash_file(&tmpfile) {
            println!("{}", hex_string);
            if dowrite {
                let path = format!(".git/objects/{}/{}", &hex_string[..2], &hex_string[2..]);
                let path = Path::new(&path);
                if Path::exists(path) {
                    return Ok(());
                }
                fs::create_dir_all(format!(".git/objects/{}", &hex_string[..2]))?;
                
                zlib_compress(&tmpfile, &PathBuf::from(path))?;
            } 
            return Ok(());
        } else {
            remove_file(&tmpfile)?;
            return Err(anyhow::anyhow!("Error hashing file: {}", tmpfile.to_str().unwrap()));
        }
    } else {
        return Err(anyhow::anyhow!("Error concating blob header: {}", filepath));
    }
}

fn main() -> Result<()> {

    let mut appcmd = Command::new("mygit")
        .subcommand(Command::new("init").about("init the git directory"))
        .subcommand(Command::new("cat-file").about("cat object with hash")
                    .arg(arg!(pretty: -p "pretty print").required(false).action(ArgAction::SetTrue))
                    .arg(arg!(<HASH> "cat object with hash").required(true)))
        .subcommand(Command::new("hash-object").about("add a blob")
                    .arg(arg!(<FILE> "add a blob").required(true))
                    .arg(arg!(write: -w "write the object into the git database").required(false).action(ArgAction::SetTrue)));

    let cmds = appcmd.clone().get_matches();

    match cmds.subcommand() {
        Some(("init", _)) => { 
            init_git()?;
        }
        Some(("cat-file", args)) => {
            let hash = args.get_one::<String>("HASH").unwrap();
            let pretty = args.get_flag("pretty");
            let path = format!(".git/objects/{}/{}", &hash[..2], &hash[2..]);
            let mut out = std::io::stdout();
            deflat_file(&path, &mut out, pretty).unwrap();
        }
        Some(("hash-object", args)) => {
            let filepath = args.get_one::<String>("FILE").unwrap();
            let dowrite = args.get_flag("write");
            hash_obj(filepath, dowrite)?;
        }
        _ => {
            println!("No subcommand was used");
            appcmd.print_help().unwrap();
        }
    }
    Ok(())
}
