mod rating_calculator;
mod read;

fn main() {
    let games = read::read("games.txt");
    rating_calculator::analyze_ratings(&games);
}
