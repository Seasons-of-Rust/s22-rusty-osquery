use termimad;
use termimad::MadSkin;
use ansi_term::{Colour, Style};
use termimad::Alignment;
use std::io::{self, Write};

use super::engine::{ProcItem, FileItem};

pub fn print_banner() {
    let banner = "
    ┬─┐┬ ┬┌─┐┌┬┐┬ ┬       
    ├┬┘│ │└─┐ │ └┬┘       
    ┴└─└─┘└─┘ ┴  ┴        
    ┌─┐┌─┐┌─┐ ┬ ┬┌─┐┬─┐┬ ┬
    │ │└─┐│─┼┐│ │├┤ ├┬┘└┬┘
    └─┘└─┘└─┘└└─┘└─┘┴└─ ┴ 
    ";
    println!("{}", Colour::Cyan.paint(banner));
}

pub fn print_prompt() {
    println!("h - help, fs - filesystem query, ps - process query, q - quit");
    print!(">>> ");
    io::stdout().flush().unwrap();
}

pub fn print_proc_help() {
    println!("ls - list processes, find - find process by name, search - find processes containing a particular string, mem - show memory usage, count - count the number of running processes");
}

pub fn print_proc_table(entries: Vec<ProcItem>) {
    println!("");

    let mut skin = MadSkin::default();
    let mut text_template: String = "|:-:|:-:|\n|**PID**|**UID**|**Command Line**|\n|:-|:-|:-|\n".to_string();
    
    skin.paragraph.align = Alignment::Left;
    skin.table.align = Alignment::Left;
    for x in entries {
        let s = format!("|{}|{}|{}|\n", x.pid, x.owner, x.cmdline);
        text_template.push_str(&s);
    }
    text_template.push_str("|-");

    println!("{}", skin.term_text(&text_template[..]));
    println!("\n");
}

pub fn print_proc_list(entries: Vec<String>) {
    for x in entries {
        println!("- {}", x);
    }
    println!("\n");
}

pub fn print_dir_list(dir_name: &String, entries: Vec<FileItem>){
    let mut text_template: String = format!("{}\n", Style::new().bold().paint(dir_name)).to_string();
    
    let n = entries.len();
    for i in 0..n {
        let x = &entries[i];
        if i == n-1 {
            let s = format!("└── {}\n", x);
            text_template.push_str(&s);
        } else {
            let s = format!("├── {}\n", x);
            text_template.push_str(&s);
        }
    }
    println!("{}", text_template);


    println!("\n");
}
