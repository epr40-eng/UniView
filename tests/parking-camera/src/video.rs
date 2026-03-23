use image::{
    ImageReader
};

use nokhwa::{
    utils::{
        CameraIndex,
        RequestedFormat,
        RequestedFormatType,
        CameraFormat,
        FrameFormat,
        Resolution,
    },
    Buffer,
    Camera,
    pixel_format::{
        RgbFormat,
    },
};

use std::{
    io::Cursor,
};

use glium::{
    texture::{
        Texture2d,
        RawImage2d,
    },
    backend::glutin::{
        Display,
    },
    glutin::{
        surface::WindowSurface,
    },
};


pub const CAMERA_RES: [u32; 2] = [3840, 2160];
pub const CAMERA_FPS: i32 = 30;


fn load_image_file(disp: Display<WindowSurface>, path: &str) -> Texture2d {
    let img = ImageReader::open(path).unwrap().decode().unwrap()
            .to_rgba8();
    let img_dim = img.dimensions();
    let img_raw = RawImage2d::from_raw_rgba_reversed(&img.into_raw(), img_dim);
    return Texture2d::new(&disp, img_raw).unwrap();
}

pub struct Cam {
    camera: Option<Camera>,
}

impl Cam {
    pub fn new() -> Self {
        return Self {
            camera: None,
        };
    }
    
    pub fn initialize_camera(&mut self, cam_ind: u32) {
        let ind = CameraIndex::Index(cam_ind);
        let req = RequestedFormat::new::<RgbFormat>(RequestedFormatType::AbsoluteHighestFrameRate);
        let mut cam = Camera::new(ind, req).unwrap();
        cam.set_resolution(Resolution::new(CAMERA_RES[0], CAMERA_RES[1]));
        cam.open_stream();
        self.camera = Some(cam);
    }

    pub fn get_frame(&mut self, disp: Display<WindowSurface>) -> Texture2d {
        let buffer = self.camera.as_mut().unwrap()
            .frame().unwrap();
        let res = buffer.resolution();
        let data = buffer.decode_image::<RgbFormat>().unwrap();
        let raw = RawImage2d::from_raw_rgb(data.to_vec(), (res.width_x, res.height_y));
        return Texture2d::new(&disp, raw).unwrap();
    }
}
