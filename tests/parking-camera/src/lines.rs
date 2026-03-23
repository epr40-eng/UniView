use image::{
    ImageBuffer,
    Luma,
    open,
};

use imageproc::{
    hough::{
        LineDetectionOptions,
        PolarLine,
        detect_lines,
        intersection_points,
    },
    contours::{
        Contour,
        find_contours,
    },
    geometry::{
        contour_area,
        min_area_rect,
    },
    point::{
        Point,
    },
};

use crate::constants::{
    HOUGH_THRESH,
    WIN_RES,
    RECT_MIN_DIM_SPACE,
    RECT_MAX_DIM_SPACE,
    RECT_MAX_Y_OFFSET,
};


pub fn hough_lines(image: &ImageBuffer<Luma<u8>, Vec<u8>>) -> Vec<((f32, f32), (f32, f32))>{
    let opt = LineDetectionOptions {
        vote_threshold: HOUGH_THRESH,
        suppression_radius: 8,
    };
     
    let polar_lines = detect_lines(image, opt);
    let mut rect_lines: Vec<((f32, f32), (f32, f32))> = Vec::new();
    for line in polar_lines {
        rect_lines.push(intersection_points(line, WIN_RES.0 as u32, WIN_RES.1 as u32).unwrap());
    }
    return rect_lines;
}

pub fn get_contours(buf: ImageBuffer<Luma<u8>, Vec<u8>>) -> Vec<Contour<i32>>{
    let mut output = Vec::new();
    for contour in find_contours::<i32>(&buf) {
        output.push(contour.clone());
        //if contour.points.len() == 4 {
        //    let points: [Point<i32>; 4] = contour.points.clone().try_into().unwrap();
        //    output.push(contour)
        //}
    }
    return output;
}

pub fn get_rects(contours: Vec<Contour<i32>>) -> Vec<[Point<i32>; 4]> {
    let mut output: Vec<[Point<i32>; 4]> = Vec::new();
    for contour in contours {
        output.push(min_area_rect(&contour.points));
    }
    return output;
}

pub fn area_filter(rects: Vec<[Point<i32>; 4]>, min_area: i32) -> Vec<[Point<i32>; 4]> {
    let area_sq = min_area ^ 2;
    let mut output: Vec<[Point<i32>; 4]> = Vec::new();
    for rect in rects {
        let mut dx2 = 0;
        let mut dy2 = 0;
        for i in rect {
            if (rect[0].x - i.x) ^ 2 > dx2 {
                dx2 = (rect[0].x - i.x) ^ 2;
            }
            if (rect[0].y - i.y) ^ 2 > dy2 {
                dy2 = (rect[0].y - i.y) ^ 2
            }
        }
        //let width_sq = (a.x - b.x) ^ 2 + (a.y - b.y) ^ 2;
        //let height_sq = (a.x - c.x) ^ 2 + (a.y - c.y) ^ 2;
        if dx2 * dy2 > area_sq {
            output.push(rect);
        }
    }
    return output;
}

pub fn get_spaces(edges: Vec<[Point<i32>; 4]>) -> Vec<[Point<i32>; 4]> {
    let mut spaces: Vec<[Point<i32>; 4]> = Vec::new();
    println!("{}", edges.len());
    for left in edges.clone() {
        let left_max_x: i32 = (left as [Point<i32>; 4]).into_iter().map(|p| p.x).max().unwrap();
        let left_max_y: i32 = (left as [Point<i32>; 4]).into_iter().map(|p| p.y).max().unwrap();
        let left_min_x: i32 = (left as [Point<i32>; 4]).into_iter().map(|p| p.x).min().unwrap();
        let left_min_y: i32 = (left as [Point<i32>; 4]).into_iter().map(|p| p.y).min().unwrap();
        for right in edges.clone() {
            let right_max_x = (right as [Point<i32>; 4]).into_iter().map(|p| p.x).max().unwrap();
            let right_max_y = (right as [Point<i32>; 4]).into_iter().map(|p| p.y).max().unwrap();
            let right_min_x = (right as [Point<i32>; 4]).into_iter().map(|p| p.x).min().unwrap();
            let right_min_y = (right as [Point<i32>; 4]).into_iter().map(|p| p.y).min().unwrap();
            let dim_x = right_min_x - left_max_x;
            let dim_y = [right_max_y - right_min_y, left_max_y - left_min_y].into_iter().min().unwrap();
            let offset_y = [(right_max_y - left_max_y).abs(), (right_min_y - left_min_y).abs()].into_iter().max().unwrap();
            if dim_x > RECT_MIN_DIM_SPACE.0 && dim_x < RECT_MAX_DIM_SPACE.0 {
                if dim_y > RECT_MIN_DIM_SPACE.1 && dim_y < RECT_MAX_DIM_SPACE.1 {
                    if offset_y < RECT_MAX_Y_OFFSET {
                        spaces.push([   Point {x: left_max_x, y: right_max_y},
                                        Point {x: right_min_x, y: right_max_y},
                                        Point {x: right_min_x, y: right_min_y},
                                        Point {x: left_max_x, y: right_min_y}]);
                    }
                }
            }
        }
    }
    println!("{}", spaces.len());
    return spaces;
}

pub fn check_overlap(cars: Vec<[Point<i32>; 4]>, spaces: Vec<[Point<i32>; 4]>) -> Vec<bool> {
    let mut overlaps: Vec<bool> = Vec::new();
    for space in spaces {
        let space_min_x = space[0].x;
        let space_max_x = space[1].x;
        let space_min_y = space[2].y;
        let space_max_y = space[0].y;
        let mut space_occupied = false;
        //println!("{}, {}, {}, {}", space_min_x, space_max_x, space_min_y, space_max_y);
        for car in &cars {
            for point in car {
                //println!("{}, {}", point.x, point.y);
                if point.x > space_min_x && point.x < space_max_x {
                    if point.y > space_min_y && point.y < space_max_y {
                        //println!("Space occupied");
                        space_occupied = true;
                    }
                }
            }
        }
        overlaps.push(space_occupied);
    }
    return overlaps;
}
