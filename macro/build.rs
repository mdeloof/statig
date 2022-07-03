use std::{
    fs::File,
    io::{BufRead, BufReader, BufWriter, Write},
};

fn main() {
    // Minify script.js for embedding in html markdown
    let script = File::open("./src/doc_templates/script.js").expect("could not find script");
    let script = BufReader::new(script);

    let min_script =
        File::create("./src/doc_templates/min_script.js").expect("could not create script");
    let mut min_script = BufWriter::new(min_script);

    for line in script.lines() {
        let line = line.expect("could not read file");

        let code = match line.split_once("//") {
            Some((code, _)) => code,
            None => line.as_str(),
        };

        min_script.write_all(code.trim().as_bytes()).unwrap();
    }
}
