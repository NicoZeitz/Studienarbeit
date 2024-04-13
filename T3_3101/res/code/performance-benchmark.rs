fn game_get_initial_state(c: &mut Criterion) {
  c.bench_function("game.get_initial_state", |b| {
    let mut random = Xoshiro256PlusPlus::seed_from_u64(42);
    b.iter_with_setup(
      || Some(GameOptions {
        seed: random.next_u64(),
      }),
      |args| black_box(Patchwork::get_initial_state(args))
    );});}

fn game_get_valid_actions(c: &mut Criterion) {
  c.bench_function("game.get_valid_actions", |b| {
    let mut random = Xoshiro256PlusPlus::seed_from_u64(42);
    b.iter_with_setup(
      || Patchwork::get_initial_state(Some(GameOptions {
        seed: random.next_u64()
      })),
      |game| black_box(game.get_valid_actions()),
    );});}

fn game_get_random_action(c: &mut Criterion) {
  c.bench_function("game.get_random_action", |b| {
    let mut random = Xoshiro256PlusPlus::seed_from_u64(42);
    b.iter_with_setup(
      || { /* as before */ },
      |game| black_box(game.get_random_action())
    );});}

fn game_do_action(c: &mut Criterion) {
  c.bench_function("game.do_action", |b| {
    let mut random = Xoshiro256PlusPlus::seed_from_u64(42);
    b.iter_with_setup(
      || {
        let mut game = Patchwork::get_initial_state(Some(
          GameOptions { seed: random.next_u64() }
        ));
        for _ in 0..(seed % 25) {
          game.do_action(game.get_random_action(), false).unwrap();
        }
        let action = game.get_random_action();
        (game, action)
      },
      |(mut game, action)| black_box(game.do_action(action, false))
    );});}

fn game_undo_action(c: &mut Criterion) {
  c.bench_function("game.undo_action", |b| {
    let mut random = Xoshiro256PlusPlus::seed_from_u64(42);
    b.iter_with_setup(
      || {
        // as before
        game.do_action(action, false).unwrap();
        (game, action)
      },
      |(mut game, action)| black_box(game.undo_action(action, false))
    );});}

fn game_clone(c: &mut Criterion) {
  c.bench_function("game.clone", |b| {
    let mut random = Xoshiro256PlusPlus::seed_from_u64(42);
    b.iter_with_setup(
      || {
        let mut game = Patchwork::get_initial_state(Some(
          GameOptions { seed: random.next_u64() }
        ));
        for _ in 0..(seed % 25) {
            game.do_action(game.get_random_action(), false).unwrap();
        }
        game
      },
      |game| black_box(game.clone())
    );});}

fn game_is_terminated(c: &mut Criterion) {
  let mut random = Xoshiro256PlusPlus::seed_from_u64(42);
  c.bench_function("game.is_terminated", |b| {
    b.iter_with_setup(
      || { /* as before */ },
      |game| black_box(game.is_terminated())
    );});}

fn action_id_from_natural_action_id(c: &mut Criterion) {
  let mut random = Xoshiro256PlusPlus::seed_from_u64(10);
  c.bench_function("action_id.from_natural_action_id", |b| {
    b.iter_with_setup(
      || {
        match random.gen_range(0..5) {
          0 => Action::Walking { starting_index: 42 },
          1 => Action::SpecialPatchPlacement {
            quilt_board_index: 42,
          },
          2 => Action::PatchPlacement {
            patch_id: 12,
            patch_index: 0,
            patch_transformation_index: 0,
            previous_player_was_1: true,
          },
          3 => Action::Phantom,
          4 => Action::Null,
        }.to_natural_action_id()
      },
      |action_id| black_box(ActionId::from_natural_action_id(action_id))
    );});}

fn natural_action_id_from_surrogate_action_id(c: &mut Criterion) {
  let mut random = Xoshiro256PlusPlus::seed_from_u64(10);
  c.bench_function("natural_action_id.from_surrogate_action_id", |b| {
    b.iter_with_setup(
      || {
        match random.gen_range(0..5) { /* ... as before ... */ } .to_surrogate_action_id()
      },
      |action_id| black_box(NaturalActionId::from_surrogate_action_id(
        action_id
      )));});}

fn patch_manager_get_patch(c: &mut Criterion) {
  c.bench_function("patch_manager.get_patch", |b| {
    b.iter(|| black_box(PatchManager::get_patch(12)));
  });}

fn patch_manager_get_special_patch(c: &mut Criterion) {
  c.bench_function("patch_manager.get_special_patch", |b| {
    b.iter(|| black_box(PatchManager::get_special_patch(44)));
  });}

fn patch_manager_get_transformation(c: &mut Criterion) {
  c.bench_function("patch_manager.get_transformation", |b| {
    b.iter(|| black_box(PatchManager::get_transformation(12, 44)));
  });}

fn player_get_position(c: &mut Criterion) {
  let mut random = Xoshiro256PlusPlus::seed_from_u64(42);
  c.bench_function("player.get_position", |b| {
    b.iter_with_setup(
      || PlayerState::new(
        position: random.gen::<u8>(),
        PlayerState::STARTING_BUTTON_BALANCE,
        QuiltBoard::new(),
      ),
      |player| black_box(player.get_position())
    );});}

fn quilt_board_is_full(c: &mut Criterion) {
  let quilt_board = QuiltBoard::default();
  c.bench_function("quilt_board.is_full", |b| {
    b.iter(|| black_box(quilt_board.is_full()));
  });}
  
fn quilt_board_is_special_tile_condition_reached(c: &mut Criterion) {
  let quilt_board = QuiltBoard::default();
  c.bench_function("...", |b| b.iter(||
    black_box(quilt_board.is_special_tile_condition_reached()))
  );}

fn quilt_board_do_action(c: &mut Criterion) {
  let action = QuiltBoard::default().get_valid_actions_for_patch(
    PatchManager::get_patch(12), 0, true)[0];
  c.bench_function("quilt_board.do_action", |b| {
    b.iter_with_setup(QuiltBoard::default, |mut quilt_board| {
      black_box(quilt_board.do_action(action));
    });});}

fn quilt_board_undo_action(c: &mut Criterion) {
  let action = QuiltBoard::default().get_valid_actions_for_patch(
    PatchManager::get_patch(12), 0, true)[0];
  let mut quilt_board = QuiltBoard::default();
  quilt_board.do_action(action);
  c.bench_function("quilt_board.undo_action", |b| {
    b.iter_with_setup(
      || quilt_board.clone(),
      |mut quilt_board| black_box(quilt_board.undo_action(action))
    );});}

fn quilt_board_get_valid_actions_for_patch(c: &mut Criterion) {
  let quilt_board = QuiltBoard::default();
  let patch = PatchManager::get_patch(12);
  c.bench_function("...", |b| b.iter(||
    black_box(quilt_board.get_valid_actions_for_patch(patch, 0, true)))
  );}

fn quilt_board_get_valid_actions_for_special_patch(
  c: &mut Criterion) {
  let quilt_board = QuiltBoard::default();
  c.bench_function("...", |b| b.iter(||
    black_box(quilt_board.get_valid_actions_for_special_patch())
  ););}
criterion_group!(benches,
  game_get_initial_state, /* ... other benchmarks ... */,
  quilt_board_get_valid_actions_for_special_patch
);
criterion_main!(benches);
