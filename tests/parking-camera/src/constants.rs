use glium::{
    implement_vertex,
    Program,
};


pub const WIN_RES: (u32, u32) = (640, 480u32);
pub const USING_CAMERA: bool = true;
pub const CAMERA_NUM: u32 = 0;
pub const HOUGH_ANGLE_NUM: usize = 30;
pub const HOUGH_ANGLE: f32 = 3.141592654 / HOUGH_ANGLE_NUM as f32;
pub const RHO_INIT: f32 = 100.0;
pub const HOUGH_THRESH: u32 = 2;
pub const RHO_STEP: f32 = 20.0;
pub const RHO_MAX: f32 = 2000.0;
pub const RECT_MIN_AREA_INIT: i32 = 4;
pub const RECT_MIN_AREA_CAR: i32 = 20;
pub const INIT_FRAMES: u32 = 10;
pub const RECT_MIN_DIM_SPACE: (i32, i32) = (60, 120);
pub const RECT_MAX_DIM_SPACE: (i32, i32) = (100, 200);
pub const RECT_MAX_Y_OFFSET: i32 = 10;
pub const NUM_SPACES: usize = 8;

#[derive(Copy, Clone)]
pub struct Vertex {
    pub position: [f32; 2],
    pub tex_coords: [f32; 2],
}
implement_vertex!(Vertex, position, tex_coords);

pub const VERTICES: [Vertex; 6] = [Vertex {position: [-1.0, -1.0], tex_coords: [0.0, 0.0]}, Vertex {position: [1.0, -1.0], tex_coords: [1.0, 0.0]}, Vertex {position: [1.0, 1.0], tex_coords: [1.0, 1.0]},
                                Vertex {position: [1.0, 1.0], tex_coords: [1.0, 1.0]}, Vertex {position: [-1.0, 1.0], tex_coords: [0.0, 1.0]}, Vertex {position: [-1.0, -1.0], tex_coords: [0.0, 0.0]}];

//Identity matrix for transform while loading image
pub const IDENTITY: [[f32; 4]; 4] =    [[1.0, 0.0, 0.0, 0.0],
                                        [0.0, 1.0, 0.0, 0.0],
                                        [0.0, 0.0, 1.0, 0.0],
                                        [0.0, 0.0, 0.0, 1.0f32]];
//Shaders for loading texture
pub const TEX_VS_SRC: &str = r#"
    #version 330
    uniform mat4 matrix;

    in vec2 position;
    in vec2 tex_coords;
    
    out vec2 vp_tex_coords;

    void main() {
        gl_Position = matrix * vec4(position, 0.0, 1.0);
        vp_tex_coords = tex_coords;
    }
"#;

pub const TEX_FS_SRC: &str = r#"
    #version 330
    uniform sampler2D tex;

    in vec2 vp_tex_coords;

    layout(location = 0) out vec4 color;
    
    void main() { 
        color = texture(tex, textureSize(tex, 0) - vp_tex_coords);
    }
"#;

//Shaders for performing gaussian blur
pub const BLUR_VS_SRC: &str = r#"
    #version 330

    uniform mat4 matrix;

    in vec2 position;
    in vec2 tex_coords;

    out vec2 vp_tex_coords;

    void main() {
        gl_Position = matrix * vec4(position, 0.0, 1.0);
        vp_tex_coords = tex_coords;
    }
"#;

pub const BLUR_FS_SRC: &str = r#"
    #version 330
    uniform sampler2D tex;

    in vec2 vp_tex_coords;

    layout(location = 0) out vec4 color;

    uniform float weights[25] = float[25](      0.003,	0.013,	0.022,  0.013,  0.003,
		    		        0.013,  0.059,	0.097,  0.059,  0.013,
                                        0.022,  0.097,	0.159,  0.097,  0.022,
                                        0.013,  0.059,  0.097,  0.059,  0.013,
                                        0.003,  0.013,  0.022,  0.013,  0.003);
    const int weights_dim = 5;
    const int weights_shift = - (weights_dim + 1) / 2;

    void main() {
        vec2 tex_size = textureSize(tex, 0);
        vec2 pixel_size = 1.0 / vec2(tex_size);
	float x = tex_size[0] - vp_tex_coords[0];
	float y = tex_size[1] - vp_tex_coords[1];
	//Gaussian Blur
	vec4 blur_totals = vec4(0.0, 0.0, 0.0, 0.0);
	for(int i = 0; i < weights_dim; i++){
		for(int j = 0; j < weights_dim; j++){
			blur_totals += weights[i + 5 *j] * texture(tex, vec2(x + (i + weights_shift) * pixel_size[0], (y + (j + weights_shift) * pixel_size[1])));
		}
	}
	blur_totals[3] = 1.0;
        color = blur_totals;
    }
"#;

//Shaders for performing gradient calculations
pub const GRAD_VS_SRC: &str = r#"
    #version 330

    uniform mat4 matrix;
    
    in vec2 position;
    in vec2 tex_coords;

    out vec2 vp_tex_coords;

    void main() {
        gl_Position = matrix * vec4(position, 0.0, 1.0);
        vp_tex_coords = tex_coords;
    }
"#;

pub const GRAD_FS_SRC: &str = r#"
    #version 330

    uniform sampler2D tex;

    in vec2 vp_tex_coords;
    
    layout (location = 0) out vec4 color;

    const mat3 sobel = mat3(	-0.125,	0,	0.125,
		    		-0.25,	0,	0.25,
				-0.125,	0,	0.125);
    const float sobel_coeff = 2;
    const int mat_dim = 3;
    const int mat_shift = - (mat_dim - 1) / 2;

    void main() {
        vec2 pixel_size = 1.0 / vec2(textureSize(tex, 0));
        float x = vp_tex_coords[0];
	float y = vp_tex_coords[1];
	vec4 gx = vec4(0, 0, 0, 0);
	vec4 gy = vec4(0, 0, 0, 0);
	for(int i = 0; i < mat_dim; i++){
            for(int j = 0; j < mat_dim; j++){
		gx += sobel[i][j] * texture(tex, vec2(x + (i + mat_shift) * pixel_size[0], y + (j + mat_shift) * pixel_size[1]));
		gy += sobel[j][i] * texture(tex, vec2(x + (i + mat_shift) * pixel_size[0], y + (j + mat_shift) * pixel_size[1]));
	    }
	}
	float gx_mean = (gx[0] + gx[1] + gx[2]) * sobel_coeff;
	float gy_mean = (gy[0] + gy[1] + gy[2]) * sobel_coeff;
	float g_mean = sqrt(gx_mean * gx_mean + gy_mean * gy_mean);
	float theta = atan(gy_mean / gx_mean);
	color = vec4(g_mean, theta, 0, 1);
        //color = vec4(vec3(step(0.2, g_mean)), 1.0);
    }
"#;

pub const SUPP_VS_SRC: &str = r#"
    #version 330

    uniform mat4 matrix;

    in vec2 position;
    in vec2 tex_coords;

    out vec2 vp_tex_coords;

    void main() {
        gl_Position = matrix * vec4(position, 0.0, 1.0);
        vp_tex_coords = tex_coords;
    }
"#;

pub const SUPP_FS_SRC: &str = r#"
    #version 330
    
    uniform sampler2D tex;

    in vec2 vp_tex_coords;

    layout (location = 0) out vec4 color;

    void main() {
        vec2 pixel_size = 1.0 / vec2(textureSize(tex, 0));
        vec4 in_loc = texture(tex, vp_tex_coords);
        float intensity = in_loc[0];
        float norm_x = cos(in_loc[1]);
        float norm_y = sin(in_loc[1]);
        float neighbour_pos = texture(tex, vp_tex_coords + (vec2(norm_x, norm_y) * pixel_size))[0];
        float neighbour_neg = texture(tex, vp_tex_coords - (vec2(norm_x, norm_y) * pixel_size))[0];
        color = vec4(float(intensity > neighbour_pos && intensity > neighbour_neg) * intensity, 0.0, 0.0, 1.0);
    }
"#;

pub const THRESH_VS_SRC: &str = r#"
    #version 330

    uniform mat4 matrix;

    in vec2 position;
    in vec2 tex_coords;

    out vec2 vp_tex_coords;

    void main() {
        gl_Position = matrix * vec4(position, 0.0, 1.0);
        vp_tex_coords = tex_coords;
    }
"#;

pub const THRESH_FS_SRC: &str = r#"
    #version 330

    uniform sampler2D tex;

    in vec2 vp_tex_coords;

    layout (location = 0) out vec4 color;

    const float weak_thresh = 0.025;
    const float strong_thresh = 0.1;

    void main() {
        vec2 pixel_size = 1.0 / vec2(textureSize(tex, 0));
        float tex_val = texture(tex, vp_tex_coords)[0];
        if(tex_val > strong_thresh) {
            color = vec4(1.0, 1.0, 1.0, 1.0);
        } else if (tex_val >= weak_thresh) {
            for (int dx = -1; dx < 2; dx++) {
                for (int dy = -1; dy < 2; dy++) {
                    if(texture(tex, vp_tex_coords + (vec2(dx, dy) * pixel_size))[0] > strong_thresh) {
                        color = vec4(1.0, 1.0, 1.0, 1.0);
                    }
                }
            }
        } else {
            color = vec4(0.0, 0.0, 0.0, 1.0);
        }
    }
"#;

pub const CONTOUR_VS_SRC: &str = r#"
    #version 330

    uniform mat4 matrix;

    in vec2 position;

    void main() {
        gl_Position = matrix * vec4(position, 0.0, 1.0);
    }
"#;

pub const CONTOUR_FS_SRC: &str = r#"
    #version 330
    
    layout (location = 0) out vec4 color;

    void main() {
        color = vec4(1.0, 1.0, 1.0, 1.0);
    }
"#;

pub const DILATE_VS_SRC: &str = r#"
    #version 330

    uniform mat4 matrix;

    in vec2 position;
    in vec2 tex_coords;

    out vec2 vp_tex_coords;

    void main() {
        gl_Position = matrix * vec4(position, 0.0, 1.0);
        vp_tex_coords = tex_coords;
    }
"#;

pub const DILATE_FS_SRC: &str = r#"
    #version 330

    uniform sampler2D tex;

    in vec2 vp_tex_coords;

    layout (location = 0) out vec4 color;

    void main() {
        vec2 pixel_size = 1.0 / vec2(textureSize(tex, 0));
        float tex_val = texture(tex, vp_tex_coords)[0];
        color = vec4(0.0, 0.0, 0.0, 1.0);
        for (int dx = -1; dx < 2; dx++) {
            for (int dy = -1; dy < 2; dy++) {
                if(texture(tex, vp_tex_coords + (vec2(dx, dy) * pixel_size))[0] > 0.0) {
                    color = vec4(1.0, 1.0, 1.0, 1.0);
                }
            }
        }
    }
"#;

pub const ERODE_VS_SRC: &str = r#"
    #version 330

    uniform mat4 matrix;

    in vec2 position;
    in vec2 tex_coords;

    out vec2 vp_tex_coords;

    void main() {
        gl_Position = matrix * vec4(position, 0.0, 1.0);
        vp_tex_coords = tex_coords;
    }
"#;

pub const ERODE_FS_SRC: &str = r#"
    #version 330

    uniform sampler2D tex;

    in vec2 vp_tex_coords;

    layout (location = 0) out vec4 color;

    void main() {
        vec2 pixel_size = 1.0 / vec2(textureSize(tex, 0));
        float tex_val = texture(tex, vp_tex_coords)[0];
        color = vec4(1.0, 1.0, 1.0, 1.0);
        for (int dx = -1; dx < 2; dx++) {
            for (int dy = -1; dy < 2; dy++) {
                if(texture(tex, vp_tex_coords + (vec2(dx, dy) * pixel_size))[0] == 0.0) {
                    color = vec4(0.0, 0.0, 0.0, 1.0);
                }
            }
        }
    }
"#;

pub const SUBTRACT_VS_SRC: &str = r#"
    #version 330

    uniform mat4 matrix;

    in vec2 position;
    in vec2 tex_coords;

    out vec2 vp_tex_coords;

    void main() {
        gl_Position = matrix * vec4(position, 0.0, 1.0);
        vp_tex_coords = tex_coords;
    }
"#;

pub const SUBTRACT_FS_SRC: &str = r#"
    #version 330

    uniform sampler2D tex;
    uniform sampler2D tex2;

    in vec2 vp_tex_coords;

    layout (location = 0) out vec4 color;

    void main() {
        vec4 tex_val = texture(tex, vp_tex_coords);
        vec4 tex2_val = texture(tex2, vp_tex_coords);
        color = tex_val - tex2_val;
        color[3] = 1;
    }
"#;

pub const LINES_VS_SRC: &str = r#"
    #version 330

    uniform mat4 matrix;

    in vec2 position;
    in vec2 tex_coords;

    out vec2 vp_tex_coords;

    void main() {
        gl_Position = matrix * vec4(position, 0.0, 1.0);
        vp_tex_coords = tex_coords;
    }
"#;

pub const LINES_FS_SRC: &str = r#"
    #version 330
    
    in vec2 vp_tex_coords;

    out vec4 color;

    void main() {
        color = vec4(1.0, 1.0, 1.0, 1.0);
    }
"#;
