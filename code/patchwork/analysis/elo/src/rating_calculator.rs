use std::{
    collections::HashMap,
    fmt::{self, Debug, Formatter},
};

use skillratings::{
    glicko2::{glicko2_rating_period, Glicko2Config, Glicko2Rating},
    Outcomes,
};

#[derive(Clone)]
pub struct Player {
    pub rating: Glicko2Rating,
    pub name: String,
}

impl Debug for Player {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("Player")
            .field("rating", &self.rating.rating)
            .field("d", &self.rating.deviation)
            .field("v", &self.rating.volatility)
            .field("name", &self.name)
            .finish()
    }
}

pub fn analyze_ratings(games: &HashMap<String, Vec<(String, Outcomes)>>) {
    let players = games
        .keys()
        .map(|name| Player {
            rating: Glicko2Rating::default(),
            name: name.to_string(),
        })
        .collect::<Vec<Player>>();

    let mut new_players = vec![];

    for player in &players {
        let results = games
            .get(&player.name)
            .unwrap()
            .iter()
            .map(|(opponent, outcome)| (players.iter().find(|p| p.name == *opponent).unwrap().rating, *outcome))
            .collect::<Vec<(Glicko2Rating, Outcomes)>>();

        let new_rating = glicko2_rating_period(&player.rating, &results, &Glicko2Config::new());

        let new_player = Player {
            rating: new_rating,
            name: player.name.clone(),
        };

        new_players.push(new_player);
    }

    new_players.sort_by_key(|p| (p.rating.rating * -100_000.0) as i64);
    for player in new_players {
        println!("{player:?}");
    }
}
