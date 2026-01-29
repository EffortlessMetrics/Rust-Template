fn main() {
    let raw = "closed";
    let key = raw.trim().to_ascii_lowercase().replace([' ', '-'], "_");
    println!("Key: '{}'", key);
    match key.as_str() {
        "done" | "closed" => println!("Done"),
        _ => println!("Todo"),
    }
}
