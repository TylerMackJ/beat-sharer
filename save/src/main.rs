use dotenv;
use reqwest;
use std::fs;
use std::io::{stdin, stdout, Read, Write};
use rand::Rng;


#[macro_use]
extern crate dotenv_codegen;

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
    dotenv::dotenv().ok();

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
            codes.push_str(",");
        }
    }

    let mut rng = rand::thread_rng();
    let uid: u8 = rng.gen();

    let client = reqwest::blocking::Client::new();
    let _res = client.put(format!("https://beat-sharer-default-rtdb.firebaseio.com/{}.json?auth={}", uid, dotenv!("secret"))).json(&codes).send().unwrap();

    if count != 0
    {
        println!("Your {} songs have been uploaded with the UID: {}, share this with your friends so they can get your songs!", count, uid);
    }
    else 
    {
        println!("You are trying to upload 0 songs. Please make sure the executable is placed inside \"Beat Saber/Beat Saber_data/CustomLevels\"!");
    }
    pause();
}