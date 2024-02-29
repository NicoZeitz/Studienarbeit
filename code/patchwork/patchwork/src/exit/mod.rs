use patchwork_lib::PatchworkError;

pub fn handle_exit(code: i32) -> ! {
    std::process::exit(code);
}

pub fn handle_exit_with_error(error: anyhow::Error) -> ! {
    if error.is::<PatchworkError>() {
        let error = error.downcast::<PatchworkError>().unwrap();
        println!("{error:?}");
        match error {
            PatchworkError::InvalidActionError { reason, action, state } => {
                println!("Reason: {reason}");
                println!("Action: {action:?}");
                println!("State: {state:?}");
                std::process::exit(1);
            }
            PatchworkError::GameStateIsInitialError => {
                std::process::exit(1);
            }
            PatchworkError::InvalidNotationError { notation, reason } => {
                println!("Notation: {notation}");
                println!("Reason: {reason}");
                std::process::exit(1);
            }
            PatchworkError::InvalidRangeError { reason } => {
                println!("Reason: {reason}");
                std::process::exit(1);
            }
        }
    }

    println!("Error: {error:?}");
    std::process::exit(1);
}
