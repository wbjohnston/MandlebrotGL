//! mandlebrot set visualizer using opengl
//! TODO: shaders need to be double precision
//! maybe opengl isnt the right thing for the job, openCL or compute shaders?

#[macro_use]
extern crate glium;
use glium::glutin::{Event, VirtualKeyCode, ElementState};
use glium::{DisplayBuild, Surface};

#[derive(Clone, Copy)]
struct Vertex {
    position: [f32; 2],
}
implement_vertex!(Vertex, position);

// the mandlebrot set lies stricle in the bounds (-2.0, -1.0) to (1.0, 1.0)
const INIT_X_SCALE: f32 = 3.0;
const INIT_Y_SCALE: f32 = 2.0;

// mandlebrot fragment shader
const FRAGMENT_SHADER_SRC: &str = r#"
#version 140

out vec4 color;
uniform vec2 window_size;           // size of window (f32, f32)
uniform mat2 scale_mat;             // transform matrix
uniform vec2 trans_vec;             // translation matrix
uniform int max_iterations;         // maximum number of interations

void main()
{
    vec2 q = scale_mat * (gl_FragCoord.xy / window_size) + trans_vec;

    highp float x0 = q.x;
    highp float y0 = q.y;

    float x = 0.0;
    float y = 0.0;
    int n_iter = 0;

    // calculate number of iterations it takes to escape
    while (x * x + y * y < 2.0 && n_iter < max_iterations) {
        float xt = x * x - y * y + x0;
        float yt = 2.0 * x * y + y0;

        // simultaneous update
        x = xt; 
        y = yt;
        n_iter += 1;
    }

    // TODO: replace this with an actual color palette
    float i = float(n_iter) / float(max_iterations);

    gl_FragColor = vec4(i, i, i, 1.0);
}
"#;

// Identity shader (there's really no verices
const VERTEX_SHADER_SRC: &str = r#"
#version 140

in vec2 position;

void main()
{
    gl_Position = vec4(position, 0.0, 1.0);
}
"#;

fn main() {
    // build display
    let display = glium::glutin::WindowBuilder::new()
        .build_glium()
        .unwrap();

    // two triangles to cover entire screen instead of a quad
    let window_vertices: Vec<Vertex> = vec![
        Vertex{ position: [-1.0,  1.0] }, // top left
        Vertex{ position: [ 1.0,  1.0] },
        Vertex{ position: [-1.0, -1.0] },

        Vertex { position: [-1.0, -1.0] }, // top right
        Vertex { position: [ 1.0,  1.0] },
        Vertex { position: [ 1.0, -1.0] },
    ];

    let vertex_buffer = glium::VertexBuffer::new(&display, &window_vertices)
        .unwrap();

    let indices = glium::index::NoIndices(
        glium::index::PrimitiveType::TrianglesList
        );

    // compile kernel
    let program = glium::Program::from_source(
        &display,
        VERTEX_SHADER_SRC,
        FRAGMENT_SHADER_SRC,
        None
        ).unwrap();


    // initial transform matrix
    let mut px = -2.0f32;
    let mut py = -1.0f32;

    let mut scale = 0.0;
    let mut dirty = true;

    'main: loop {
        if dirty { // image needs to be redrawn
            let mut target = display.draw();

            let (rx, ry) = target.get_dimensions();

            // matrix used to scale fragments
            let scale_mat = [
                [INIT_X_SCALE * 10f32.powf(scale),  0.0],
                [0.0, INIT_Y_SCALE * 10f32.powf(scale)],
            ];

            // create uniforms to be uploaded to the GPU
            let uniforms = uniform! {
                scale_mat: scale_mat,
                trans_vec: [px, py],
                max_iterations: 255,
                window_size: [rx as f32, ry as f32]
            };

            target.draw(
                &vertex_buffer,
                &indices,
                &program,
                &uniforms,
                &Default::default()
                ).unwrap();

            target.finish().unwrap();
            dirty = false;
        }

        for e in display.poll_events() {
            match e {
                Event::Closed => break 'main,
                Event::Resized{..} => dirty = true,
                Event::KeyboardInput(ElementState::Pressed, _, Some(s)) => {
                    match s {
                        VirtualKeyCode::W => { // up
                            py += 0.1 * 10f32.powf(scale);
                            dirty = true;
                        },
                        VirtualKeyCode::S => { // down
                            py -= 0.1 * 10f32.powf(scale);
                            dirty = true;
                        },
                        VirtualKeyCode::A => { // left
                            px -= 0.1 * 10f32.powf(scale); 
                            dirty = true;
                        },
                        VirtualKeyCode::D => { // right
                            px += 0.1 * 10f32.powf(scale); 
                            dirty = true;
                        },
                        VirtualKeyCode::Z => { // zoom in
                            scale -= 0.01;
                            dirty = true;
                        },
                        VirtualKeyCode::X => { // zoom out
                            scale += 0.01; 
                            dirty = true;
                        },
                        VirtualKeyCode::Escape  // quit
                            | VirtualKeyCode::Q => {
                            break 'main;
                        }
                        _ => {}
                    }
                },
                _ => {}
            }
        }
    }
}

