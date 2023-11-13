#[allow(unused_imports)]
use std::env;
#[allow(unused_imports)]
use std::fs;
use std::io::Read;

use flate2::read::ZlibDecoder;


fn main() {

    // Uncomment this block to pass the first stage
    let args: Vec<String> = env::args().collect();
    if args[1] == "init" {
        fs::create_dir(".git").unwrap();
        fs::create_dir(".git/objects").unwrap();
        fs::create_dir(".git/refs").unwrap();
        fs::write(".git/HEAD", "ref: refs/heads/master\n").unwrap();
        println!("Initialized git directory")
    } else if args[1] == "cat-file" {
        if args[2] == "-p" {
            let hash = &args[3];
            let path = format!(".git/objects/{}/{}", &hash[..2], &hash[2..]);
            let mut file = fs::File::open(path).unwrap();

            let mut s = String::new();
            let mut d = vec![0; 4096];
            loop {
                match file.read(&mut d) {
                    Ok(0) => break,
                    Ok(n) => {
                        let mut g = ZlibDecoder::new(&d[..n]);
                        let _ = g.read_to_string(&mut s);
                        print!("{}", s);
                    }
                    Err(err) => panic!("Error: {}", err),
                }
            }
        } else {
            println!("unknown option: {}", args[2])
        }
    }
}
