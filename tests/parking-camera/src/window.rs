use glium::
{
    Surface,
    winit::{
        event::{
            Event,
            WindowEvent,
        },
        event_loop::EventLoop,
        window::Window,
    },
    backend::{
        glutin::{
            Display,
            SimpleWindowBuilder,
        },
        Facade,
    },
    glutin::{
        surface::{
            WindowSurface,
        },
    },
    framebuffer::{
        SimpleFrameBuffer,
    },
    Texture2d,
};

use imageproc::{
    point::{
        Point,
    },
};

use crate::video;
use crate::constants::{
    WIN_RES,
};


#[derive(Default)]
pub struct Application {
    pub event_loop: Option<EventLoop<()>>,
    pub window: Option<Window>,
    pub display: Option<Display<WindowSurface>>,
    pub camera: Option<video::Cam>,
    pub init_contours: Option<Vec<[Point<i32>; 4]>>,
    pub initialized: u32,
    pub background: Option<Texture2d>,
    pub init_spaces: Option<Vec<[Point<i32>; 4]>>,
}

impl Application {
    pub fn new() -> Self {
        return Self {
            event_loop: None,
            window: None,
            display: None,
            camera: None,
            init_contours: None,
            initialized: 0,
            background: None,
            init_spaces: None,
        };
    }

    pub fn setup_frame(&mut self) {
        let mut target = self.display.clone().unwrap().draw();
        target.clear_color(0.0, 0.0, 0.0, 1.0);
        target.finish().unwrap();
        let event_loop = self.event_loop.take().unwrap();
        self.event_loop = Some(event_loop);
    }

    pub fn run_loop(&mut self, func: fn(&mut Application, u32)) {
        let event_loop = self.event_loop.take().unwrap();
        let _ = event_loop.run(move | event, window_target| {
            match event {
                Event::WindowEvent {event, .. } => match event {
                    WindowEvent::CloseRequested => window_target.exit(),
                    WindowEvent::RedrawRequested => func(self, self.initialized),
                    _ => (),
                },
                Event::AboutToWait => self.window.as_ref().unwrap().request_redraw(),
                _ => (),
            };
        }).expect("Event Loop Started");
    }

    pub fn initialize(&mut self, x_res: u32, y_res: u32, title: &str) {
        self.event_loop = Some(EventLoop::builder()
            .build()
            .expect("Event Loop Initialization"));
        let el_int = self.event_loop.as_mut().unwrap();
        let (window, display) = SimpleWindowBuilder::new()
            .with_inner_size(x_res, y_res)
            .with_title(title)
            .build(el_int);
        self.window = Some(window);
        self.display = Some(display);
        self.init_contours = Some(Vec::new())
    }

    pub fn init_camera(&mut self, cam_num: u32) {
        let mut cam = video::Cam::new();
        cam.initialize_camera(cam_num);
        self.camera = Some(cam);
    }

    pub fn clear_texture(&mut self) -> Texture2d{
        return Texture2d::empty(&self.display.clone().unwrap(), WIN_RES.0, WIN_RES.1).unwrap();
    }

    pub fn get_display(&mut self) -> &Option<Display<WindowSurface>> {
        return &self.display;
    }

    pub fn get_camera(&mut self) -> &Option<video::Cam> {
        return &self.camera;
    }

    pub fn get_frame(&mut self) -> Texture2d{
        return self.camera.as_mut().unwrap().get_frame(self.display.clone().unwrap());
    }

    pub fn write_contours(&mut self, contours: Vec<[Point<i32>; 4]>) {
        self.init_contours = Some(contours.clone());
    }
}
