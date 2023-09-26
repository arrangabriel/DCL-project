use std::fs::File;
use std::io::Read;
use std::path::Path;

use anyhow::Result;
use wasmparser;
use wasmparser::{Parser, Payload::*};
use wasmprinter;

fn main() {
    let file_path = Path::new("./wasm-example-files/c/helloworld.wasm");
    let print_result = print_wasm_file(file_path);
    println!("{}", print_result.unwrap());
    let file = File::open(file_path).unwrap();
    parse(file).unwrap();
}

fn parse(mut reader: impl Read) -> Result<()> {
    let mut buf = Vec::new();
    reader.read_to_end(&mut buf)?;
    let parser = Parser::new(0);

    for payload in parser.parse_all(&buf) {
        match payload? {
            // Sections for WebAssembly modules
            Version { .. } => { /* ... */ }
            TypeSection(_) => { /* ... */ }
            ImportSection(_) => { /* ... */ }
            FunctionSection(_) => { /* ... */ }
            TableSection(_) => { /* ... */ }
            MemorySection(_) => { /* ... */ }
            TagSection(_) => { /* ... */ }
            GlobalSection(_) => { /* ... */ }
            ExportSection(_) => { /* ... */ }
            StartSection { .. } => { /* ... */ }
            ElementSection(_) => { /* ... */ }
            DataCountSection { .. } => { /* ... */ }
            DataSection(_) => { /* ... */ }

            // Here we know how many functions we'll be receiving as
            // `CodeSectionEntry`, so we can prepare for that, and
            // afterwards we can parse and handle each function
            // individually.
            CodeSectionStart { .. } => { /* ... */ }
            CodeSectionEntry(body) => {
                // here we can iterate over `body` to parse the function
                // and its locals
            }

            // Sections for WebAssembly components
            ModuleSection { .. } => { /* ... */ }
            InstanceSection(_) => { /* ... */ }
            CoreTypeSection(_) => { /* ... */ }
            ComponentSection { .. } => { /* ... */ }
            ComponentInstanceSection(_) => { /* ... */ }
            ComponentAliasSection(_) => { /* ... */ }
            ComponentTypeSection(_) => { /* ... */ }
            ComponentCanonicalSection(_) => { /* ... */ }
            ComponentStartSection { .. } => { /* ... */ }
            ComponentImportSection(_) => { /* ... */ }
            ComponentExportSection(_) => { /* ... */ }

            CustomSection(_) => { /* ... */ }

            // most likely you'd return an error here
            UnknownSection { id, .. } => { /* ... */ }

            // Once we've reached the end of a parser we either resume
            // at the parent parser or the payload iterator is at its
            // end and we're done.
            End(_) => {}
        }
    }

    Ok(())
}

fn print_wasm_file(file_path: &Path) -> Result<String> {
    wasmprinter::print_file(file_path)
}