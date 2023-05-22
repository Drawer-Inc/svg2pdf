use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process;

use clap::Parser;
use termcolor::{ColorChoice, ColorSpec, StandardStream, WriteColor};
use usvg::{Node, TreeParsing, TreeTextToPath};

#[derive(Debug, Parser)]
#[clap(about, version)]
struct Args {
    /// Path to read SVG file from.
    input: PathBuf,
    /// Path to write PDF file to.
    output: Option<PathBuf>,
    /// The number of SVG pixels per PDF points.
    #[clap(long, default_value = "72.0")]
    dpi: f64,
}

fn main() {
    if let Err(msg) = run() {
        print_error(&msg).unwrap();
        process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let args = Args::parse();

    // Determine output path.
    let name =
        Path::new(args.input.file_name().ok_or("Input path does not point to a file")?);
    let output = args.output.unwrap_or_else(|| name.with_extension("pdf"));

    // Load source file.
    let svg =
        std::fs::read_to_string(&args.input).map_err(|_| "Failed to load SVG file")?;

    // Convert string to SVG.
    let options = usvg::Options::default();
    let mut fontdb = fontdb::Database::new();
    fontdb.load_system_fonts();

    let mut tree = usvg::Tree::from_str(&svg, &options).map_err(|err| err.to_string())?;
    tree.convert_text(&fontdb);

    //TODO: hide behind debug flag
    println!("Size: {:?}\nView Box: {:?}", tree.size, tree.view_box);
    print_tree(&tree.root, 0);

    // Convert SVG to PDF.
    let pdf = svg2pdf::convert_tree(&tree);

    // Write output file.
    std::fs::write(output, pdf).map_err(|_| "Failed to write PDF file")?;

    Ok(())
}

fn print_tree(node: &Node, level: u32) {
    for child in node.children() {
        println!("Level {}: {:#?}", level, child);
        print_tree(&child, level + 1);
    }
}

fn print_error(msg: &str) -> io::Result<()> {
    let mut w = StandardStream::stderr(ColorChoice::Always);

    let mut color = ColorSpec::new();
    color.set_fg(Some(termcolor::Color::Red));
    color.set_bold(true);
    w.set_color(&color)?;
    write!(w, "error")?;

    w.reset()?;
    writeln!(w, ": {msg}.")
}
