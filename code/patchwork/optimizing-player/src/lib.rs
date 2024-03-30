#[allow(dead_code)]
mod lpsolve;
mod optimizing_player;

use lpsolve::SolveStatus;
pub use optimizing_player::OptimizingPlayer;

pub fn test() {
    let mut problem = lpsolve::Problem::new(0,0).unwrap();
    let solution = problem.solve();
    problem.get_objective();
    match solution {
        SolveStatus::OutOfMemory | SolveStatus::Degenerate | SolveStatus::NumericalFailure => todo!(),
        SolveStatus::NotRun => todo!(),
        SolveStatus::Infeasible | SolveStatus::Unbounded => todo!(),
        SolveStatus::Optimal | SolveStatus::Suboptimal => todo!(),
        SolveStatus::UserAbort | SolveStatus::Timeout => todo!(),
        SolveStatus::Presolved | SolveStatus::ProcFail | SolveStatus::ProcBreak => todo!(),
        SolveStatus::FeasibleFound | SolveStatus::NoFeasibleFound => todo!(),
    }
    problem.set_timeout(std::time::Duration::from_secs(10));

    // lpsolve::Problem::new(rows, cols)
}