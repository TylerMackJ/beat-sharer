use std::fs;

trait StringUtils {
    fn substring(&self, start: usize, len: usize) -> Self;
}

impl StringUtils for String {
    fn substring(&self, start: usize, len: usize) -> Self {
        self.chars().skip(start).take(len - start).collect()
    }
}

fn main() {
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

    fs::write("./codes.bshr", codes).expect("Unable to write file");
}