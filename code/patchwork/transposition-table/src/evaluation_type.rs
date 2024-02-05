/// The type of node in the transposition table.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum EvaluationType {
    /// PV-Node: The value for this position is the exact evaluation
    Exact = 0,
    /// All-Node: No action during the search resulted in a position that was
    /// better than the current player could get from playing a different action
    /// in an earlier position (i.e eval was <= alpha for all actions in the
    /// position). Due to the way alpha-beta search works, the value we get here
    /// won't be the exact evaluation of the position, but rather the upper
    /// bound of the evaluation. This means that the evaluation is, at most,
    /// equal to this value.
    ///
    /// Fail-low / Alpha cut-off: The evaluation is at most this value.
    UpperBound = 1,
    /// Cut-Node: A action was found during the search that was too good,
    /// meaning the opponent will play a different move earlier on, not allowing
    /// the position where this action was available to be reached. Because the
    /// search cuts off at this point (beta cut-off), an even better action may
    /// exist. This means that the evaluation for the position could be even
    /// higher, making the stored value the lower bound of the actual value.
    ///
    /// Fail-high / Beta cut-off: The evaluation is at least this value.
    LowerBound = 2,
}
