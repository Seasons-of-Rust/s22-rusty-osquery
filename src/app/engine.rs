use chrono::DateTime;
use chrono::Utc;
use regex::Regex;
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::fmt;
use std::fs;
use std::path::Path;
//use hex::decode;

//use std::io;
//use std::fs::File;
//use std::io::BufReader;
//use std::io::Read;
use itertools::join;
use nom::{
    branch::alt,
    bytes::complete::is_not,
    bytes::complete::tag,
    character::complete::alpha1,
    character::complete::alphanumeric1,
    character::complete::{char, digit1, one_of},
    combinator::opt,
    combinator::recognize,
    combinator::rest,
    combinator::value,
    multi::many0,
    multi::separated_list0,
    sequence::delimited,
    sequence::pair,
    sequence::separated_pair,
    IResult,
};
use std::os::unix::fs::MetadataExt;

trait HasLookup {
    fn lookup(&self, attribute: &String) -> u32;
    fn lookup_str(&self, attribute: &String) -> String;
}

pub trait HasSchema {
    fn get_schema(&self) -> &BTreeMap<String, String>;
    fn get_table_body(self, cols: &Vec<String>) -> String;
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FilterOp {
    Eq,
    Leq,
    Geq,
    Like,
}

impl fmt::Display for FilterOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self {
            FilterOp::Eq => write!(f, "EQ"),
            FilterOp::Leq => write!(f, "LEQ"),
            FilterOp::Geq => write!(f, "GEQ"),
            FilterOp::Like => write!(f, "LIKE"),
        }
    }
}

pub fn match_op(input: &str) -> IResult<&str, FilterOp> {
    alt((
        value(FilterOp::Eq, tag("=")),
        value(FilterOp::Leq, tag("<=")),
        value(FilterOp::Geq, tag(">=")),
        value(FilterOp::Like, tag("like")),
    ))(input)
}

pub struct FilterItem {
    pub subject: String,
    pub op: FilterOp,
    pub target: String,
}

impl FilterItem {
    pub fn new(subject: String, op: FilterOp, target: String) -> FilterItem {
        FilterItem {
            subject: subject,
            op: op,
            target: target,
        }
    }
}

impl fmt::Display for FilterItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            format!("{} {} {}", self.subject, self.op, self.target)
        )
    }
}

pub struct FilterItems {
    pub filters: Vec<FilterItem>,
}

impl FilterItems {
    pub fn check<T: HasLookup>(&self, row: T) -> bool {
        self.filters.iter().all(|x| match x.op {
            FilterOp::Eq => {
                match x.target.parse::<u32>() {
                    Ok(z) => row.lookup(&x.subject) == z,
                    _ => row.lookup_str(&x.subject) == x.target,
                }
            },
            FilterOp::Leq => {
                match x.target.parse::<u32>() {
                    Ok(z) => row.lookup(&x.subject) <= z,
                    _ => false
                }
                
            },
            FilterOp::Geq => {
                match x.target.parse::<u32>() {
                    Ok(z) => row.lookup(&x.subject) <= z,
                    _ => false
                }
            },
            FilterOp::Like => row.lookup_str(&x.subject).contains(&x.target),
            _ => false,
        })
    }

    pub fn get_field(&mut self, field: String) -> Option<String> {
        let index = self.filters.iter().position(|x| x.subject == field);
        match index {
            Some(i) => {
                let path = self.filters[i].target.clone();
                self.filters.remove(i);
                Some(format!("{}", path))
            }
            _ => None,
        }
    }
}

impl fmt::Display for FilterItems {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut blah = String::new();
        for x in &self.filters {
            let t = format!("{}", x);
            blah.push_str(&t);
        }

        write!(f, "{}", blah)
    }
}

pub fn target_match(input: &str) -> IResult<&str, &str> {
    let (rest, m) = alt((recognize(delimited(char('"'), many0(is_not("\"")), char('"'))), recognize(many0(one_of("0123456789")))))(input)?;
    Ok((rest, m))
}

pub fn identifier(input: &str) -> IResult<&str, &str> {
    // [a-zA-Z_][a-zA-Z0-9_]*
    let (rest, m) = recognize(pair(
        alt((alpha1, tag("_"))),
        many0(alt((alphanumeric1, tag("_")))),
    ))(input)?;
    Ok((rest, m))
}

pub fn label_match(input: &str) -> IResult<&str, FilterItem> {
    let (rest, _) = opt(tag(" "))(input)?;
    let (rest, subject) = identifier(rest)?;
    let (rest, _) = opt(tag(" "))(rest)?;
    let (rest, op) = match_op(rest)?;
    let (rest, _) = opt(tag(" "))(rest)?;
    let (rest, target) = target_match(rest)?;
    let (rest, _) = opt(tag(" "))(rest)?;
    Ok((
        rest,
        FilterItem {
            subject: subject.to_string(),
            op: op,
            target: target.to_string().replace('"', ""),
        },
    ))
}

pub fn vector_selector(input: &str) -> IResult<&str, FilterItems> {
    let (rest, filters) = separated_list0(char(','), label_match)(input)?;
    Ok((rest, FilterItems { filters }))
}

pub struct FileItem {
    name: String,
    is_dir: bool,
    file_owner: u32,
    file_creation_time: String,
}

impl FileItem {
    fn to_row(self) -> String {
        format!(
            "|{}|{}|{}|{}|\n",
            self.file_creation_time, self.is_dir, self.name, self.file_owner
        )
    }
}

impl HasLookup for &FileItem {
    fn lookup(&self, attribute: &String) -> u32 {
        match attribute.as_str() {
            "uid" => self.file_owner,
            "dir" => {
                if self.is_dir {
                    1
                } else {
                    0
                }
            }
            _ => 0,
        }
    }

    fn lookup_str(&self, attribute: &String) -> String {
        match attribute.as_str() {
            "name" => self.name.clone(),
            "dir" => {
                if self.is_dir {
                    "true".to_string()
                } else {
                    "false".to_string()
                }
            }
            "uid" => format!("{}", self.file_owner).to_string(),
            "created" => self.file_creation_time.clone(),
            _ => "ERROR".to_string(),
        }
    }
}

pub struct FileTable {
    pub table: Vec<FileItem>,
    pub schema: BTreeMap<String, String>,
}

impl FileTable {
    fn new() -> FileTable {
        FileTable {
            table: Vec::new(),
            schema: BTreeMap::from([
                ("name".to_string(), "The file name".to_string()),
                ("dir".to_string(), "The file type".to_string()),
                ("uid".to_string(), "The file owner".to_string()),
                ("created".to_string(), "The file creation time".to_string()),
            ]),
        }
    }

    pub fn add_row(&mut self, item: FileItem) {
        self.table.push(item);
    }

    pub fn get_body(self, cols: &Vec<String>) -> String {
        let mut table_str = String::new();
        for x in self.table {
            if cols.len() == 0 || cols[0] == "*" {
                let s = x.to_row();
                table_str.push_str(&s);
            } else {
                let s = format!(
                    "|{}|\n",
                    cols.into_iter()
                        .map(|z| (&x).lookup_str(&z.to_string()))
                        .filter(|y| y != "")
                        .collect::<Vec<String>>()
                        .join("|")
                );
                table_str.push_str(&s);
            }
        }
        table_str
    }
}

impl HasSchema for FileTable {
    fn get_schema(&self) -> &BTreeMap<String, String> {
        &self.schema
    }

    fn get_table_body(self, cols: &Vec<String>) -> String {
        self.get_body(&cols)
    }
}

pub struct ProcMapItem {
    pub start_addr: String,
    pub end_addr: String,
    pub permissions: String,
    pub size_maybe: String,
    pub start_time: String,
    pub file_desc: u32,
    pub file_name: String,
}

impl HasLookup for &ProcMapItem {
    fn lookup(&self, attribute: &String) -> u32 {
        match attribute.as_str() {
            "file_desc" => self.file_desc,
            _ => 0,
        }
    }

    fn lookup_str(&self, attribute: &String) -> String {
        match attribute.as_str() {
            "start_addr" => self.start_addr.clone(),
            "end_addr" => self.end_addr.clone(),
            "permissions" => self.permissions.clone(),
            "size_maybe" => self.size_maybe.clone(),
            "start_time" => self.start_time.clone(),
            "file_desc" => format!("{}", self.file_desc).to_string(),
            "file_name" => self.file_name.clone(),
            _ => "X(".to_string()
        }
    }
}


impl ProcMapItem {
    pub fn to_row(self) -> String {
        format!("|{}|{}|{}|{}|{}|{}|{}|\n", self.end_addr, self.file_desc, self.file_name, self.permissions, self.size_maybe, self.start_addr, self.start_time)    
    }
}

pub struct ProcItem {
    pub pid: u32,
    pub ppid: u32,
    pub cmdline: String,
    pub owner: u32,
}

impl HasLookup for &ProcItem {
    fn lookup(&self, attribute: &String) -> u32 {
        match attribute.as_str() {
            "pid" => self.pid,
            "ppid" => self.ppid,
            "uid" => self.owner,
            _ => 0,
        }
    }

    fn lookup_str(&self, attribute: &String) -> String {
        match attribute.as_str() {
            "pid" => format!("{}", self.pid).to_string(),
            "ppid" => format!("{}", self.ppid).to_string(),
            "cmdline" => self.cmdline.clone(),
            "uid" => format!("{}", self.owner).to_string(),
            _ => "ERROR".to_string(),
        }
    }
}


pub struct NetItem {
    pub uid: u32,
    pub local_address: String,
    pub remote_address: String,
    pub st: String,
    pub timeout: u32,
    pub inode: u32,
    pub ref_cnt: u32,
    pub protocol: String,
}

impl HasLookup for &NetItem {
    fn lookup(&self, attribute: &String) -> u32 {
        match attribute.as_str() {
            "uid" => self.uid,
            "timeout" => self.timeout,
            "inode" => self.inode,
            "ref_cnt" => self.ref_cnt,
            _ => 0,
        }
    }

    fn lookup_str(&self, attribute: &String) -> String {
        match attribute.as_str() {
            "uid" => format!("{}", self.uid).to_string(),
            "timeout" => format!("{}", self.timeout).to_string(),
            "local_address" => self.local_address.clone(),
            "remote_address" => self.remote_address.clone(),
            "st" => self.st.clone(),
            "inode" => format!("{}", self.ref_cnt).to_string(),
            "ref_cnt" => format!("{}", self.ref_cnt).to_string(),
            "protocol" => self.protocol.clone(),
            _ => "ERROR".to_string(),
        }
    }
}

impl NetItem {
    pub fn to_row(self) -> String {
        format!("|{}|{}|{}|{}|{}|{}|{}|{}|\n", self.inode, self.local_address, self.protocol, self.ref_cnt, self.remote_address, self.st, self.timeout, self.uid)
    }
}

pub struct NetTable {
    pub table: Vec<NetItem>,
    pub schema: BTreeMap<String, String>,
}

impl NetTable {
    fn new() -> NetTable {
        NetTable {
            table: Vec::new(),
            schema: BTreeMap::from([
                ("inode".to_string(), "X".to_string()),
                ("local_address".to_string(), "X".to_string()),
                (
                    "protocol".to_string(),
                    "X".to_string(),
                ),
                (
                    "ref_cnt".to_string(),
                    "X".to_string(),
                ),
                (
                    "remote_address".to_string(),
                    "X".to_string(),
                ),
                (
                    "st".to_string(),
                    "X".to_string(),
                ),
                (
                    "timeout".to_string(),
                    "X".to_string(),
                ),
                (
                    "uid".to_string(),
                    "X".to_string()
                )
            ]),
        }
    }

    pub fn add_row(&mut self, item: NetItem) {
        self.table.push(item);
    }

    pub fn get_body(self, cols: &Vec<String>) -> String {
        let mut table_str = String::new();
        for x in self.table {
            if cols.len() == 0 || cols[0] == "*" {
                let s = x.to_row();
                table_str.push_str(&s);
            } else {
                let s = format!(
                    "|{}|\n",
                    cols.into_iter()
                        .map(|z| (&x).lookup_str(&z.to_string()))
                        .filter(|y| y != "")
                        .collect::<Vec<String>>()
                        .join("|")
                );
                table_str.push_str(&s);
            }
        }
        table_str
    }
}

impl HasSchema for NetTable {
    fn get_schema(&self) -> &BTreeMap<String, String> {
        &self.schema
    }

    fn get_table_body(self, cols: &Vec<String>) -> String {
        self.get_body(&cols)
    }
}



pub struct ProcMapTable {
    pub table: Vec<ProcMapItem>,
    pub schema: BTreeMap<String, String>,
}

impl ProcMapTable {
    fn new() -> ProcMapTable {
        ProcMapTable {
            table: Vec::new(),
            schema: BTreeMap::from([
                ("start_addr".to_string(), "X".to_string()),
                ("end_addr".to_string(), "X".to_string()),
                (
                    "permissions".to_string(),
                    "X".to_string(),
                ),
                (
                    "size_maybe".to_string(),
                    "X".to_string(),
                ),
                (
                    "start_time".to_string(),
                    "X".to_string(),
                ),
                (
                    "file_desc".to_string(),
                    "X".to_string(),
                ),
                (
                    "file_name".to_string(),
                    "X".to_string(),
                ),
            ]),
        }
    }

    pub fn add_row(&mut self, item: ProcMapItem) {
        self.table.push(item);
    }

    pub fn get_body(self, cols: &Vec<String>) -> String {
        let mut table_str = String::new();
        for x in self.table {
            if cols.len() == 0 || cols[0] == "*" {
                let s = x.to_row();
                table_str.push_str(&s);
            } else {
                let s = format!(
                    "|{}|\n",
                    cols.into_iter()
                        .map(|z| (&x).lookup_str(&z.to_string()))
                        .filter(|y| y != "")
                        .collect::<Vec<String>>()
                        .join("|")
                );
                table_str.push_str(&s);
            }
        }
        table_str
    }
}

impl HasSchema for ProcMapTable {
    fn get_schema(&self) -> &BTreeMap<String, String> {
        &self.schema
    }

    fn get_table_body(self, cols: &Vec<String>) -> String {
        self.get_body(&cols)
    }
}


impl ProcItem {
    pub fn to_row(self) -> String {
        format!("|{}|{}|{}|{}|\n", self.cmdline, self.pid, self.ppid, self.owner)
    }
}

pub struct ProcTable {
    pub table: Vec<ProcItem>,
    pub schema: BTreeMap<String, String>,
}

impl ProcTable {
    fn new() -> ProcTable {
        ProcTable {
            table: Vec::new(),
            schema: BTreeMap::from([
                ("pid".to_string(), "The process ID".to_string()),
                ("ppid".to_string(), "The parent process ID".to_string()),
                (
                    "uid".to_string(),
                    "The user who ran the program".to_string(),
                ),
                (
                    "cmdline".to_string(),
                    "The command used to run the program".to_string(),
                ),
            ]),
        }
    }

    pub fn add_row(&mut self, item: ProcItem) {
        self.table.push(item);
    }

    pub fn get_body(self, cols: &Vec<String>) -> String {
        let mut table_str = String::new();
        for x in self.table {
            if cols.len() == 0 || cols[0] == "*" {
                let s = x.to_row();
                table_str.push_str(&s);
            } else {
                let s = format!(
                    "|{}|\n",
                    cols.into_iter()
                        .map(|z| (&x).lookup_str(&z.to_string()))
                        .filter(|y| y != "")
                        .collect::<Vec<String>>()
                        .join("|")
                );
                table_str.push_str(&s);
            }
        }
        table_str
    }
}

impl HasSchema for ProcTable {
    fn get_schema(&self) -> &BTreeMap<String, String> {
        &self.schema
    }

    fn get_table_body(self, cols: &Vec<String>) -> String {
        self.get_body(&cols)
    }
}

pub fn export<T: HasSchema>(table: T, cols: &mut Vec<String>) -> String {
    let mut table_str = String::new();
    if cols.len() == 0 || cols[0] == "*" {
        let s = table.get_schema();
        let col_vec = join(s.keys().cloned(), "**|**");
        let n = s.len();
        let row_sep = "|:-".repeat(n);
        let header_row = format!("{}|\n|**{}**|\n{}|\n", row_sep, col_vec, row_sep);
        table_str.push_str(&header_row);
    } else {
        let s = table.get_schema();
        let filtered_cols = join(
            cols.into_iter()
                .filter(|x| s.contains_key(&x.clone() as &str)),
            "**|**",
        );
        let n = cols.len();
        let row_sep = "|:-".repeat(n);
        let header_row = format!("{}|\n|**{}**|\n{}|\n", row_sep, filtered_cols, row_sep);
        //println!("{}", header_row);
        table_str.push_str(&header_row);
    }
    let body = table.get_table_body(&cols);
    table_str.push_str(&body);
    table_str
}

fn get_pids(x: &String) -> Result<u32, String> {
    let re = Regex::new(r"/proc/(\d+)").unwrap();
    let m = re.captures_iter(&x[..]).next();
    match m {
        Some(t) => {
            let pid: u32 = t[1].parse().unwrap();
            Ok(pid)
        }
        None => Err("Not a proc dir".to_string()),
    }
}

fn dir_to_list(file_path: String) -> Result<Vec<String>, String> {
    let path = Path::new(&file_path);
    if !path.exists() {
        return Err("Path does not exist!".to_string());
    }
    let paths = fs::read_dir(&path).unwrap();
    let mut dirs: Vec<String> = Vec::new();
    for path in paths {
        let entry = path.unwrap();
        let p = entry.path();
        if p.is_dir() {
            let path_str = p.display().to_string();
            dirs.push(path_str);
        }
    }
    Ok(dirs)
}

fn read_file_to_stdout(file_path: &Path) -> String {
    let contents = fs::read_to_string(&file_path);
    match contents {
        Ok(c) => c,
        _ => "could not access".to_string(),
    }
}

pub fn query_proc_maps(cols: &mut Vec<String>, filter_str: &String) -> Result<String, String> {
    let res = vector_selector(&filter_str[..]);
    let mut filters = match res {
        Ok((_, x)) => x,
        _ => FilterItems {
            filters: Vec::new(),
        },
    };
    let mut proc_items: ProcMapTable = ProcMapTable::new();
    match filters.get_field("pid".to_string()) {
        Some(file_path) => {
            let proc_path = format!("/proc/{}/maps", file_path);
            let path = Path::new(&proc_path);
            if !path.exists() {
                return Err("Path does not exist!".to_string());
            }

            let mut res = read_file_to_stdout(&path);
            let proc_entry_re = Regex::new(r"([0-9|a-f]+)\-([0-9|a-f]+)\s(\S{4})\s([0-9|a-f]+)\s(\d+:\d+)\s(\d+)\s+(\S+)?$").unwrap();
            for l in res.lines() {
                let mut capts = proc_entry_re.captures_iter(&l[..]);
                let pm = capts.next();
                match pm {
                    Some(p) => {
                        let file_name: String = match p.get(7) {
                            Some(_) => p[7].to_string(),
                            _ => "n/a".to_string()
                        };
                        
                        let pmi = ProcMapItem {
                            start_addr: p[1].to_string(),
                            end_addr: p[2].to_string(),
                            permissions: p[3].to_string(),
                            size_maybe: p[4].to_string(),
                            start_time: p[5].to_string(),
                            file_desc: p[6].parse::<u32>().unwrap(),
                            file_name: file_name.to_string(),
                        };
                        

                        proc_items.add_row(pmi);
                    }
                    _ => (),
                }
            }
            
            Ok(export(proc_items, cols))
        },
        _ => Err("You must specify PID".to_string())
    }
    
}


pub fn query_net(cols: &mut Vec<String>, filter_str: &String) -> Result<String, String> {
    let res = vector_selector(&filter_str[..]);
    let mut filters = match res {
        Ok((_, x)) => x,
        _ => FilterItems {
            filters: Vec::new(),
        },
    };
    let mut net_items: NetTable = NetTable::new();
    
    let tcp_path = "/proc/net/tcp";
    let udp_path = "/proc/net/udp";

    let net_re = Regex::new(r"\d+:\s([0-9|A-F]+:[0-9|A-F]+)\s([0-9|A-F]+:[0-9|A-F]+)\s([0-9|A-F]+)\s[0-9|A-F]+:[0-9|A-F]+\s[0-9|A-F]+:[0-9|A-F]+\s\d+\s+(\d+)\s+(\d+)\s(\d+)\s(\d+)").unwrap();

    let path = Path::new(&tcp_path);
    if !path.exists() {
        return Err("Path does not exist!".to_string());
    }

    let mut res = read_file_to_stdout(&path);
    
    for l in res.lines() {
        let mut capts = net_re.captures_iter(&l[..]);
        let pm = capts.next();
        match pm {
            Some(p) => { 
                let ni = NetItem {
                    uid: p[4].parse::<u32>().unwrap(),
                    local_address: p[1].to_string(),
                    remote_address: p[2].to_string(),
                    st: p[3].to_string(),
                    timeout: p[5].parse::<u32>().unwrap(),
                    inode: p[6].parse::<u32>().unwrap(),
                    ref_cnt: p[7].parse::<u32>().unwrap(),
                    protocol: "tcp".to_string()
                };
                
                let f = &filters;
                let b = f.check(&ni);
                if b {
                    net_items.add_row(ni);
                    
                }
            }
            _ => (),
        }
    }
    

    let upath = Path::new(&udp_path);
    if !upath.exists() {
        return Err("Path does not exist!".to_string());
    }
    let ures = read_file_to_stdout(&upath);
    for l in ures.lines() {
        let mut capts = net_re.captures_iter(&l[..]);
        let pm = capts.next();
        match pm {
            Some(p) => { 
                let ni = NetItem {
                    uid: p[4].parse::<u32>().unwrap(),
                    local_address: p[1].to_string(),
                    remote_address: p[2].to_string(),
                    st: p[3].to_string(),
                    timeout: p[5].parse::<u32>().unwrap(),
                    inode: p[6].parse::<u32>().unwrap(),
                    ref_cnt: p[7].parse::<u32>().unwrap(),
                    protocol: "udp".to_string()
                };
                
                let f = &filters;
                let b = f.check(&ni);
                if b {
                    net_items.add_row(ni);
                }
            }
            _ => (),
        }
    }

    Ok(export(net_items, cols))
}



pub fn query_dir(cols: &mut Vec<String>, filter_str: &String) -> Result<String, String> {
    let res = vector_selector(&filter_str[..]);
    let mut filters = match res {
        Ok((_, x)) => x,
        _ => FilterItems {
            filters: Vec::new(),
        },
    };

    match filters.get_field("path".to_string()) {
        Some(file_path) => {
            let path = Path::new(&file_path);
            if !path.exists() {
                return Err("Path does not exist!".to_string());
            }

            let paths = fs::read_dir(&path).unwrap();
            let mut dirs: FileTable = FileTable::new();
            for path in paths {
                let entry = path.unwrap();
                let p = entry.path();

                let path_str = p.display().to_string();
                let md = p.metadata().expect("check metadata failed");

                //let file_type = format!("{:?}", md.file_type()).to_string();
                let ct = match md.created() {
                    Ok(t) => {
                        let datetime: DateTime<Utc> = t.into();
                        format!("{}", datetime.format("%d/%m/%Y %T")).to_string()
                    }
                    _ => "??".to_string(),
                };

                let fi = FileItem {
                    name: path_str,
                    is_dir: p.is_dir(),
                    file_owner: md.uid(),
                    file_creation_time: ct,
                };
                let f = &filters;
                let b = f.check(&fi);
                if b {
                    dirs.add_row(fi);
                }
            }
            Ok(export(dirs, cols))
        }
        _ => Err("Path not specified".to_string()),
    }
}

pub fn query_procs(cols: &mut Vec<String>, filter_str: &String) -> Result<String, String> {
    let res = vector_selector(&filter_str[..]);
    let filters = match res {
        Ok((_, x)) => x,
        _ => FilterItems {
            filters: Vec::new(),
        },
    };

    let proc_dirs = dir_to_list("/proc/".to_string()).unwrap();
    let mut proc_items: ProcTable = ProcTable::new();
    for pd in proc_dirs {
        let y = get_pids(&pd);
        match y {
            Ok(z) => {
                let cmdline_dir = format!("{}/cmdline", pd);
                let cmdpath = Path::new(&cmdline_dir);
                let mut res = read_file_to_stdout(&cmdpath);
                if res == "" {
                    let comm_dir = format!("{}/comm", pd);
                    let comm_path = Path::new(&comm_dir);
                    res = read_file_to_stdout(&comm_path);
                }
                

                let uid = match cmdpath.metadata() {
                    Ok(md) => md.uid(),
                    _ => 0,
                };
                let pi = ProcItem {
                    pid: z,
                    ppid: 0,
                    cmdline: res,
                    owner: uid,
                };
                let f = &filters;
                let b = f.check(&pi);
                if b {
                    proc_items.add_row(pi);
                }
            }
            _ => (),
        }
    }
    Ok(export(proc_items, cols))
}

pub struct OSVersion {
    pub kernel_version: String,
    pub build_id: String,
    pub gcc_version: String,
}

pub struct OSVersionTable {
    pub columns: HashMap<String, String>,
}

fn get_os_version_info() -> Result<OSVersionTable, String> {
    let path = Path::new("/proc/version");
    let res = read_file_to_stdout(&path);
    let version_re = Regex::new(r"version\s([0-9\.-]+[-[a-z]+]?)").unwrap();
    let build_re = Regex::new(r"\(([a-z\-@0-9]+)\)").unwrap();
    let gcc_re = Regex::new(r"\((gcc.+)\)").unwrap();

    let version = version_re.find(&res[..]).map(|x| x.as_str()).unwrap_or("");
    let build_id = build_re.find(&res[..]).map(|x| x.as_str()).unwrap_or("");
    let gcc_v = gcc_re.find(&res[..]).map(|x| x.as_str()).unwrap_or("");

    let os_version_table = OSVersionTable {
        columns: HashMap::from([
            ("kernel_version".to_string(), version.to_string()),
            ("build_id".to_string(), build_id.to_string()),
            ("gcc_version".to_string(), gcc_v.to_string()),
        ]),
    };

    Ok(os_version_table)
}

pub fn do_get_os_version_info(cols: &mut Vec<String>) -> Result<HashMap<String, String>, String> {
    let os_v = get_os_version_info().unwrap();
    if cols.len() == 0 || cols[0] == "*" {
        return Ok(os_v.columns);
    }
    let res: HashMap<String, String> = cols
        .into_iter()
        .filter(|x| os_v.columns.contains_key(&x as &str))
        .map(|x| (x.clone(), os_v.columns[x].clone()))
        .collect();
    Ok(res)
}
