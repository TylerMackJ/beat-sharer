use std::fs;
use std::{thread, time};

trait StringUtils {
    fn substring(&self, start: usize, len: usize) -> Self;
}

impl StringUtils for String {
    fn substring(&self, start: usize, len: usize) -> Self {
        self.chars().skip(start).take(len - start).collect()
    }
}

fn main() {
    let contents = fs::read_to_string("./codes.bshr").expect("Something went wrong reading the saved codes file");
    let delay = time::Duration::from_secs(1);
    let codes = get_codes();
    for code in contents.split("\n")
    { 
        if !codes.contains(code)
        {
            open::that(format!("beatsaver://{}", code)).unwrap();
            thread::sleep(delay);
        }
    }
}

fn get_codes() -> String {
    let paths = fs::read_dir("./").unwrap();
    let mut codes = String::new();

    for path in paths {
        let filename = path.unwrap().path();

        let end = match filename.to_str().unwrap().find(" (")
        {
            Some(t) => {t},
            None => {continue},
        };
        
        let code = filename.to_str().unwrap().to_string().substring(2, end);

        if code.chars().count() <= 5
        {
            codes.push_str(&code);
            codes.push_str("\n");
        }
    }
    return codes
}