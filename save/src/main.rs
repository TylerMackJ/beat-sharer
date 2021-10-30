use std::fs;
use std::io::{stdin, stdout, Read, Write};

trait StringUtils {
    fn substring(&self, start: usize, len: usize) -> Self;
}

impl StringUtils for String {
    fn substring(&self, start: usize, len: usize) -> Self {
        self.chars().skip(start).take(len - start).collect()
    }
}

fn pause() {
    let mut stdout = stdout();
    stdout.write(b"Press Enter to continue...").unwrap();
    stdout.flush().unwrap();
    stdin().read(&mut [0]).unwrap();
}

fn main() {
    let paths = fs::read_dir("./").unwrap();
    let mut codes = String::new();
    let mut count = 0;

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
            count += 1;
            codes.push_str(&code);
            codes.push_str("\n");
        }
    }

    match fs::write("./codes.bshr", codes)
    {
        Ok(_) =>
        {
            if count == 0
            {
                println!("Found {} songs.\nMake sure to place the executable in \"Beat Saber/Beat Saber_data/CustomLevels\" so that it can find the songs!", count);
            }
            else
            {
                println!("Found {} songs!", count);
            }
            pause();
        }
        Err(_) =>
        {
            println!("Error writing to file!");
            pause();
        }
    }

}