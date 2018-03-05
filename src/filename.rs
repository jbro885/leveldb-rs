use regex::Regex;

pub enum FileType<'a> {
    Log(&'a str, u64),
    Current(&'a str),
    Table(&'a str, u64),
}

lazy_static!{
    static ref CURRENT_FILE_REGEX: Regex = {
        Regex::new(r"([\w]+)/CURRENT").unwrap()
    };

    static ref LOG_FILE_REGEX: Regex = {
        Regex::new(r"([\w]+)/([\d]{7})\.log").unwrap()
    };

    static ref TABLE_FILE_REGEX: Regex = {
        Regex::new(r"([\w]+)/([\d]{7})\.ldb").unwrap()
    };
}

impl<'a> FileType<'a> {
    pub fn parse_name(filename: &'a str) -> Self {
        if CURRENT_FILE_REGEX.is_match(filename) {
            let v = CURRENT_FILE_REGEX.captures(filename).unwrap();
            FileType::Current(v.get(0).unwrap().as_str())
        } else if LOG_FILE_REGEX.is_match(filename) {
            let v = LOG_FILE_REGEX.captures(filename).unwrap();
            let num = v.get(1).unwrap().as_str(); // TODO
            FileType::Log(v.get(0).unwrap().as_str(), 000)
        } else if TABLE_FILE_REGEX.is_match(filename) {
            let v = TABLE_FILE_REGEX.captures(filename).unwrap();
            let num = v.get(1).unwrap().as_str(); // XXX
            FileType::Table(v.get(0).unwrap().as_str(), 000)
        } else {
            unimplemented!()
        }
    }

    pub fn is_logfile(&self) -> bool {
        match self {
            &FileType::Log(_, _) => true,
            _ => false,
        }

    }

    pub fn filename(&self) -> String {
        match self {
            &FileType::Log(name, num) => format!("{:}/{:07}.log", name, num),
            &FileType::Current(name) => format!("{:}/CURRENT", name),
            &FileType::Table(name, num) => format!("{:}/{:07}.ldb", name, num),
        }
    }
}
