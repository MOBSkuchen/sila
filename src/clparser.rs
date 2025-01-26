
use crate::colorize_rs::AnsiColor;
pub const NAME: &str = env!("CARGO_PKG_NAME");
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");
pub const AUTHOR: &str = env!("CARGO_PKG_AUTHORS");

use colorize_rs::{color_enabled};
use crate::clparser::ClParserError::{FlagMissingValue, TooFewArguments, TooManyArguments};

#[derive(Debug)]
pub enum ClParserError {
    TooFewArguments(usize, usize),
    TooManyArguments(usize),
    FlagMissingValue(String)
}

pub type ClParserResult<T> = Result<T, ClParserError>;
pub type ClParserResultCallQueue = ClParserResult<Vec<PendingCall>>;
pub type ClParserResultCall = ClParserResult<Vec<String>>;
pub type ClParserResultNone = ClParserResult<()>;
pub type CallFunction = Box<dyn Fn(&ArgumentParser, &Vec<String>) -> bool>;

#[macro_export] 
macro_rules! mk_clfn {
    ($function: ident) => {
        Box::new(|apr, args| {$function(apr, args)})
    };
}

#[macro_export]
macro_rules! mk_clfn_ng {
    ($function: ident) => {
        Box::new(|apr, _| {$function(apr)})
    };
}

#[macro_export]
macro_rules! mk_clfn_static {
    ($function: ident) => {
        Box::new(|_, _| {$function()})
    };
}

pub fn fetch_args_clean() -> Vec<String> {
    let args: Vec<String> = std::env::args().collect();
    args[1..].to_vec()
}

fn _print_help(argument_parser: &ArgumentParser) -> bool {
    let mut shorts: Vec<String> = argument_parser.positionals.iter().map(|x| {x.short()}).collect();
    shorts.append(&mut argument_parser.arguments.iter().map(|x| {x.short()}).collect());
    shorts.append(&mut argument_parser.flags.iter().map(|x| {x.short()}).collect());
    println!("{} (v{}) usage => {}", argument_parser.prog, argument_parser.version, shorts.join(" "));
    for argument in &argument_parser.arguments {
        println!("-> {}", argument.get_description())
    }

    for pos in &argument_parser.positionals {
        println!("-> {}", pos.get_description())
    }

    for flag in &argument_parser.flags {
        println!("-> {}", flag.get_description())
    }
    
    false
}

fn _print_version(argument_parser: &ArgumentParser) -> bool {
    println!("{} => Version {}", argument_parser.prog, argument_parser.version);
    false
}

fn _disable_color(argument_parser: &ArgumentParser) -> bool {
    color_enabled(false);
    false
}

pub struct Argument {
    name: String,
    nargs: usize,
    args: Vec<String>,
    triggers: CallFunction,
    positional: bool,
    description: String,
}

impl Argument {
    pub fn new(name: String, args: Vec<String>, triggers: CallFunction, description: String, positional: bool) -> Self {
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

    fn call(&self, argument_parser: &ArgumentParser, args: &Vec<String>) -> bool {
        (self.triggers)(argument_parser, args)
    }
}

pub struct Flag {
    name: String,
    mini: String,
    value: bool,
    triggers: CallFunction,
    description: String,
    allow_end: bool,
}

impl Flag {
    pub fn new(name: String, mini: String, value: bool, triggers: CallFunction, description: String, allow_end: bool) -> Self {
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
            format!("{} / {} => <VALUE> | {}", self.name, self.mini, self.description)
        } else {
            format!("{} / {} | {}", self.name, self.mini, self.description)
        }
    }

    fn short(&self) -> String {
        if self.value {
            format!("[{} / {} <VALUE>]", self.name, self.mini)
        } else {
            format!("[{} / {}]", self.name, self.mini)
        }
    }

    fn call(&self, argument_parser: &ArgumentParser, value: &Vec<String>) -> bool {
        (self.triggers)(argument_parser, value)
    }
}

enum CallType {
    ARGUMENT,
    POSITIONAL,
    FLAG
}

struct PendingCall {
    index: usize,
    args: Vec<String>,
    call_type: CallType
}

impl PendingCall {
    pub fn new(index: usize, args: Vec<String>, call_type: CallType) -> Self {
        Self { index, args, call_type}
    }
    
    pub fn call(&self, argument_parser: &ArgumentParser) -> bool {
        match self.call_type {
            CallType::ARGUMENT => {
                argument_parser.arguments[self.index].call(argument_parser, &self.args)
            }
            CallType::POSITIONAL => {
                argument_parser.positionals[self.index].call(argument_parser, &self.args)
            }
            CallType::FLAG => {
                argument_parser.flags[self.index].call(argument_parser, &self.args)
            }
        }
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

    fn parse_argument(&self, argument: &Argument, args: &Vec<String>) -> ClParserResult<Vec<String>> {
        if args.len() < argument.nargs {
            Err(TooFewArguments(argument.nargs, args.len()))
        } else {
            Ok(args[1..argument.nargs+1].to_vec())
        }
    }

    fn parse_positionals(&self, args: &mut Vec<String>) -> ClParserResultCallQueue {
        let mut remove_list = vec![];
        let mut pending_calls = vec![];
        for (i, pos) in self.positionals.iter().enumerate() {
            pending_calls.push(PendingCall::new(i, self.parse_argument(pos, args)?, CallType::POSITIONAL));
            remove_list.append(&mut (i..pos.args.len()+1).collect());
        }

        for (i, remove) in remove_list.iter().enumerate() {
            args.remove(remove - i);
        }
        
        Ok(pending_calls)
    }

    fn parse_arguments(&self, args: &mut Vec<String>) -> ClParserResultCallQueue {
        let mut remove_list = vec![];
        let mut pending_calls = vec![];
        for (i, arg) in args.iter().enumerate() {
            for (n, darg) in (&self.arguments).iter().enumerate() {
                if darg.name == *arg {
                    pending_calls.push(PendingCall::new(n, self.parse_argument(darg, args)?, CallType::ARGUMENT));
                    remove_list.append(&mut (i..darg.args.len()+1).collect());
                }
            }
        }

        for (i, remove) in remove_list.iter().enumerate() {
            args.remove(remove - i);
        }
        
        Ok(pending_calls)
    }

    fn parse_flag(&self, flag: &Flag, item: &str, args: &Vec<String>, i: usize) -> ClParserResult<Vec<String>> {
        let val = if flag.value {
            let value;
            if item.contains("=") {
                value = item.split_once("=").expect("This should not fail").1;
            } else if (args.len() as i32 - 1) > (i as i32) {
                value = &args[i + 1];
            } else {
                return Err(FlagMissingValue(item.into()))
            }
            vec![value.to_string()]
        } else {
            vec![]
        };

        Ok(val)
    }

    fn parse_flags(&self, args: &mut Vec<String>) -> ClParserResultCallQueue {
        let mut remove_list = vec![];
        let mut pending_calls = vec![];
        for i in 0..args.len() {
            let item = &args[i];
            for (n, flag) in (&self.flags).iter().enumerate() {
                if item.starts_with(&flag.name) && (!flag.value || (&flag.name == item)) || 
                    item.starts_with(&flag.mini) && (!flag.value || (&flag.mini == item)) {
                    pending_calls.push(PendingCall::new(n, self.parse_flag(flag, item, args, i)?, CallType::FLAG));
                    remove_list.push(i);
                    if flag.value { remove_list.push(i + 1); }
                }
            }
        }

        for (i, remove) in remove_list.iter().enumerate() {
            args.remove(remove - i);
        }

        Ok(pending_calls)
    }

    pub fn parse(&self, mut args: Vec<String>) -> ClParserResultNone {
        let mut pending_calls = vec![];
        pending_calls.append(&mut self.parse_flags(&mut args)?);
        pending_calls.append(&mut self.parse_arguments(&mut args)?);
        pending_calls.append(&mut self.parse_positionals(&mut args)?);
        if args.len() > 0 {
            Err(TooManyArguments(args.len()))
        } else {
            for x in pending_calls {
                if x.call(self) {break}
            }
            Ok(())
        }
    }
    
    pub fn handle_errors(&self, error: ClParserError) {
        match error {
            TooFewArguments(expected, got) => {
                println!("Invalid usage! Got too few arguments. Expected {}, but got {}", 
                         expected.to_string().bold().underlined(), got.to_string().bold().underlined());
                _print_help(self);
            }
            TooManyArguments(got) => {
                println!("Invalid usage! Got too many arguments. Got {} too many",
                         got.to_string().bold().underlined());
                _print_help(self);
            }
            FlagMissingValue(flag) => {
                println!("The flag '{}' is missing its value! Look at its description below:", flag);
                _print_help(self);
            }
        }
    }

    pub fn get_auto_help(&self) -> Flag {
        Flag::new(
            "--help".to_string(),
            "-h".to_string(),
            false,
            mk_clfn_ng!(_print_help),
            "Get help".to_string(),
            true
        )
    }

    pub fn get_auto_version(&self) -> Flag {
        Flag::new(
            "--version".to_string(),
            "-v".to_string(),
            false,
            mk_clfn_ng!(_print_help),
            "Get version".to_string(),
            true
        )
    }

    pub fn get_auto_no_color(&self) -> Flag {
        Flag::new(
            "--no-color".to_string(),
            "-nc".to_string(),
            false,
            mk_clfn_ng!(_print_help),
            "Disable colors".to_string(),
            false
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

    pub fn add_no_color(&mut self) -> &mut Self {
        self.flags.push(self.get_auto_no_color());
        self
    }
}