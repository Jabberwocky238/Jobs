use std::{env, error::Error, fs::File};

use csv::WriterBuilder;

fn create_csv_in_home_dir() -> Result<(), Box<dyn Error>> {
    // 获取用户的 HOME 目录
    let home_dir = env::var("HOME").or_else(|_| env::var("USERPROFILE"))?;

    // 创建 CSV 文件的完整路径
    let file_path = format!("{}/example.csv", home_dir);

    // 创建并写入 CSV 文件
    let file = File::create(&file_path)?;
    let mut wtr = WriterBuilder::new().from_writer(file);

    // 写入标题行
    wtr.write_record(&["name", "age", "city"])?;

    // 写入数据行
    wtr.write_record(&["Alice", "24", "New York"])?;
    wtr.write_record(&["Bob", "30", "Los Angeles"])?;

    // 确保数据被写入文件
    wtr.flush()?;

    println!("CSV file created at: {}", file_path);

    Ok(())
}