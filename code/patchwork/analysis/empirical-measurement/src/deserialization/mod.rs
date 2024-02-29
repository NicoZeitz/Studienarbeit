use std::num::NonZeroUsize;

use patchwork_core::{ActionId, Patchwork};

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct Game {
    pub turns: Vec<GameTurn>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct GameTurn {
    pub state: Patchwork,
    pub action: Option<ActionId>,
}

pub struct GameLoader {
    rx: std::sync::mpsc::Receiver<Game>,
    loaded: std::sync::Arc<std::sync::atomic::AtomicUsize>,
}

impl GameLoader {
    pub(crate) fn new(path: &std::path::PathBuf, parallelism: Option<NonZeroUsize>) -> Self {
        let dir = std::fs::read_dir(path).unwrap();
        let parallelism = parallelism.unwrap_or_else(|| std::thread::available_parallelism().unwrap());
        let (tx, rx) = std::sync::mpsc::channel();
        let loaded = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));

        let thread_pool = rayon::ThreadPoolBuilder::new()
            .num_threads(parallelism.get())
            .build()
            .unwrap();

        for entry in dir {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.is_dir() {
                continue;
            }

            let tx = tx.clone();
            let loaded = loaded.clone();
            thread_pool.spawn(move || {
                while loaded.load(std::sync::atomic::Ordering::Relaxed) > (parallelism.get() + 2) * 10_000 {
                    std::thread::sleep(std::time::Duration::from_millis(100));
                    std::thread::yield_now();
                }

                let bytes = std::fs::read(path).unwrap();
                let game_chunks: Vec<Game> = bincode::deserialize(&bytes).unwrap();
                for game_chunk in game_chunks {
                    loaded.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    if tx.send(game_chunk).is_err() {
                        break;
                    };
                }
            });
        }

        Self { rx, loaded }
    }
}

impl Iterator for GameLoader {
    type Item = Game;

    fn next(&mut self) -> Option<Self::Item> {
        let item = self.rx.recv().ok();
        self.loaded.fetch_sub(1, std::sync::atomic::Ordering::Relaxed);
        item
    }
}
