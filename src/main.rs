use std::collections::HashMap;
use std::fs;
use std::fmt;
use std::io::BufReader;
use std::io::{self, BufRead, Write};
use std::time::{Duration, SystemTime};

#[derive(Debug)]
struct CasheEntity {
    value: String,
    created_at: SystemTime,
    ttl: Option<Duration>,
}
struct Cache {
    data: HashMap<String, CasheEntity>,
}
#[derive(Debug)]
enum CacheError {
    Io(io::Error),
    InvalidTTL,
    KeyNotFound,
    KeyExpired,
    ParseError(String),
}

impl fmt::Display for CacheError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CacheError::Io(err) => write!(f, "IO error: {}", err),
            CacheError::InvalidTTL => write!(f, "TTL must be a positive number"),
            CacheError::KeyNotFound => write!(f, "Key not found"),
            CacheError::ParseError(details) => write!(f, "Parse error: {}", details),
            CacheError::KeyExpired => write!(f, "Key has expired"),
        }
    }
}

impl From<io::Error> for CacheError {
    fn from(err: io::Error) -> Self {
        CacheError::Io(err)
    }
}

impl Cache {
    fn new() -> Self {
        Cache {
            data: HashMap::new(),
        }
    }

    fn set(&mut self, key: String, value: String, ttl: Option<Duration>) {
        let entity = CasheEntity {
            value,
            created_at: SystemTime::now(),
            ttl,
        };
        self.data.insert(key, entity);
    }

    fn get(&mut self, key: &str) -> Result<&String, CacheError> {
        let expired = if let Some(entity) = self.data.get(key) {
            if let Some(ttl) = entity.ttl {
                SystemTime::now()
                    .duration_since(entity.created_at)
                    .unwrap_or_default()
                    >= ttl
            } else {
                false
            }
        } else {
            return Err(CacheError::KeyNotFound);
        };

        if expired {
            self.data.remove(key);
            Err(CacheError::KeyExpired)
        } else {
            self.data.get(key).map(|entity| &entity.value).ok_or(CacheError::KeyNotFound)
        }
    }

    fn delete(&mut self, key: &str) -> Result<(), CacheError> {
        let entiry = self.data.get(key);
        if entiry.is_none() {
            Err(CacheError::KeyNotFound)
        }
        else {
            self.data.remove(key);
            Ok(())
        }
    }

    fn save(&self, path: &str) -> Result<(), CacheError> {
        let mut file = fs::File::create(path).map_err(CacheError::from)?;
        for (key, value) in &self.data {
            writeln!(file, "{}={:#?}", key, value).map_err(CacheError::from)?;
        }
        Ok(())
    }

    fn load(&mut self, path: &str) -> Result<(), CacheError> {
        let file = fs::File::open(path).map_err(CacheError::Io)?;
        let reader = BufReader::new(file);

        let mut lines = reader.lines().peekable();

        if lines.peek().is_none() {
            return Err(CacheError::ParseError("File is empty".to_string()));
        }

        for line in lines {
            let line = line.map_err(CacheError::Io)?;
            println!("{}", line);
        }
        Ok(())
    }
    fn len(&self) -> usize {
        self.data.len()
    }
}

fn main() {
    let mut cache = Cache::new();
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    const DB_PATH: &str = "db.txt";

    println!("🦀 Mini-Redis ready!");
    println!("Commands: SET <key> <value>, GET <key>, DEL <key>, SAVE, LOAD, EXIT");

    fn parse_ttl(s: &str) -> Result<Duration, CacheError> {
        let secs: u64 = s.parse().map_err(|_| CacheError::InvalidTTL)?;
        if secs == 0 {
            return Err(CacheError::InvalidTTL);
        }
        Ok(Duration::from_secs(secs))
    }

    loop {
        print!("> ");
        if let Err(e) = stdout.flush() {
            eprintln!("Не удалось вывести промпт: {}", e);
        }

        let mut input = String::new();
        stdin.lock().read_line(&mut input).unwrap();
        let input = input.trim();

        if input.is_empty() {
            continue;
        }

        let parts: Vec<&str> = input.split_whitespace().collect();
        let command = parts[0].to_uppercase();

        match command.as_str() {
            "SET" => {
                if parts.len() != 3 && parts.len() != 4 {
                    println!("Usage: SET <key> <value> [ttl_in_secs]");
                    continue;
                }
                let key = parts[1].to_string();
                let value = parts[2].to_string();
                let ttl = if parts.len() == 4 {
                    match parse_ttl(parts[3]) {
                        Ok(duration) => Some(Duration::from_secs(duration.as_secs())),
                        Err(e) => {
                            println!("{}", e);
                            continue;
                        }
                    }
                } else {
                    None
                };
                cache.set(key, value, ttl);
                println!("OK");
            }
            "GET" => {
                if parts.len() != 2 {
                    println!("Usage: GET <key>");
                    continue;
                }
                let key = parts[1];
                match cache.get(key) {
                    Ok(value) => println!("{}", value),
                    Err(CacheError::KeyNotFound) => println!("(key not found)"),
                    Err(CacheError::KeyExpired) => println!("(key expired)"),
                    Err(e) => eprintln!("Error: {}", e),
                }
            }
            "DEL" => {
                if parts.len() < 2 {
                    println!("Usage: DEL <key>");
                    continue;
                }
                let key = parts[1].to_string();
                match cache.delete(&key) {
                    Ok(()) => println!("Deleted key: {}", key),
                    Err(CacheError::KeyNotFound) => println!("(key not found)"),
                    Err(e) => eprintln!("Error: {}", e),
                }
            }
            "SAVE" => {
                match cache.save(DB_PATH) {
                    Ok(()) => println!("✅ Saved {} keys to {}", cache.len(), DB_PATH),
                    Err(CacheError::Io(e)) => eprintln!("❌ IO error: {}", e),
                    Err(CacheError::InvalidTTL) => eprintln!("❌ Invalid TTL"),
                    Err(e) => eprintln!("❌ Other error: {}", e),
                }
            }
            "LOAD" => match cache.load(DB_PATH) {
                Ok(()) => println!("Loaded {} keys from {}", cache.len(), DB_PATH),
                Err(CacheError::Io(e)) => eprintln!("IO error: {}", e),
                Err(CacheError::InvalidTTL) => eprintln!("Invalid TTL"),
                Err(CacheError::ParseError(e)) => eprintln!("Parse error: {}", e),
                Err(e) => eprintln!("Load error: {}", e),
            },
            "EXIT" => {
                println!("Bye!");
                break;
            }
            _ => println!("Unknown command"),
        }
    }
}
