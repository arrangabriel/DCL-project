use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

use anyhow::{anyhow, Error, Result};
use wast::core::{FuncKind, ModuleField, ModuleKind};
use wast::parser::{parse, ParseBuffer};
use wast::Wat;

use crate::chop_up::{emit_transformed_wat, IGNORE_FUNC_PREFIX, InstructionType, MemoryInstructionType};

mod chop_up;

pub fn run_split(file_path: &str, state_size: usize, skip_safe: bool, explain: bool, output: &mut dyn Write) -> Result<()> {
    let file_contents = read_file(file_path)?;
    transform_wat_string(&file_contents, output, state_size, skip_safe, explain)
}

pub fn transform_wat_string(input: &str, output: &mut dyn Write, state_size: usize, skip_safe: bool, explain: bool) -> Result<()> {
    let buffer = ParseBuffer::new(input)?;
    let wat = parse(&buffer)?;
    emit_transformed_wat(
        &wat,
        &input.split('\n').collect::<Vec<&str>>(),
        output,
        skip_safe,
        state_size,
        explain,
    )
}

pub enum OutputFormat {
    Normal,
    CSV,
}

pub fn run_analysis(file_path: &str, output_format: OutputFormat) -> Result<()> {
    let file_contents = read_file(file_path)?;
    let buffer = ParseBuffer::new(&file_contents)?;
    let wat = parse(&buffer)?;
    let (instruction_count, load_count, store_count) = analyze_wat(&wat)?;

    let memory_instruction_count = load_count + store_count;
    let normal_instruction_count = instruction_count - memory_instruction_count;
    match output_format {
        OutputFormat::Normal => {
            println!("\
Analysis for file: {file_path}
Total size = {file_size}
Total instructions = {instruction_count}
  Of which:
  Normal instructions = {normal_instruction_count}
  Memory instructions = {memory_instruction_count}
    Of which:
    Load instructions  = {load_count}
    Store instructions = {store_count}",
                     file_size = file_contents.bytes().len());
        }
        OutputFormat::CSV => {
            println!("\
file,size,total_instructions,normal_instructions,memory_instructions,load_instructions,store_instructions
{file_path},{file_size},{instruction_count},{normal_instruction_count},{memory_instruction_count},{load_count},{store_count}",
                     file_size = file_contents.bytes().len());
        }
    }
    Ok(())
}

fn analyze_wat(wat: &Wat) -> Result<(i32, i32, i32)> {
    let mut instruction_count = 0;
    let mut load_count = 0;
    let mut store_count = 0;
    for field in extract_module_fields(wat)? {
        if let ModuleField::Func(func) = field {
            if let Some(name) = func.name {
                if name.name.starts_with(IGNORE_FUNC_PREFIX) { continue; }
            }

            if let FuncKind::Inline { expression, .. } = &func.kind {
                for instruction in expression.instrs.iter() {
                    match InstructionType::from(instruction) {
                        InstructionType::Memory(ty) => {
                            instruction_count += 1;
                            match ty {
                                MemoryInstructionType::Load { .. } => load_count += 1,
                                MemoryInstructionType::Store { .. } => store_count += 1
                            }
                        }
                        InstructionType::Benign(_) => instruction_count += 1
                    }
                }
            } else {
                return Err(anyhow!("FuncKind is not inline"));
            };
        }
    }
    Ok((instruction_count, load_count, store_count))
}

fn read_file(file_path: &str) -> Result<String> {
    let path = Path::new(file_path);
    if !path.is_file() {
        return Err(anyhow!("No such file: {file_path}"));
    }
    let mut file_contents = String::new();
    File::open(path)
        .and_then(|mut file| file.read_to_string(&mut file_contents))
        .map_err(|err| anyhow!("Failed to read file: {err:?}"))?;
    Ok(file_contents)
}

fn extract_module_fields<'a>(wat: &'a Wat) -> Result<&'a [ModuleField<'a>]> {
    match wat {
        Wat::Module(module) => match &module.kind {
            ModuleKind::Text(fields) => Ok(fields),
            ModuleKind::Binary(_) => Err("ModuleKind is binary"),
        },
        Wat::Component(_) => Err("Input module is component"),
    }.map_err(Error::msg).map(|fields| fields.as_slice())
}
