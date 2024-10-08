use std::env;
use std::error::Error;
use std::fs;
use std::io::Write;
use std::path::PathBuf;

use crossterm::cursor;
use crossterm::event::read;
use crossterm::event::Event;
use crossterm::event::KeyCode;
use crossterm::terminal::Clear;
use crossterm::terminal::ClearType;
use crossterm::Command;

use crate::JManager;
use crate::JNodeAction;
use crate::ManagerAction;
use crate::ManagerStorage;

pub struct Console {
    pub manager: JManager,
    pub current: PathBuf,
}

const TREE_INDENT: usize = 4;

impl Console {
    pub fn new() -> Self {
        let current = std::env::current_dir().unwrap();
        let manager = JManager::new();
        Self { manager, current }
    }
    pub fn prompt(&self) -> String {
        format!("[Jobs]@{} >> ", self.current.display())
    }
    pub fn exec(&mut self, raw_cmd: &str) -> Result<(), Box<dyn Error>> {
        let mut args = raw_cmd.split_whitespace();
        let cmd = args.next().unwrap();
        
        let home_dir = env::var("HOME").or_else(|_| env::var("USERPROFILE"))?;
        let file_path = PathBuf::from(home_dir).join("example.csv");
        match cmd {
            "cd" => {
                let path = parse_cd(&raw_cmd.to_owned())?;
                self.cd(&path)
            }
            "ls" => self.ls(),
            "scan" => self.scan(),
            "show" => self.show(),
            "tree" => {
                let depth = args.next().unwrap_or("3").parse::<usize>().unwrap();
                self.tree(depth)
            }
            "dump" => self.manager.dump(&file_path),
            "load" => self.manager.load(&file_path),
            _ => Err("Unknown command".into()),
        }
    }
    pub fn cd(&mut self, path: &str) -> Result<(), Box<dyn Error>> {
        let path = path[1..path.len() - 1].to_string();
        let to = to_absolute(&self.current, &PathBuf::from(path));
        if to.is_dir() {
            self.current = to;
            self.manager.locate_node(&self.current)?;
            Ok(())
        } else {
            Err("Not a directory".into())
        }
    }
    pub fn ls(&self) -> Result<(), Box<dyn Error>> {
        let mut result = String::new();
        let mut cnt = 0;
        let infos = fs::read_dir(&self.current)
            .unwrap()
            .filter(|v| v.is_ok())
            .map(|v| v.unwrap())
            .map(|v| {
                (
                    v.file_name().to_str().unwrap().to_string(),
                    v.path().is_dir(),
                )
            })
            .collect::<Vec<_>>();
        for info in infos.iter().filter(|(_, is)| *is) {
            result.push_str(&format!("{}/", info.0));
            cnt += 1;
            if cnt % 1 == 0 {
                result.push('\n');
            }
        }
        for info in infos.iter().filter(|(_, is)| !*is) {
            result.push_str(&format!("{}", info.0));
            cnt += 1;
            if cnt % 1 == 0 {
                result.push('\n');
            }
        }
        println!("{result}");
        Ok(())
    }
    pub fn scan(&mut self) -> Result<(), Box<dyn Error>> {
        let h: u64 = self.manager.locate_node(&self.current)?;
        self.manager.update_node(&h)?;
        Ok(())
    }
    pub fn show(&mut self) -> Result<(), Box<dyn Error>> {
        let h: u64 = self.manager.locate_node(&self.current)?;
        let info = self.manager.get_info(&h)?;
        println!("{}", info);
        Ok(())
    }
    pub fn tree(&mut self, depth: usize) -> Result<(), Box<dyn Error>> {
        let indent = format!("|{}", " ".repeat(TREE_INDENT));
        let mut chs = vec![];
        let h = self.manager.locate_node(&self.current)?;
        chs.push((h, 0));
        while let Some((h, d)) = chs.pop() {
            if d < depth {
                let children = self.manager.get_children_node(&h);
                // 先打印文件夹，再打印文件
                for (child, h) in children.iter() {
                    if child.is_dir() {
                        chs.push((*h, d + 1));
                    }
                }
                for (child, h) in children.iter() {
                    if !child.is_dir() {
                        chs.push((*h, d + 1));
                    }
                }
            }
            let info = self.manager.get_info(&h)?;
            println!(
                "{}{}",
                indent.repeat(d),
                info.name() + (if info.is_dir() { "/" } else { "" }),
            );
        }
        Ok(())
    }
}

pub fn run() -> Result<(), Box<dyn Error>> {
    let mut console = Console::new();
    let mut history: Vec<String> = vec![];
    let mut buffer = String::new(); // 用于存储按键输入的缓冲区
    let mut arrow_cursor = history.len();
    println!("Press up or down arrow key, 'q' to quit.");
    print!("{}", console.prompt());
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
                        print!("\r{}", " ".repeat(50));
                        print!("\r{}{buffer}", console.prompt());
                    }
                    KeyCode::Down => {
                        if arrow_cursor < history.len() - 1 {
                            arrow_cursor += 1;
                        }
                        match history.get(arrow_cursor) {
                            Some(value) => buffer = value.clone(),
                            None => buffer.clear(),
                        }
                        print!("\r{}", " ".repeat(50));
                        print!("\r{}{buffer}", console.prompt());
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
                            console.exec(&buffer)?;
                            buffer.clear();
                        }
                        print!("\n{}", console.prompt());
                    }
                    KeyCode::Tab => {
                        buffer.clear();
                        println!();
                        console.exec(&"ls")?;
                        print!("\n{}", console.prompt());
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

#[inline]
fn to_absolute(current: &PathBuf, path: &PathBuf) -> PathBuf {
    let path = path.canonicalize().unwrap();
    dbg!(&path);
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        current.join(path).canonicalize().unwrap()
    }
}

fn parse_cd(buffer: &String) -> Result<String, std::io::Error> {
    let buffer = buffer[3..].trim().to_string();
    let mut path = String::new();
    let mut yinhao = false;
    for c in buffer.chars() {
        if c == '"' {
            yinhao = !yinhao;
        }
        if yinhao {
            path.push(c);
        } else {
            if c.is_whitespace() {
                break;
            } else {
                path.push(c);
            }
        }
    }
    Ok(path)
}