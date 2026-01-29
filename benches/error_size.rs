use app_http::errors::AppError;
use std::mem::size_of;

fn main() {
    println!("Size of AppError: {} bytes", size_of::<AppError>());
}
