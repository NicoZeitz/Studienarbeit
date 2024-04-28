pub fn get_valid_actions_for_patch(&self, patch: &'static Patch, ...) -> Vec<ActionId> {
    PatchManager::get_transformations(patch.id)
        .iter()
        .enumerate()
        .filter(|(_, transformation)| 
            self.tiles & transformation.tiles == 0)
        .map(|(patch_transformation_index, _)| 
            ActionId::patch_placement(...))
        .collect::<Vec<ActionId>>()
}
