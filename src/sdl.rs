use sdl2;
use io;

pub struct Contexts<'scope> {
    pub sdl: sdl2::Sdl,
    pub font: sdl2::ttf::Font<'scope, 'static>,
    pub canvas: sdl2::render::WindowCanvas
}

pub fn with_contexts<T, F>(func: F) -> T 
    where F: FnOnce(Contexts) -> T
{
    let sdl_context = sdl2::init().unwrap();
    let sdl_ttf_context = sdl2::ttf::init().unwrap();
    let font = sdl_ttf_context.load_font("fonts/Carlito-Regular.ttf", 24).unwrap();
    let video_sybsys = sdl_context.video().unwrap();
    let window = video_sybsys.window("Chippy", io::WINDOW_WIDTH, io::WINDOW_HEIGHT)
        .position_centered()
        .opengl()
        .build()
        .unwrap();
    let canvas = window.into_canvas().build().unwrap();
    let contexts = Contexts {
        sdl: sdl_context,
        font: font,
        canvas: canvas
    };
    func(contexts)
}
