use std::{
    cell::RefCell,
    hash::{DefaultHasher, Hash, Hasher},
    io::Write,
    rc::Rc,
};

use patchwork_core::Patchwork;

pub struct SearchRecorderNode {
    pub state: Patchwork,
    pub value: Option<i32>,
    pub alpha: Option<i32>,
    pub beta: Option<i32>,
    pub description: Option<String>,
    pub children: Vec<Rc<RefCell<SearchRecorderNode>>>,
}

pub struct SearchRecorder<const ENABLED: bool = false> {
    pub index: usize,
    pub root: Option<Rc<RefCell<SearchRecorderNode>>>,
    pub current_nodes: Vec<Rc<RefCell<SearchRecorderNode>>>,
}

impl<const ENABLED: bool> SearchRecorder<ENABLED> {
    pub const ENABLED: bool = ENABLED;

    #[allow(unused)]
    pub fn new() -> Self {
        Self {
            index: 1,
            root: None,
            current_nodes: vec![],
        }
    }

    #[allow(unused)]
    pub fn push_state(&mut self, state: Patchwork) {
        if !Self::ENABLED {
            return;
        }

        let node = Rc::new(RefCell::new(SearchRecorderNode {
            state,
            value: None,
            alpha: None,
            beta: None,
            description: None,
            children: vec![],
        }));

        if self.current_nodes.is_empty() {
            self.root = Some(Rc::clone(&node));
        } else {
            let last_node = self.current_nodes.last().unwrap();
            RefCell::borrow_mut(last_node).children.push(Rc::clone(&node));
        }
        self.current_nodes.push(node);
    }

    #[allow(unused)]
    pub fn pop_state_with_value(&mut self, value: i32, alpha: i32, beta: i32, description: &str) {
        if !Self::ENABLED {
            return;
        }

        let last_node = self.current_nodes.pop().unwrap();
        RefCell::borrow_mut(&last_node).value = Some(value);
        RefCell::borrow_mut(&last_node).alpha = Some(alpha);
        RefCell::borrow_mut(&last_node).beta = Some(beta);
        RefCell::borrow_mut(&last_node).description = Some(description.to_string());
    }

    #[allow(unused)]
    pub fn print_to_file(&mut self) {
        if !Self::ENABLED {
            return;
        }

        let file = std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(format!("search_tree_{:04}.txt", self.index))
            .unwrap();

        self.index += 1;

        let mut writer = std::io::BufWriter::new(file);
        self.print_to_file_recursive(&self.root, 0, &mut writer, "");
    }

    #[allow(clippy::only_used_in_recursion)]
    fn print_to_file_recursive(
        &self,
        node: &Option<Rc<RefCell<SearchRecorderNode>>>,
        depth: usize,
        writer: &mut std::io::BufWriter<std::fs::File>,
        padding: &str,
    ) {
        if let Some(node) = node {
            let node = RefCell::borrow(node);

            writeln!(
                writer,
                "{padding}Value: {:4?}, Text: {:?}, Alpha: {:?}, Beta: {:?}, Depth: {depth:?}, Player: {:?}", // , State: {:?}",
                node.value.unwrap(),
                node.description.as_ref().unwrap(),
                node.alpha.unwrap(),
                node.beta.unwrap(),
                node.state.get_current_player(),
                // node.state
            )
            .unwrap();

            for i in 0..node.children.len() {
                let child = &node.children[i];

                if i < node.children.len() - 1 {
                    let mut hasher = DefaultHasher::new();
                    RefCell::borrow(child).state.hash(&mut hasher);
                    let hash1 = hasher.finish();

                    let next_child = &node.children[i + 1];

                    let mut hasher = DefaultHasher::new();
                    RefCell::borrow(next_child).state.hash(&mut hasher);
                    let hash2 = hasher.finish();

                    if hash1 == hash2 {
                        self.print_to_file_recursive(
                            &Some(Rc::clone(child)),
                            depth + 1,
                            writer,
                            format!("{padding}    ↓   ").as_str(),
                        );
                        continue;
                    }
                }

                self.print_to_file_recursive(
                    &Some(Rc::clone(child)),
                    depth + 1,
                    writer,
                    format!("{padding}    ").as_str(),
                );
            }
        }
    }
}

impl Default for SearchRecorder {
    fn default() -> Self {
        Self::new()
    }
}
