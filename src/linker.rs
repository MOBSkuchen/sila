use lld_rx::{link, LldFlavor, LldResult};

fn set_entry(lld_flavor: &LldFlavor, args: &mut Vec<String>, entry: String) {
    match lld_flavor {
        LldFlavor::Elf => {
            args.push(format!("-e {}", entry));
        }
        LldFlavor::Wasm => {
            args.push(format!("-e {}", entry));
        }
        LldFlavor::MachO => {
            args.push(format!("-e {}", entry));
        }
        LldFlavor::Coff => {
            args.push(format!("/entry:{}", entry));
        }
    }
}

fn set_output(lld_flavor: &LldFlavor, args: &mut Vec<String>, output: String) {
    match lld_flavor {
        LldFlavor::Elf => {
            args.push(format!("-o \"{}\"", output));
        }
        LldFlavor::Wasm => {
            args.push(format!("-o \"{}\"", output));
        }
        LldFlavor::MachO => {
            args.push(format!("-o \"{}\"", output));
        }
        LldFlavor::Coff => {
            args.push(format!("/out:\"{}\"", output));
        }
    }
}

fn lld_link(target: LldFlavor, output_path: String, 
            is_lib: bool, mut extra_args: Vec<String>, 
            start_symbol: Option<String>) -> LldResult {
    if is_lib && start_symbol.is_some() {
        println!("Start symbol {} will be discarded as you are building a library.", start_symbol.clone().unwrap());
    }
    
    let mut args: Vec<String> = vec![];
    
    if is_lib {
        args.push("/dll".into())
    }
    
    if start_symbol.is_some() {
        set_entry(&target, &mut args, start_symbol.unwrap());
    }
    
    set_output(&target, &mut args, output_path);
    
    args.append(&mut extra_args);
    
    link(target, args)
}