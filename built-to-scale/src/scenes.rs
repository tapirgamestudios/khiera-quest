use alloc::boxed::Box;

mod game;
mod state;

pub use state::*;

enum TransitionScene {}

trait Scene {
    fn transition(&mut self, transition: &mut Transition) -> Option<TransitionScene>;
    fn update<'a>(&mut self, update: &'a mut Update<'a>);
    fn display(&mut self, display: &mut Display);
}

impl SceneManager {
    pub fn new() -> Self {
        Self {
            current_scene: Box::new(game::Game::new()),
        }
    }

    pub fn frame<'a>(&mut self, update: &'a mut Update<'a>) {
        self.current_scene.update(update);
    }

    pub fn display(&mut self, display: &mut Display) {
        self.current_scene.display(display);
    }
}

pub struct SceneManager {
    current_scene: Box<dyn Scene>,
}
