use super::interface;
use super::engine;
use std::io;


pub fn call_fs_query(x: String) {
    match engine::query_folder(&x) {
        Ok(l) => interface::print_dir_list(&x, l),
        Err(e) => println!("{}", e),
    }
}

pub fn do_proc_list() {
    match engine::query_procs() {
        Ok(l) => interface::print_proc_table(l),
        Err(e) => println!("{}", e),
    }
}

pub fn do_find_proc(m: Option<&str>) {
    match m {
        Some(x) => {
            match engine::do_find_procs(&x) {
                Ok(l) => interface::print_proc_table(l),
                Err(e) => println!("{}", e),
            }
        },
        _ => println!("[!] Expected format: find <proc name>"),
    }
}

pub fn do_search_proc(m: Option<&str>) {
    match m {
        Some(x) => {
            match engine::search_proc_memory(&x) {
                Ok(l) => interface::print_proc_list(l),
                Err(e) => println!("{}", e),
            }
        },
        _ => println!("[!] Expected format: search <str>"),
    }
}


pub fn call_proc_query(input: String) {
    let mut args= input.splitn(2, " ");
    let o = args.next();
    match o {
        Some("ls") => do_proc_list(),
        Some("find") => {
            let m = args.next();
            do_find_proc(m);
        },
        Some("search") => {
            let m = args.next();
            do_search_proc(m);
        }
        Some(_) => println!("TO DO!"),
        _ => interface::print_proc_help()
    }
}

pub fn mainloop() {
    interface::print_banner();
    loop {
        interface::print_prompt();
        let input = get_option();
        let args: Vec<&str> = input.split("::").collect();
        let z = *args.get(0).unwrap();
        if z == "q" {
            break
        } else {
            let x = args.get(1);
            match (x, z) {
                (Some(y), "fs") => call_fs_query((*y).to_string()),
                (Some(y), "ps") => call_proc_query((*y).to_string()),
                _ => {
                    println!("to do!");
                },
            }
        }
    }

}

pub fn get_option() -> String {
    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("can't process input");
    input.trim().to_string()
}