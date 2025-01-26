
pub const NAME: &str = env!("CARGO_PKG_NAME");
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");
pub const AUTHOR: &str = env!("CARGO_PKG_AUTHORS");

use std::collections::HashMap;
use color_print::{cprintln};

#[derive(Debug)]
pub enum ClParserError {
    TooFewArguments(usize, usize),
    TooManyArguments(usize),
    FlagMissingValue(String)
}

pub type ClParserResult<T> = Result<T, ClParserError>;
pub type ClParserResultSatisfied = ClParserResult<bool>;

pub fn fetch_args_clean() -> Vec<String> {
    let args: Vec<String> = std::env::args().collect();
    args[1..].to_vec()
}

fn _print_help(argument_parser: &ArgumentParser) {
    
}

fn _print_version(argument_parser: &ArgumentParser) {
    cprintln!("<green,bold>{}</> => <yellow>Version <bold><blink>{}", argument_parser.prog, argument_parser.version);
}

pub struct Argument {
    name: String,
    nargs: usize,
    args: Vec<String>,
    triggers: Box<dyn Fn(&ArgumentParser, Vec<String>)>,
    positional: bool,
    description: String,
}

impl Argument {
    pub fn new(name: String, args: Vec<String>, triggers: Box<dyn Fn(&ArgumentParser, Vec<String>)>, description: String, positional: bool) -> Self {
        let nargs = args.len();
        Self {
            name,
            nargs,
            args,
            triggers,
            positional,
            description,
        }
    }

    fn get_description(&self) -> String {
        if self.nargs < 2 {
            format!("{} | {}", self.name, self.description)
        } else {
            format!("{} => {} | {}", self.name, self.args.join(" "), self.description)
        }
    }

    fn short(&self) -> String {
        if self.nargs < 2 {
            format!("[{}]", self.name)
        } else {
            format!("[{} {}]", self.name, self.args.join(" "))
        }
    }

    fn call(&self, argument_parser: &ArgumentParser, args: Vec<String>) {
        (self.triggers)(argument_parser, args);
    }
}

pub struct Flag {
    name: String,
    mini: String,
    value: bool,
    triggers: Box<dyn Fn(&ArgumentParser, Option<String>)>,
    description: String,
    allow_end: bool,
}

impl Flag {
    pub fn new(name: String, mini: String, value: bool, triggers: Box<dyn Fn(&ArgumentParser, Option<String>)>, description: String, allow_end: bool) -> Self {
        Self {
            name,
            mini,
            value,
            triggers,
            description,
            allow_end,
        }
    }

    fn get_description(&self) -> String {
        if self.value {
            format!("--{} / -{} => <VALUE> | {}", self.name, self.mini, self.description)
        } else {
            format!("--{} / -{} | {}", self.name, self.mini, self.description)
        }
    }

    fn short(&self) -> String {
        if self.value {
            format!("[--{} / -{} <VALUE>]", self.name, self.mini)
        } else {
            format!("[--{} / -{}]", self.name, self.mini)
        }
    }

    fn call(&self, argument_parser: &ArgumentParser, value: Option<String>) {
        (self.triggers)(argument_parser, value);
    }
}

pub struct ArgumentParser {
    prog: String,
    version: String,
    description: String,
    arguments: Vec<Argument>,
    positionals: Vec<Argument>,
    flags: Vec<Flag>,
}

impl ArgumentParser {
    pub fn new() -> Self {
        Self {
            prog: NAME.to_string(),
            version: VERSION.to_string(),
            description: DESCRIPTION.to_string(),
            arguments: Vec::new(),
            positionals: Vec::new(),
            flags: Vec::new(),
        }
    }

    pub fn add_argument(&mut self, arg: Argument) -> &mut Self {
        if arg.positional {
            self.positionals.push(arg);
        } else {
            self.arguments.push(arg);
        }
        self
    }

    pub fn add_flag(&mut self, flag: Flag) -> &mut Self {
        self.flags.push(flag);
        self
    }

    fn parse_argument(&self, argument: &Argument, args: &[String], index: &mut usize, satisfied: bool) -> ClParserResultSatisfied {
        if args.len() - *index < argument.nargs {
            if satisfied {
                Ok(true)
            } else {
                Err(ClParserError::TooFewArguments(argument.nargs, args.len() - *index))
            }
        } else {
            let arg_values: Vec<String> = args[*index..*index + argument.nargs].to_vec();
            *index += argument.nargs;
            argument.call(&self, arg_values);
            Ok(false)
        }
    }

    fn parse_positionals(&self, args: &[String], index: &mut usize, mut satisfied: bool) -> ClParserResultSatisfied {
        for pos in &self.positionals {
            satisfied = self.parse_argument(pos, args, index, satisfied)?;
        }
        Ok(satisfied)
    }

    fn parse_flag(&self, flag: &Flag, call: &str, vals: &HashMap<String, String>) -> ClParserResultSatisfied {
        let val = if flag.value {
            vals.get(call).cloned()
        } else {
            None
        };

        flag.call(&self, val);

        if flag.allow_end {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn parse_flags(&self, args: &[String], index: &mut usize, mut satisfied: bool) -> ClParserResultSatisfied {
        let mut vals = HashMap::new();
        let mut names = Vec::new();
        let mut minis = Vec::new();

        for arg in args.iter().skip(*index) {
            if arg.starts_with("--") {
                let mut key = &arg[2..];
                if let Some((k, v)) = key.split_once('=') {
                    key = k;
                    vals.insert(k.to_string(), v.to_string());
                }
                names.push(key.to_string());
            } else if arg.starts_with('-') {
                let mut key = &arg[1..];
                if let Some((k, v)) = key.split_once('=') {
                    key = k;
                    vals.insert(k.to_string(), v.to_string());
                }
                minis.push(key.to_string());
            }
        }

        for flag in &self.flags {
            if names.contains(&flag.name) {
                satisfied = satisfied || self.parse_flag(flag, &flag.name, &vals)?;
            } else if minis.contains(&flag.mini) {
                satisfied = satisfied || self.parse_flag(flag, &flag.mini, &vals)?;
            } else {
            }
        }
        
        Ok(satisfied)
    }

    pub fn parse(&self, args: Vec<String>) -> ClParserResultSatisfied {
        let mut index = 0;
        let mut satisfied = false;
        
        satisfied = self.parse_flags(&args, &mut index, satisfied)?;
        satisfied = self.parse_positionals(&args, &mut index, satisfied)?;

        if index < args.len() && !satisfied {
            Err(ClParserError::TooManyArguments(args.len() - index))
        } else {
            Ok(satisfied)
        }
    }

    pub fn get_auto_help(&self) -> Flag {
        Flag::new(
            "help".to_string(),
            "h".to_string(),
            false,
            { Box::new(|apr, _| _print_help(apr)) },
            "Get help".to_string(),
            true,
        )
    }

    pub fn get_auto_version(&self) -> Flag {
        Flag::new(
            "version".to_string(),
            "v".to_string(),
            false,
            { Box::new(|apr, _| _print_version(apr)) },
            "Get version".to_string(),
            true,
        )
    }
    
    pub fn add_help(&mut self) -> &mut Self {
        self.flags.push(self.get_auto_help());
        self
    }

    pub fn add_version(&mut self) -> &mut Self {
        self.flags.push(self.get_auto_version());
        self
    }
}