use super::interface;
use super::engine;
use std::io;
use regex::Regex;

pub struct UserInput {
    table_name: String,
    params: Vec<String>,
    filter_string: String,
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
            
            let ui = UserInput{
                table_name: p[2].to_string(),
                params: par,
                filter_string: filter_string,
            };
            Ok(ui)
        },
        _ => Err("Could not parse input".to_string())
    }

}

fn handle_input(ui:  &mut UserInput) {
    match ui.table_name.as_str() {
        "procs" => {
            /*let filter_items_test = engine::FilterItems{
                filters: vec![engine::FilterItem::new("uid".to_string(), engine::FilterOp::Eq, "1000".to_string())]
            };*/
            match engine::query_procs(&mut ui.params, &ui.filter_string) {
                Ok(res) => interface::print_data_table(res),
                _ => println!("ERROR!"),
            }
        },
        "fs" => println!("to do!"),
        "os_version" => {
            match engine::do_get_os_version_info(&mut ui.params) {
                Ok(res) => interface::print_hash_table(res),
                _ => println!("ERROR!"),
            }
        },
        _ => println!("to do: {}", ui.table_name),
    }
}



pub fn mainloop() {
    interface::print_banner();
    loop {
        interface::print_prompt();
        let input = get_option();
        if input == "q" {
            break
        } else {
            match parse_option(input) {
                Ok(mut ui) => handle_input(&mut ui),
                Err(e) => println!("ERROR: {}", e),
            }
        }
    }
}
/*
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
*/

pub fn get_option() -> String {
    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("can't process input");
    input.trim().to_string()
}