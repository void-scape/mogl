fn main() {
    glazer::run(
        mogl::Memory::default(),
        mogl::WIDTH,
        mogl::HEIGHT,
        mogl::handle_input,
        mogl::update_and_render,
        glazer::debug_target(),
    );
}
