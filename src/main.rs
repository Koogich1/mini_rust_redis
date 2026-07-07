use std::collections::HashMap;
use std::fs;
use std::io::BufReader;
use std::io::{self, BufRead, Write};

struct Cache {
    data: HashMap<String, String>,
}

impl Cache {
    fn new() -> Self {
        Cache {
            data: HashMap::new(),
        }
    }

    fn set(&mut self, key: String, value: String) {
        self.data.insert(key, value);
    }

    fn get(&self, key: &str) -> Option<&String> {
        self.data.get(key)
    }

    fn delete(&mut self, key: &str) -> bool {
        self.data.remove(key).is_some()
    }

    fn save(&self, path: &str) -> io::Result<()> {
        let mut file = fs::File::create(path)?;
        for (key, value) in &self.data {
            writeln!(file, "{}={}", key, value)?;
        }
        Ok(())
    }

    fn load(&mut self, path: &str) -> io::Result<()> {
        let file = fs::File::open(path)?;
        let reader = BufReader::new(file);
        for line in reader.lines() {
            let line = line?;
            if let Some((key, value)) = line.split_once('=') {
                self.data.insert(key.to_string(), value.to_string());
            }
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

    // REPL: Read-Eval-Print Loop
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

        // Разбиваем команду на части
        let parts: Vec<&str> = input.split_whitespace().collect();
        let command = parts[0].to_uppercase();

        match command.as_str() {
            "SET" => {
                if parts.len() != 3 {
                    println!("Usage: SET <key> <value>");
                    continue;
                }
                let key = parts[1].to_string();
                let value = parts[2].to_string();
                cache.set(key, value);
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
            "LOAD" => match cache.load(DB_PATH) {
                Ok(()) => println!("Loaded {} keys from {}", cache.len(), DB_PATH),
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
