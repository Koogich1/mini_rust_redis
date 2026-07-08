use std::collections::HashMap;
use std::fs;
// use std::io::BufReader;
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

    fn get(&mut self, key: &str) -> Option<&String> {
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
            return None;
        };

        if expired {
            self.data.remove(key);
            None
        } else {
            self.data.get(key).map(|entity| &entity.value)
        }
    }

    fn delete(&mut self, key: &str) -> bool {
        self.data.remove(key).is_some()
    }

    fn save(&self, path: &str) -> io::Result<()> {
        let mut file = fs::File::create(path)?;
        for (key, value) in &self.data {
            writeln!(file, "{}={:#?}", key, value)?;
        }
        Ok(())
    }

    // fn load(&mut self, path: &str) -> io::Result<()> {
    //     let file = fs::File::open(path)?;
    //     let reader = BufReader::new(file);
    //     for line in reader.lines() {
    //         let line = line?;
    //     }
    //     Ok(())
    // }
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
                    match parts[3].parse::<u64>() {
                        Ok(secs) => Some(Duration::from_secs(secs)),
                        Err(_) => {
                            println!("Error: TTL must be a valid positive number");
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
                    Some(value) => println!("{}", value),
                    None => println!("(nil)"),
                }
            }
            "DEL" => {
                let key = parts[1].to_string();
                cache.delete(&key);
            }
            "SAVE" => match cache.save(DB_PATH) {
                Ok(()) => println!("Saved {} keys to {}", cache.len(), DB_PATH),
                Err(e) => eprintln!("Save error: {}", e),
            },
            // "LOAD" => match cache.load(DB_PATH) {
            //     Ok(()) => println!("Loaded {} keys from {}", cache.len(), DB_PATH),
            //     Err(e) => eprintln!("Load error: {}", e),
            // },
            "EXIT" => {
                println!("Bye!");
                break;
            }
            _ => println!("Unknown command"),
        }
    }
}
