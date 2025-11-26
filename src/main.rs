fn main() {
    glazer::run_opengl(
        mogl::Memory::default(),
        mogl::WIDTH,
        mogl::HEIGHT,
        mogl::handle_input,
        mogl::update_and_render,
        mogl::initialize_opengl,
        glazer::debug_target(),
    );
}

