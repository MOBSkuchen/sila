#![allow(static_mut_refs)]
extern crate colorize_rs;

use crate::clparser::{fetch_args_clean, Argument, ArgumentParser, Flag};
use crate::comp_errors::CodeResult;
use crate::filemanager::FileManager;
use crate::lexer::tokenize;
use crate::parser::Parser;
use std::string::ToString;

mod clparser;
mod codeviz;
mod comp_errors;
mod filemanager;
mod lexer;
mod parser;

fn compile_job(file_manager: &FileManager) -> CodeResult<()> {
    let tokens = tokenize(file_manager.get_content())?;

    let parser = Parser::new(tokens, file_manager);
    let ast = parser.parse(&mut 0)?;

    for item in ast {
        println!("{:?}", item);
    }

    Ok(())
}

fn _compile(_: &ArgumentParser, args: &Vec<String>) -> bool {
    let file_manager_r = FileManager::new_from(args[0].clone());
    if file_manager_r.is_err() {
        file_manager_r.unwrap_err().output();
        return true;
    }

    let file_manager = file_manager_r.unwrap();

    let x = compile_job(&file_manager);
    if x.is_err() {
        x.unwrap_err().visualize_error(&file_manager);
    }

    false
}

fn main() {
    let mut argument_parser = ArgumentParser::new();
    argument_parser.add_help();
    argument_parser.add_version();
    argument_parser.add_no_color();
    argument_parser.add_argument(Argument::new(
        "compile".to_string(),
        vec!["file_path".to_string()],
        mk_clfn!(_compile),
        "Compile a file".to_string(),
        false,
    ));
    argument_parser.add_flag(Flag::new(
        "--output".to_string(),
        "-o".to_string(),
        true,
        empty!(),
        "Set output path".to_string(),
    ));

    let result = argument_parser.parse(fetch_args_clean(), true);
    if result.is_err() {
        argument_parser.handle_errors(result.unwrap_err());
        return;
    }
    let (pending_calls, flag_map) = result.unwrap();

    for pending_call in pending_calls {
        if pending_call.has_name("compile".to_string()) {
            pending_call.call(
                &argument_parser,
                Some(&pending_call.merge_args(vec![(&flag_map).get("--output")
                .unwrap().clone().or(Some("output".to_string())).unwrap()])),
            );
            break;
        }

        if pending_call.call(&argument_parser, None) {
            break;
        }
    }
}
