use glium::{
    texture::{
        Texture2d,
        RawImage2d,
    },
    backend::glutin::{
        Display,
    },
    glutin::{
        surface::{
            WindowSurface,
        },
    },
    index::{
        NoIndices,
        PrimitiveType,
    },
    uniforms::{
        EmptyUniforms,
    },
    framebuffer::{
        SimpleFrameBuffer,
    },
    uniform,
    VertexBuffer,
    Surface,
    Program,
    DrawParameters,
};

use image::{
    ImageBuffer,
    Rgb,
    Rgba,
    Luma,
    DynamicImage::ImageRgba8,
    GrayImage
};

use imageproc::{
    contours::{
        Contour,
    },
    drawing::{
        draw_hollow_polygon_mut,
    },
    point::{
        Point,
    },
};

use crate::constants::{
    Vertex,
    VERTICES,
    IDENTITY,
    TEX_VS_SRC,
    TEX_FS_SRC,
    BLUR_VS_SRC,
    BLUR_FS_SRC,
    GRAD_VS_SRC,
    GRAD_FS_SRC,
    SUPP_VS_SRC,
    SUPP_FS_SRC,
    THRESH_VS_SRC,
    THRESH_FS_SRC,
    CONTOUR_VS_SRC,
    CONTOUR_FS_SRC,
    DILATE_VS_SRC,
    DILATE_FS_SRC,
    ERODE_VS_SRC,
    ERODE_FS_SRC,
    SUBTRACT_VS_SRC,
    SUBTRACT_FS_SRC,
    LINES_VS_SRC,
    LINES_FS_SRC,
    WIN_RES,
};

pub fn draw_program(disp: &Display<WindowSurface>, fb: &mut Option<SimpleFrameBuffer>, text: &Texture2d, vs: &str, fs: &str) {
    let vertex_buffer = VertexBuffer::new(disp, &VERTICES).unwrap();
    let indices = NoIndices(PrimitiveType::TrianglesList);
    let uniforms = uniform!(matrix: IDENTITY, tex: text);
    let program = Program::from_source(disp, vs, fs, None).unwrap();
    if fb.is_none() {
        let mut target = disp.clone().draw();
        target.draw(&vertex_buffer, &indices, &program, &uniforms, &Default::default()).unwrap();
        target.finish().unwrap();
    } else {
        fb.as_mut().unwrap().draw(&vertex_buffer, &indices, &program, &uniforms, &Default::default()).unwrap();
    }
}

pub fn draw_rect(disp: &Display<WindowSurface>, fb: &mut Option<SimpleFrameBuffer>, text: &Texture2d, vertices: [Vertex; 4]) {
    let vertex_buffer = VertexBuffer::new(disp, &vertices).unwrap();
    let indices = NoIndices(PrimitiveType::LineLoop);
    let uniforms = uniform!(matrix: IDENTITY, tex: text);
    let program = Program::from_source(disp, LINES_VS_SRC, LINES_FS_SRC, None).unwrap();
    if fb.is_none() {
        let mut target = disp.clone().draw();
        target.draw(&vertex_buffer, &indices, &program, &uniforms, &Default::default()).unwrap();
        target.finish().unwrap();
    } else {
        fb.as_mut().unwrap().draw(&vertex_buffer, &indices, &program, &uniforms, &Default::default()).unwrap();
    }
}

pub fn draw_subtract(disp: &Display<WindowSurface>, fb: &mut Option<SimpleFrameBuffer>, text: &Texture2d, text2: &Texture2d) {
    let vertex_buffer = VertexBuffer::new(disp, &VERTICES).unwrap();
    let indices = NoIndices(PrimitiveType::TrianglesList);
    let uniforms = uniform!(matrix: IDENTITY, tex: text, tex2: text2);
    let program = Program::from_source(disp, SUBTRACT_VS_SRC, SUBTRACT_FS_SRC, None).unwrap();
    if fb.is_none() {
        let mut target = disp.clone().draw();
        target.draw(&vertex_buffer, &indices, &program, &uniforms, &Default::default()).unwrap();
    } else {
        fb.as_mut().unwrap().draw(&vertex_buffer, &indices, &program, &uniforms, &Default::default()).unwrap();
    }
}

pub fn draw_texture(disp: &Display<WindowSurface>, fb: &mut Option<SimpleFrameBuffer>, text: &Texture2d) {
    draw_program(disp, fb, text, TEX_VS_SRC, TEX_FS_SRC);
}

pub fn draw_blur(disp: &Display<WindowSurface>, fb: &mut Option<SimpleFrameBuffer>, text: &Texture2d) {
    draw_program(disp, fb, text, BLUR_VS_SRC, BLUR_FS_SRC);
}

pub fn draw_gradient(disp: &Display<WindowSurface>, fb: &mut Option<SimpleFrameBuffer>, text: &Texture2d) {
    draw_program(disp, fb, text, GRAD_VS_SRC, GRAD_FS_SRC);
}

pub fn draw_suppression(disp: &Display<WindowSurface>, fb: &mut Option<SimpleFrameBuffer>, text: &Texture2d) {
    draw_program(disp, fb, text, SUPP_VS_SRC, SUPP_FS_SRC);
}

pub fn draw_threshholding(disp: &Display<WindowSurface>, fb: &mut Option<SimpleFrameBuffer>, text: &Texture2d) {
    draw_program(disp, fb, text, THRESH_VS_SRC, THRESH_FS_SRC);
}

pub fn draw_dilation(disp: &Display<WindowSurface>, fb: &mut Option<SimpleFrameBuffer>, text: &Texture2d) {
    draw_program(disp, fb, text, DILATE_VS_SRC, DILATE_FS_SRC);
}

pub fn draw_erosion(disp: &Display<WindowSurface>, fb: &mut Option<SimpleFrameBuffer>, text: &Texture2d) {
    draw_program(disp, fb, text, ERODE_VS_SRC, ERODE_FS_SRC);
}

pub fn draw_contours(disp: &Display<WindowSurface>, fb: &mut Option<SimpleFrameBuffer>, contours: Vec<[Point<i32>; 4]>) {
    let mut vertices: Vec<Vec<Vertex>> = Vec::new();
    for contour in contours {
        //println!("Contour found!");
        let mut vertex_group: Vec<Vertex> = Vec::new();
        for point in &contour {
            vertex_group.push(Vertex {position: [2.0 * point.x as f32 / WIN_RES.0 as f32 - 1.0, 2.0 * point.y as f32 / WIN_RES.1 as f32 - 1.0], tex_coords: [0.0, 0.0]});
        }
        vertices.push(vertex_group);
    }
    let indices = NoIndices(PrimitiveType::TriangleFan);
    let uniforms = uniform!(matrix: IDENTITY);
    let program = Program::from_source(disp, CONTOUR_VS_SRC, CONTOUR_FS_SRC, None).unwrap();
    let params = DrawParameters {
        polygon_mode: glium::draw_parameters::PolygonMode::Line,
        ..Default::default()
    };
    let mut target = disp.clone().draw();
    if fb.is_none() {
        target.clear_color(0.0, 0.0, 0.0, 1.0);
    }
    for contour in vertices {
        let vertex_buffer = VertexBuffer::new(disp, &contour).unwrap();
        if fb.is_none() {
            target.draw(&vertex_buffer, &indices, &program, &uniforms, &params).unwrap();
        } else {
            fb.as_mut().unwrap().draw(&vertex_buffer, &indices, &program, &uniforms, &params).unwrap();
        }
    }
    target.finish().unwrap();
}

pub fn write_texture_to_file(text: &Texture2d, filename: &str) {
    let width = text.get_width();
    let height = text.get_height().unwrap();
    let buffer: Vec<Vec<(u8, u8, u8, u8)>> = text.read();
    let img = ImageBuffer::from_fn(width, height, |x, y| {
        let pixel = buffer[y as usize][x as usize];
        Rgba(pixel.into())
    });
    img.save(filename).unwrap();
}

pub fn rgba_to_grayscale(text: &Texture2d) -> ImageBuffer<Luma<u8>, Vec<u8>> {
    let width = text.get_width();
    let height = text.get_height().unwrap();
    let buffer: Vec<Vec<(u8, u8, u8, u8)>> = text.read();
    let img_gray = ImageBuffer::from_fn(width, height, |x, y| {
        let pixel_in = buffer[y as usize][x as usize];
        let pixel_out = [(0.2126 * pixel_in.0 as f32 + 0.7152 * pixel_in.1 as f32 + 0.0722 * pixel_in.2 as f32) as u8];
        Luma(pixel_out)
    });
    return img_gray;
}
