use std::fs;
use std::{thread, time};
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
    let contents = fs::read_to_string("./codes.bshr").expect("Something went wrong reading the saved codes file");
    let delay = time::Duration::from_secs(1);
    let mut done = false;
    let mut first = true;

    if get_codes().split("\n").count() == 0
    {
        println!("Did not find any already installed songs.\nMake sure the executable is placed in \"Beat Saber/Beat Saber_data/CustomLevels\".\nIf you do not have any songs downloaded please wait a moment and the download will start...");
        thread::sleep(time::Duration::from_secs(3));
    }

    while !done
    {
        if !first
        {
            first = true;
            println!("Some songs failed... Retrying");
        }
        done = true;
        let codes = get_codes();
        for code in contents.split("\n")
        { 
            if !codes.contains(code)
            {
                done = false;
                println!("Installing {}", code);
                open::that(format!("beatsaver://{}", code)).unwrap();
                thread::sleep(delay);
            }
        }
    }
    println!("All songs added, you now have {} songs!", get_codes().split("\n").count());
    pause();
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