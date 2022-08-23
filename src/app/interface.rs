use ansi_term::{Colour, Style};
use std::collections::HashMap;
use std::io::{self, Write};
use termimad;
use termimad::Alignment;
use termimad::*;
use termimad::MadSkin;

pub fn print_banner() {
    let banner = "                              
                              ░░███                                 
 ████████  █████ ████  █████  ███████   █████ ████                 
░░███░░███░░███ ░███  ███░░  ░░░███░   ░░███ ░███                   
 ░███ ░░░  ░███ ░███ ░░█████   ░███     ░███ ░███                   
 ░███      ░███ ░███  ░░░░███  ░███ ███ ░███ ░███                   
 █████     ░░████████ ██████   ░░█████  ░░███████                   
░░░░░       ░░░░░░░░ ░░░░░░     ░░░░░    ░░░░░███                   
                                         ███ ░███                   
                                        ░░██████                    
  ██████   █████   ████████ █████ ████  ██████░ ████████  █████ ████
 ███░░███ ███░░   ███░░███ ░░███ ░███  ███░░███░░███░░███░░███ ░███ 
░███ ░███░░█████ ░███ ░███  ░███ ░███ ░███████  ░███ ░░░  ░███ ░███ 
░███ ░███ ░░░░███░███ ░███  ░███ ░███ ░███░░░   ░███      ░███ ░███ 
░░██████  ██████ ░░███████  ░░████████░░██████  █████     ░░███████ 
 ░░░░░░  ░░░░░░   ░░░░░███   ░░░░░░░░  ░░░░░░  ░░░░░       ░░░░░███ 
                      ░███                                 ███ ░███ 
                      █████                               ░░██████  
                     ░░░░░                                 ░░░░░░
    
    ";
    println!("{}", Colour::Cyan.paint(banner));

}

pub fn print_prompt() {
    println!("h - help, q - quit");
    print!(">>> ");
    io::stdout().flush().unwrap();
}

pub fn print_help() {
    let mut skin = MadSkin::default();
    skin.set_headers_fg(rgb(31, 255, 15));
    let text_template = r#"
## Help

### Available Tables:

|:-|:-|
|**Table**|**Description**|
|:-|:-|
| procs | Processes running on the system |
| proc_maps | Data mapped into process memory |
| net | Network connections |
| fs | Query the file system |
| os_version | Query the operating system version |
|-

### Command Syntax:

select <columns> from <table> where <key value pairs>;

* values must be surronded by quotations
* key-value pairs must be seperated by a comma

### Table Schema:

To print table schema:

show <table name>.schema;

"#;
    println!("");
    println!("{}", skin.term_text(&text_template[..]));
    println!("\n");
}

pub fn print_procs_schema() {
    println!("");

    let mut skin = MadSkin::default();
    let mut text_template: String  = "|:-|:-|\n|**pid**|The Process ID|\n|**uid**|The ID of the user who ran the program|\n|**cmdline**|The command used to run the program|\n|-".to_string();

    skin.paragraph.align = Alignment::Left;
    skin.table.align = Alignment::Left;

    println!("{}", skin.term_text(&text_template[..]));
    println!("\n");
}

pub fn print_fs_schema() {
    println!("");

    let mut skin = MadSkin::default();
    let mut text_template: String  = "|:-|:-|\n|**name*|The name of the file|\n|**dir**|The file type|\n|**uid**|The uid of the file owner|\n|**created**|The file creation time|\n|-".to_string();

    skin.paragraph.align = Alignment::Left;
    skin.table.align = Alignment::Left;

    println!("{}", skin.term_text(&text_template[..]));
    println!("\n");
}

pub fn print_proc_map_schema() {
    println!("");

    let mut skin = MadSkin::default();
    let mut text_template: String  = "|:-|:-|\n|**start_addr**|Start of mapping in process vm|\n|**end_addr**|End of mapping in process vm|\n|**permissions**|Permissions (rwx) for the section|\n|**size_maybe**|Size, maybe?|\n|**start_time**|Time at which the mapping occurred, I think |\n|**file_desc**|File Descriptor|\n|**file_name**|The name of the file mapped into memory if applicable|\n|-".to_string();

    skin.paragraph.align = Alignment::Left;
    skin.table.align = Alignment::Left;

    println!("{}", skin.term_text(&text_template[..]));
    println!("\n");
}

pub fn print_net_schema() {
    println!("");

    let mut skin = MadSkin::default();
    let mut text_template: String  = "|:-|:-|\n|**uid**|uid of the owner of the network connection|\n|**local_address**|Our address|\n|**remote_address**|Their address|\n|**st**|Connection State|\n|**timeout**|Timeout|\n|**inode**|Inode|\n|**ref_cnt**|Reference Count|\n|**protocol**|Network Protocol (tcp or udp)|\n|-".to_string();

    skin.paragraph.align = Alignment::Left;
    skin.table.align = Alignment::Left;

    println!("{}", skin.term_text(&text_template[..]));
    println!("\n");
}

pub fn print_os_version_schema() {
    println!("");

    let mut skin = MadSkin::default();
    let mut text_template: String  = "|:-|:-|\n|**kernel_version**|The version of the Linux kernel in use|\n|**build_id**|Hopefully the distro name?|\n|**gcc_version**|The version of the gcc compiler in use|\n|-".to_string();

    skin.paragraph.align = Alignment::Left;
    skin.table.align = Alignment::Left;

    println!("{}", skin.term_text(&text_template[..]));
    println!("\n");
}

pub fn print_hash_table(entries: HashMap<String, String>) {
    println!("");

    let mut skin = MadSkin::default();
    let mut text_template: String = "|:-|:-|\n".to_string();

    skin.paragraph.align = Alignment::Left;
    skin.table.align = Alignment::Left;
    for (k, v) in &entries {
        let s = format!("|**{}**|{}|\n", k, v);
        text_template.push_str(&s);
    }
    text_template.push_str("|-");

    println!("{}", skin.term_text(&text_template[..]));
    println!("\n");
}

pub fn print_data_table(table: String) {
    println!("");
    let mut skin = MadSkin::default();

    skin.paragraph.align = Alignment::Left;
    skin.table.align = Alignment::Left;
    let text_template = format!("{}|-", table);
    println!("{}", skin.term_text(&text_template[..]));
    println!("\n");
}

pub fn dog() {
    println!("");
    let lesser_dog =  "
    ░░░░░░░░░░░▄▀▄▀▀▀▀▄▀▄░░░░░░░░░░░░░░░░░░
    ░░░░░░░░░░░█░░░░░░░░▀▄░░░░░░▄░░░░░░░░░░
    ░░░░░░░░░░█░░▀░░▀░░░░░▀▄▄░░█░█░░░░░░░░░
    ░░░░░░░░░░█░▄░█▀░▄░░░░░░░▀▀░░█░░░░░░░░░
    ░░░░░░░░░░█░░▀▀▀▀░░░░░░░░░░░░█░░░░░░░░░
    ░░░░░░░░░░█░░░░░░░░░░░░░░░░░░█░░░░░░░░░
    ░░░░░░░░░░█░░░░░░░░░░░░░░░░░░█░░░░░░░░░
    ░░░░░░░░░░░█░░▄▄░░▄▄▄▄░░▄▄░░█░░░░░░░░░░
    ░░░░░░░░░░░█░▄▀█░▄▀░░█░▄▀█░▄▀░░░░░░░░░░
    ░░░░░░░░░░░░▀░░░▀░░░░░▀░░░▀░░░░░░░░░░░░
    secret dog";

    println!("{}", lesser_dog);
    println!("\n");
}