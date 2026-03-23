use std::{
    thread,
    time::Duration,
};

use glium::{
    framebuffer::SimpleFrameBuffer,
    Texture2d,
};

mod window;
mod video;
mod draw;
mod constants;
mod lines;

use crate::lines::{
    hough_lines,
    get_contours,
    get_rects,
    get_spaces,
    check_overlap,
    area_filter,
};

use crate::constants::{
    WIN_RES,
    USING_CAMERA,
    CAMERA_NUM,
    RECT_MIN_AREA_INIT,
    RECT_MIN_AREA_CAR,
    INIT_FRAMES,
    NUM_SPACES,
};


fn mainloop(appl: &mut window::Application, frame_num: u32) {
    let disp = appl.get_display().clone().unwrap();
    let cam_tex = appl.get_frame();
    let mut blur_tex = Texture2d::empty(&disp, WIN_RES.0, WIN_RES.1).unwrap();
    let mut cam_fb = SimpleFrameBuffer::new(&disp, &blur_tex).unwrap();
    let mut gradient_tex = Texture2d::empty(&disp, WIN_RES.0, WIN_RES.1).unwrap();
    let mut blur_fb = SimpleFrameBuffer::new(&disp, &gradient_tex).unwrap();
    let mut suppression_tex = Texture2d::empty(&disp, WIN_RES.0, WIN_RES.1).unwrap();
    let mut gradient_fb = SimpleFrameBuffer::new(&disp, &suppression_tex).unwrap();
    let mut threshholding_tex = Texture2d::empty(&disp, WIN_RES.0, WIN_RES.1).unwrap();
    let mut suppression_fb = SimpleFrameBuffer::new(&disp, &threshholding_tex).unwrap();
    let mut dilation_tex = Texture2d::empty(&disp, WIN_RES.0, WIN_RES.1).unwrap();
    let mut threshholding_fb = SimpleFrameBuffer::new(&disp, &dilation_tex).unwrap();
    let mut erosion_tex = Texture2d::empty(&disp, WIN_RES.0, WIN_RES.1).unwrap();
    let mut dilation_fb = SimpleFrameBuffer::new(&disp, &erosion_tex).unwrap();
    let mut handoff_tex = Texture2d::empty(&disp, WIN_RES.0, WIN_RES.1).unwrap();
    let mut erosion_fb = SimpleFrameBuffer::new(&disp, &handoff_tex).unwrap();
    let mut cam_ptr = Some(cam_fb);
    if frame_num < INIT_FRAMES {
        draw::draw_texture(&disp, &mut cam_ptr, &cam_tex);
    } else {
        draw::draw_subtract(&disp, &mut cam_ptr, &cam_tex, appl.background.as_ref().unwrap());
    }
    let mut blur_ptr = Some(blur_fb);
    for i in 0..4 {
        draw::draw_blur(&disp, &mut blur_ptr, &blur_tex);
    }
    draw::draw_gradient(&disp, &mut Some(gradient_fb), &gradient_tex);
    draw::draw_suppression(&disp, &mut Some(suppression_fb), &suppression_tex);
    draw::draw_threshholding(&disp, &mut Some(threshholding_fb), &threshholding_tex);
    let mut dilation_ptr = Some(dilation_fb);
    for i in 0..8 {
        draw::draw_dilation(&disp, &mut dilation_ptr, &dilation_tex);
    }
    let mut erosion_ptr = Some(erosion_fb);
    for i in 0..8 {
        draw::draw_erosion(&disp, &mut erosion_ptr, &erosion_tex);
    }
    //draw::draw_threshholding(&disp, &mut None, &threshholding_tex);
    //draw::write_texture_to_file(&handoff_tex, "images/edges.png");
    let grayscale = draw::rgba_to_grayscale(&erosion_tex);
    let contours = get_rects(get_contours(grayscale.clone()));
    if frame_num >= INIT_FRAMES && contours.len() > 0 {
        let cars = area_filter(contours.clone(), RECT_MIN_AREA_CAR);
        draw::draw_contours(&disp, &mut None, cars.clone());
        let overlaps = check_overlap(cars.clone(), appl.init_spaces.clone().unwrap());
        let mut num_occupied = 0;
        for overlap in overlaps {
            if overlap {
                num_occupied += 1;
            }
            //print!("{}, ", overlap);
        }
        println!("{} spaces occupied.", num_occupied);
    }
    if frame_num < INIT_FRAMES {
        let mut init_tex = Texture2d::empty(&disp, WIN_RES.0, WIN_RES.1).unwrap();
        let mut init_fb = SimpleFrameBuffer::new(&disp, &init_tex).unwrap();
        draw::draw_contours(&disp, &mut Some(init_fb), area_filter(contours.clone(), RECT_MIN_AREA_INIT));
        //thread::sleep(Duration::from_secs(10));
        appl.write_contours(contours);
        let mut rects = get_rects(get_contours(grayscale.clone()));
        let mut spaces = get_spaces(rects.clone());
        appl.init_spaces = Some(spaces.clone());
        let cam_vec: Vec<Vec<(u8, u8, u8, u8)>> = cam_tex.read();
        appl.background = Some(Texture2d::new(&disp, cam_vec).unwrap());
        if spaces.len() != NUM_SPACES && frame_num == INIT_FRAMES - 1{
            appl.initialized -= 1;
        }
    }
    appl.initialized += 1;
    //println!("{}", contours[0].points[0].x); 
    //let lines = polar_lines_to_rect_lines(hough_lines(&grayscale));
    //thread::sleep(Duration::from_secs(15));
    println!("Frame {} update complete", appl.initialized);
}

fn main() {
    let mut appl = window::Application::new();
    appl.initialize(WIN_RES.0, WIN_RES.1, "CV Window");
    appl.setup_frame();

    if USING_CAMERA {
        appl.init_camera(CAMERA_NUM);
    }

    appl.run_loop(mainloop);
}
