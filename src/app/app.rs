use super::engine;
use super::interface;
use regex::Regex;
use std::io;

pub struct UserInput {
    table_name: String,
    params: Vec<String>,
    filter_string: String,
}

pub fn parse_schema_query(input: String) -> Result<String, String> {
    let re = Regex::new(r"show\s([a-zA-z_]+)\.schema;").unwrap();
    let mut capts = re.captures_iter(&input[..]);
    let m = capts.next();
    match m {
        Some(p) => {
            Ok(p[1].to_string())
        },
        _ => Err("Error! If you can see this message something is really wrong. My bad.".to_string()),
    }
}

pub fn parse_option(input: String) -> Result<UserInput, String> {
    let re = Regex::new(r"select\s([a-zA-z_\*,|\s?]+)\sfrom\s([a-z_]+)(\swhere\s(.*))?;").unwrap();
    let mut capts = re.captures_iter(&input[..]);
    let m = capts.next();

    match m {
        Some(p) => {
            //println!("{:?}", p);
            let par: Vec<String> = p[1].split(",").map(|x| x.trim().to_string()).collect();
            let filter_string = match p.get(4) {
                Some(_) => p[4].to_string(),
                _ => String::new(),
            };

            let ui = UserInput {
                table_name: p[2].to_string(),
                params: par,
                filter_string: filter_string,
            };
            Ok(ui)
        }
        _ => Err("Oh Nos! Expected query syntax is select <cols> from <table> where <cond>;\n".to_string()),
    }
}

fn handle_input(ui: &mut UserInput) {
    match ui.table_name.as_str() {
        "procs" => {
            /*let filter_items_test = engine::FilterItems{
                filters: vec![engine::FilterItem::new("uid".to_string(), engine::FilterOp::Eq, "1000".to_string())]
            };*/
            match engine::query_procs(&mut ui.params, &ui.filter_string) {
                Ok(res) => interface::print_data_table(res),
                _ => println!("ERROR!"),
            }
        }
        "fs" => match engine::query_dir(&mut ui.params, &ui.filter_string) {
            Ok(res) => interface::print_data_table(res),
            _ => println!("ERROR!"),
        },
        "os_version" => match engine::do_get_os_version_info(&mut ui.params) {
            Ok(res) => interface::print_hash_table(res),
            _ => println!("ERROR!"),
        },
        _ => println!("Uh Oh! Table {} does not exist.", ui.table_name),
    }
}

pub fn get_schema(table: String) {
    match table.as_str() {
        "os_version" => interface::print_os_version_schema(),
        "procs" => interface::print_procs_schema(),
        "fs" => interface::print_fs_schema(),
        _ => println!("Uh Oh! Table {} does not exist!", table),
    }
}

pub fn mainloop() {
    interface::print_banner();
    loop {
        interface::print_prompt();
        let input = get_option();
        if input == "q" {
            break;
        } if input == "h" {
            interface::print_help();
        } else if input == "show dog;" {
            interface::dog();
        } else {
            match parse_schema_query(input.clone()) {
            Ok(x) => {
                get_schema(x)
            },
            _ => {
             match parse_option(input) {
                Ok(mut ui) => handle_input(&mut ui),
                Err(e) => println!("⚠ {}", e),
                }  
            }
            }
        }
    }
}

pub fn get_option() -> String {
    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("can't process input");
    input.trim().to_string()
}
