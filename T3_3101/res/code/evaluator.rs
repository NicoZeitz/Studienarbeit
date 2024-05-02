pub trait Evaluator: Sync {
    fn evaluate_intermediate_node(&self, game: &Patchwork) -> i32;

    fn evaluate_terminal_node(&self, game: &Patchwork) -> i32 {
        match game.get_termination_result().termination {
            TerminationType::Player1Won => POSITIVE_INFINITY,
            TerminationType::Player2Won => NEGATIVE_INFINITY,
        }
    }

    fn evaluate_node(&self, game: &Patchwork) -> i32 {
        if game.is_terminated() {
            self.evaluate_terminal_node(game)
        } else {
            self.evaluate_intermediate_node(game)
        }
    }
}