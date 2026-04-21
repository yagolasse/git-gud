// Example Rust code for syntax highlighting tests
use std::collections::HashMap;

fn main() {
    println!("Hello, world!");
    
    let mut map = HashMap::new();
    map.insert("key", "value");
    
    for (key, value) in &map {
        println!("{}: {}", key, value);
    }
    
    let result = calculate(5, 10);
    println!("Result: {}", result);
}

fn calculate(a: i32, b: i32) -> i32 {
    if a > b {
        a * 2
    } else {
        b * 3
    }
}

struct Point {
    x: f64,
    y: f64,
}

impl Point {
    fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
    
    fn distance(&self, other: &Point) -> f64 {
        ((self.x - other.x).powi(2) + (self.y - other.y).powi(2)).sqrt()
    }
}