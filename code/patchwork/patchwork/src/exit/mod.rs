pub fn handle_exit(code: i32) -> ! {
    std::process::exit(code);
}

pub fn handle_exit_with_error(error: anyhow::Error) -> ! {
    println!("Error: {:?}", error);
    std::process::exit(1);
}
