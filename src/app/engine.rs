use std::fs;
use std::path::Path;
use std::fmt;
use regex::Regex;
use std::os::unix::fs::MetadataExt;
use chrono::offset::Utc;
use chrono::DateTime;
use std::io;
use std::fs::File;
use std::io::BufReader;
use std::io::Read;

pub struct FileItem {
    name: String,
    is_dir: bool,
    file_owner: u32,
    file_creation_time: String,
}

pub struct ProcItem {
    pub pid: u32,
    pub cmdline: String,
    pub owner: u32,
}

impl fmt::Display for FileItem {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.is_dir {
            write!(f, " 🗂   {}", self.name)
        } else {
            write!(f, " 🗒   {}  (uid = {} created = {})", self.name, self.file_owner, self.file_creation_time)
        }
    }
}


fn get_pids(x: &String) -> Result<u32, String> {
    let re = Regex::new(r"/proc/(\d+)").unwrap();
    let m = re.captures_iter(&x[..]).next();
    match m {
        Some(t) => {
          let pid: u32 = t[1].parse().unwrap();
          Ok(pid)
        },
        None => Err("Not a proc dir".to_string())
    }
 }

fn dir_to_list(file_path: String) -> Result<Vec<String>, String> {
    let path = Path::new(&file_path);
    if !path.exists(){
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

fn get_strings_from_file(file_path: &str) -> Result<String, io::Error> {
    let f = File::open(file_path)?;
    let mut reader = BufReader::new(f);
    let mut buffer = Vec::new();
    reader.read_to_end(&mut buffer)?;
    let mut res: String = String::new();
    for value in buffer {
        if value > 33 && value < 127 {
            res.push(value as char);
        } else {
            res.push('.');
        }
    }
    Ok(res)
}


pub fn query_procs() -> Result<Vec<ProcItem>, String> {
    let proc_dirs = dir_to_list("/proc/".to_string()).unwrap();
    let mut proc_items: Vec<ProcItem> = Vec::new();
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
                let pi = ProcItem{
                    pid: z,
                    cmdline: res,
                    owner: uid 
                };
                proc_items.push(pi);
            }, 
            _ => ( ),
        }
    }
    Ok(proc_items)
}

pub fn search_proc_memory(pstr: &str) -> Result<Vec<String>, String> {
    let proc_dirs = dir_to_list("/proc/".to_string()).unwrap();
    let mut procs: Vec<String> = Vec::new();
    let mut i = 0;
    let n = proc_dirs.len();
    for pd in proc_dirs {
        let exe_dir = format!("{}/exe", pd);
        let res = get_strings_from_file(&exe_dir);
        match res {
            Ok(t) => {
                if t.contains(pstr) {
                    procs.push(exe_dir);
                }
            },
            _ => ( )
        }
        if i%100 == 0 {
            println!("{}/{} Processes checked...", i, n);
        }
        i+=1;
    }
    Ok(procs)
}

pub fn do_find_procs(pname: &str) -> Result<Vec<ProcItem>, String> {
    match query_procs() {
        Ok(p) => {
            Ok(p.into_iter().filter(|x| x.cmdline.contains(pname)).collect())
        },
        e => e,
    }
}

pub fn query_folder(file_path: &String) -> Result<Vec<FileItem>, String>  {
    let path = Path::new(&file_path);
    if !path.exists(){
        return Err("Path does not exist!".to_string());
    }

    let paths = fs::read_dir(&path).unwrap();
    let mut dirs: Vec<FileItem> = Vec::new();
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
            },
            _ => "??".to_string()
        };

        let fi = FileItem {
            name: path_str,
            is_dir: p.is_dir(),
            file_owner: md.uid(),
            file_creation_time: ct,
        };
        dirs.push(fi);
    }
    Ok(dirs)
}
