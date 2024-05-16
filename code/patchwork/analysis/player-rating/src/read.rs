use std::{collections::HashMap, path::Path};

use regex::Regex;
use skillratings::Outcomes;

pub fn read<P: AsRef<Path>>(path: P) -> HashMap<String, Vec<(String, Outcomes)>> {
    println!("Reading from {:?}", path.as_ref());

    let contents = std::fs::read_to_string(path).expect("Something went wrong reading the file");

    let mut games = HashMap::new();
    let mut ignored = 0;
    let mut loaded = 0;

    for chunk in contents.split('\n').collect::<Vec<&str>>().chunks(6) {
        if chunk.len() < 4 {
            ignored += 1;
            continue;
        }

        let Some(player_1) = Regex::new("\\[White \"(?<name>.+)\"\\]")
            .unwrap()
            .captures(chunk[1])
            .and_then(|o| o.name("name"))
            .map(|o| o.as_str())
        else {
            ignored += 1;
            continue;
        };
        let Some(player_2) = Regex::new("\\[Black \"(?<name>.+)\"\\]")
            .unwrap()
            .captures(chunk[2])
            .and_then(|o| o.name("name"))
            .map(|o| o.as_str())
        else {
            ignored += 1;
            continue;
        };
        let Some(result) = Regex::new("\\[Result \"(?<result>1\\-0|0\\-1)\"\\]")
            .unwrap()
            .captures(chunk[3])
            .and_then(|o| o.name("result"))
            .map(|o| o.as_str())
        else {
            ignored += 1;
            continue;
        };

        loaded += 1;
        if games.get(player_1).is_none() {
            games.insert(player_1.to_string(), vec![]);
        }
        games.get_mut(player_1).unwrap().push((
            player_2.to_string(),
            if result == "1-0" { Outcomes::WIN } else { Outcomes::LOSS },
        ));

        if games.get(player_2).is_none() {
            games.insert(player_2.to_string(), vec![]);
        }
        games.get_mut(player_2).unwrap().push((
            player_1.to_string(),
            if result == "0-1" { Outcomes::WIN } else { Outcomes::LOSS },
        ));
    }

    println!("Loaded {loaded} games, ignored {ignored} unknown");

    games
}
