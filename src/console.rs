use std::error::Error;
use std::io::Write;

use crossterm::cursor;
use crossterm::event::read;
use crossterm::event::Event;
use crossterm::event::KeyCode;
use crossterm::terminal::Clear;
use crossterm::terminal::ClearType;
use crossterm::Command;

// struct Console<'a> {
//     // root: DirInfo,
//     // current: &'a mut DirInfo,
// }

// impl<'a> Console<'a> {
//     pub fn new() -> Self {
//         let current_path = std::env::current_dir().unwrap();
//         let split = current_path
//             .iter()
//             .map(|x| x.to_str().unwrap().to_string())
//             .collect::<Vec<_>>();
//         let mut split = split
//             .iter()
//             .enumerate()
//             .map(|(i, v)| Path::new(&split[..=i].join(&"/")).to_owned())
//             .map(|p| DirInfo::new(&p))
//             .collect::<Vec<_>>();

//         let mut hasher: DefaultHasher = DefaultHasher::new();

//         for i in 0..split.len() - 1 {
//             split[i + 1].hash(&mut hasher);
//             let child_hash = hasher.finish();
//             split[i].children.insert(child_hash, JNode::DirInfo(split[i + 1]));
//         }
        
//         let root = split.remove(0);
        
        
//         Self {
//             root,
//             current,
//         }
//     }
// }

pub fn run() -> Result<(), Box<dyn Error>> {
    let mut history: Vec<String> = vec![];
    let mut buffer = String::new(); // 用于存储按键输入的缓冲区
    let mut arrow_cursor = history.len();
    let mut prompt = String::from("0>> ");
    println!("Press up or down arrow key, 'q' to quit.");
    print!("{prompt}");
    std::io::stdout().flush()?;

    loop {
        match read()? {
            Event::Key(event) => {
                // println!("{:?}", event);
                match event.code {
                    KeyCode::Up => {
                        if arrow_cursor > 0 {
                            arrow_cursor -= 1;
                        }
                        match history.get(arrow_cursor) {
                            Some(value) => buffer = value.clone(),
                            None => buffer.clear(),
                        }
                        print!("\r{prompt}{buffer}{}", " ".repeat(20));
                    }
                    KeyCode::Down => {
                        if arrow_cursor < history.len() - 1 {
                            arrow_cursor += 1;
                        }
                        match history.get(arrow_cursor) {
                            Some(value) => buffer = value.clone(),
                            None => buffer.clear(),
                        }
                        print!("\r{prompt}{buffer}{}", " ".repeat(20));
                    }
                    KeyCode::Enter => {
                        if !buffer.is_empty() {
                            println!("\nYou entered: [{}]", buffer);
                            if buffer == "quit" || buffer == "q" {
                                println!("Gracefully Exiting...");
                                break;
                            }
                            history.push(buffer.clone());
                            arrow_cursor += 1;
                            prompt = format!("{}>> ", arrow_cursor);
                            buffer.clear();
                        }
                        print!("\n{prompt}");
                    }
                    KeyCode::Backspace => {
                        if !buffer.is_empty() {
                            buffer.pop();
                            cursor::MoveLeft(1).execute_winapi()?;
                            Clear(ClearType::UntilNewLine).execute_winapi()?;
                        }
                    }
                    KeyCode::Char(c) => {
                        print!("{c}");
                        buffer.push(c);
                    }
                    _ => {
                        println!("{:?}", event);
                    }
                }
            }
            Event::Mouse(event) => println!("{:?}", event),
            Event::Resize(width, height) => println!("New size {}x{}", width, height),
        }
        std::io::stdout().flush()?;
    }
    Ok(())
}
