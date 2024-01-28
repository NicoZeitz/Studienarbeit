pub fn handle_exit() -> ! {
    std::process::exit(0);
}

pub fn handle_exit_with_error(error: anyhow::Error) -> ! {
    println!("Error: {:?}", error);
    std::process::exit(1);
}
