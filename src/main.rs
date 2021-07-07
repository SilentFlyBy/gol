extern crate glfw;

extern crate gl;
use std::io::Write;
use std::str::FromStr;
use std::sync::Arc;
use std::sync::Mutex;
use gl::types::*;

extern crate crossbeam_channel;

use core::time;
use std::ffi::CString;
use std::os::raw::c_void;
use std::{io, thread};
use std::time::Instant;
use glfw::{Action, Context, Key, MouseButton};
use std::{ptr, sync::mpsc::Receiver, mem, str};
use grid::Grid;

use crate::rle::RLE;
use crate::shader::Shader;

mod grid;
mod rle;
mod shader;

const VERTEX_SHADER_SOURCE: &str = r#"
    #version 330 core
    layout (location = 0) in float vertexActive;

    uniform int gridLength;

    out float fragmentActive;

    const float stepSize = 0.02;

    void main() {
        int cell = gl_VertexID / 6;
        int row = cell / gridLength;
        int col = cell - (row * gridLength);
        int vertexNr = gl_VertexID - (cell * 6);

        float colFloat = float(col);
        float rowFloat = float(row);
        float gridLengthFloat = float(gridLength);

        float x = -1.0 + ((colFloat / gridLengthFloat) * 2.0);
        float y = 1.0 - ((rowFloat / gridLengthFloat) * 2.0);



        if (vertexNr == 1) {
            y = y - stepSize;
        }
        else if (vertexNr == 2 || vertexNr == 3) {
            x = x + stepSize;
            y = y - stepSize;
        }
        else if (vertexNr == 4) {
            x = x + stepSize;
        }

        fragmentActive = vertexActive;
        gl_Position = vec4(x, y, 0.0, 1.0);
    }
"#;

const FRAGMENT_SHADER_SOURCE: &str = r#"
    #version 330 core
    in float fragmentActive;
    out vec4 FragColor;

    void main() {
       FragColor = fragmentActive == 1.0 ? vec4(1.0f, 0.5f, 0.2f, 1.0f) : vec4(0.2f, 0.3f, 0.3f, 1.0f);
    }
"#;
const GRID_LENGTH: usize = 100;

const CELL_SIZE: usize = 2 * 3;
const VERTEX_ARRAY_SIZE: usize = GRID_LENGTH * GRID_LENGTH * CELL_SIZE;

struct InputStates {
    mouse_x: f64,
    mouse_y: f64,
    mouse_left: bool
}

fn main() {
    let mut grid = Grid::new();
    
    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
    glfw.window_hint(glfw::WindowHint::ContextVersion(3, 3));
    glfw.window_hint(glfw::WindowHint::OpenGlProfile(glfw::OpenGlProfileHint::Core));
    #[cfg(target_os = "macos")]
    glfw.window_hint(glfw::WindowHint::OpenGlForwardCompat(true));
    
    let (mut window, events) = glfw.create_window(800, 800, "yagol", glfw::WindowMode::Windowed)
    .expect("Failed to create GLFW window.");
    
    window.make_current();
    window.set_key_polling(true);
    window.set_framebuffer_size_polling(true);
    window.set_mouse_button_polling(true);
    window.set_cursor_pos_polling(true);
    
    gl::load_with(|symbol| window.get_proc_address(symbol) as *const _);
    
    
    let vtx_arr_primary: Arc<Mutex<Vec<f32>>> = Arc::new(Mutex::new(vec![0.0; VERTEX_ARRAY_SIZE]));
    let vtx_arr_secondary: Arc<Mutex<Vec<f32>>> = Arc::new(Mutex::new(vec![0.0; VERTEX_ARRAY_SIZE]));
    let buffer_order = Arc::new(Mutex::new(false));

    let shader = Shader::new(VERTEX_SHADER_SOURCE, FRAGMENT_SHADER_SOURCE);
    shader.set_int("gridLength", GRID_LENGTH as i32);
    let vao = setup_vertex_buffer();
    
    let mut button_states = InputStates {mouse_x: 0.0, mouse_y: 0.0, mouse_left: false};
    let mut mouse_last_x: f64 = 0.0;
    let mut mouse_last_y: f64 = 0.0;
    let mut mouse_last_left = false;
    
    
    let view_x: Arc<Mutex<i64>> = Arc::new(Mutex::new(0));
    let view_y: Arc<Mutex<i64>> = Arc::new(Mutex::new(0));
    
    
    let (tx, rx) = crossbeam_channel::bounded(1);
    let view_x_clone = Arc::clone(&view_x);
    let view_y_clone = Arc::clone(&view_y);

    let arr_primary = vtx_arr_primary.clone();
    let arr_secondary = vtx_arr_secondary.clone();
    let buffer_order_clone = buffer_order.clone();

    thread::spawn(move || {
        let s = r#"

"#;
        let rle = RLE::from_str(s).unwrap();
        grid.set_cell(50, 50, true);
        grid.set_cell(49, 50, true);
        grid.set_cell(49, 49, true);
        grid.set_cell(48, 50, true);
        grid.set_cell(50, 51, true);
        //rle.apply(&mut grid);
        loop {
            let now = Instant::now();
            let x = view_x_clone.lock().unwrap().clone();
            let y = view_y_clone.lock().unwrap().clone();

            let mut buf_order = false;
            if let Ok(val) = buffer_order_clone.lock() {
                buf_order = *val;
            }


            if buf_order {
                if let Ok(mut lock) = arr_secondary.lock() {
                    grid.get_grid(y, x, GRID_LENGTH, &mut *lock);
                }
            }
            else {
                if let Ok(mut lock) = arr_primary.lock() {
                    grid.get_grid(y, x, GRID_LENGTH, &mut *lock);
                }
            }
            

            grid.calc_next_generation();

            let elapsed = now.elapsed().as_micros();
            let delay_time = 50000;
            let sleep_time = if elapsed <= delay_time {(delay_time - elapsed) as u64} else { 0};

            tx.send("New Generation").unwrap();

            thread::sleep(time::Duration::from_micros(sleep_time));
        }
    });


    update_vertex_buffer(vao, &vec![0.0; VERTEX_ARRAY_SIZE]);

    while !window.should_close() {
        let now = Instant::now();

        if let Ok(_) = rx.try_recv() {
            if let Ok(mut val) = buffer_order.lock() {
                if *val {
                    if let Ok(lock) = vtx_arr_primary.lock() {
                        update_vertex_buffer(vao, &*lock);
                    }
                }
                else {
                    if let Ok(lock) = vtx_arr_secondary.lock() {
                        update_vertex_buffer(vao, &*lock);
                    }
                }

                *val = !*val;
            }
        }

        process_events(&mut window, &events, &mut button_states);


        if button_states.mouse_left {
            if !mouse_last_left {
                mouse_last_x = button_states.mouse_x;
                mouse_last_y = button_states.mouse_y;
            }

            let mut x = view_x.lock().unwrap();
            let mut y = view_y.lock().unwrap();
            *x -= (button_states.mouse_x - mouse_last_x).floor() as i64;
            *y -= (button_states.mouse_y - mouse_last_y).floor() as i64;
        }

        mouse_last_left = button_states.mouse_left;
        mouse_last_x = button_states.mouse_x;
        mouse_last_y = button_states.mouse_y;

        
        unsafe {
            gl::ClearColor(0.2, 0.3, 0.3, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);

            shader.use_shader();
            gl::BindVertexArray(vao);
            gl::DrawArrays(gl::TRIANGLES, 0, VERTEX_ARRAY_SIZE as i32);
            gl::BindVertexArray(0);
        }
        
        window.swap_buffers();
        glfw.poll_events();

        print!("\rRender Framerate: {0:>3} FPS", (1000000 / now.elapsed().as_micros()));
        io::stdout().flush().unwrap();
    }
}

fn process_events(window: &mut glfw::Window, events: &Receiver<(f64, glfw::WindowEvent)>, input_states: &mut InputStates) {
    for (_, event) in glfw::flush_messages(events) {
        match event {
            glfw::WindowEvent::FramebufferSize(width, height) => {
                // make sure the viewport matches the new window dimensions; note that width and
                // height will be significantly larger than specified on retina displays.
                unsafe { gl::Viewport(0, 0, width, height) }
            },
            glfw::WindowEvent::MouseButton(btn, action, mods) => {
                if btn == MouseButton::Button1 {
                    match action {
                        Action::Release => input_states.mouse_left = false,
                        Action::Press => input_states.mouse_left = true,
                        _ => ()
                    }
                }
            },
            glfw::WindowEvent::CursorPos(width, height) => {
                input_states.mouse_x = width;
                input_states.mouse_y = height;
            },
            glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => window.set_should_close(true),
            _ => {}
        }
    }
}

fn update_vertex_array(vtx_arr: &mut Vec<f32>, grid: Vec<bool>) {
    vtx_arr.clear();
    for val in grid {
        let num = if val { 1.0 } else { 0.0 };
        vtx_arr.push(num)
    }
}

fn setup_vertex_buffer() -> GLuint {
    let vao = unsafe {
    let (mut vbo, mut vao) = (0, 0);
        gl::GenVertexArrays(1, &mut vao);
        gl::GenBuffers(1, &mut vbo);
        // bind the Vertex Array Object first, then bind and set vertex buffer(s), and then configure vertex attributes(s).
        gl::BindVertexArray(vao);

        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);

        gl::VertexAttribPointer(0, 1, gl::FLOAT, gl::FALSE, 1 * mem::size_of::<GLfloat>() as GLsizei, ptr::null());
        //gl::VertexAttribPointer(1, 1, gl::FLOAT, gl::FALSE, 3 * mem::size_of::<GLfloat>() as GLsizei, (2 * std::mem::size_of::<f32>()) as *const gl::types::GLvoid);
        gl::EnableVertexAttribArray(0);
        //gl::EnableVertexAttribArray(1);

        // You can unbind the VAO afterwards so other VAO calls won't accidentally modify this VAO, but this rarely happens. Modifying other
        // VAOs requires a call to glBindVertexArray anyways so we generally don't unbind VAOs (nor VBOs) when it's not directly necessary.
        gl::BindVertexArray(0);

        vao
    };

    return vao
}

fn update_vertex_buffer(vao: GLuint, vtx_arr: &Vec<f32>) {
    unsafe {
        gl::BindVertexArray(vao);
        gl::BufferData(gl::ARRAY_BUFFER,
                       (vtx_arr.len() * mem::size_of::<GLfloat>()) as GLsizeiptr,
                       &vtx_arr[0] as *const f32 as *const c_void,
                       gl::STATIC_DRAW);

        gl::BindVertexArray(0);
    }
}