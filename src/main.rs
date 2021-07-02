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

mod grid;
mod rle;

const VERTEX_SHADER_SOURCE: &str = r#"
    #version 330 core
    layout (location = 0) in vec2 aPos;
    layout (location = 1) in float vertexActive;

    out float fragmentActive;

    void main() {
        fragmentActive = vertexActive;
        gl_Position = vec4(aPos.x, aPos.y, 0.0, 1.0);
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

const VERTEX_SIZE: usize = 2;
const COLOR_SIZE: usize = 1;
const CELL_SIZE: usize = 2 * 3 * (VERTEX_SIZE + COLOR_SIZE);
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
    
    
    let mut vtx_arr: Vec<f32> = vec![0.0; VERTEX_ARRAY_SIZE];
    let shader_program = setup_shaders();
    let vao = setup_vertex_buffer();
    
    let mut button_states = InputStates {mouse_x: 0.0, mouse_y: 0.0, mouse_left: false};
    let mut mouse_last_x: f64 = 0.0;
    let mut mouse_last_y: f64 = 0.0;
    let mut mouse_last_left = false;
    
    
    let mut view_x: Arc<Mutex<i64>> = Arc::new(Mutex::new(0));
    let mut view_y: Arc<Mutex<i64>> = Arc::new(Mutex::new(0));
    
    
    let (tx, rx) = crossbeam_channel::bounded(1);
    let view_x_clone = Arc::clone(&view_x);
    let view_y_clone = Arc::clone(&view_y);

    thread::spawn(move || {
        let s = r#"
#N 104P177 reactions
#O Jason Summers and Nicolay Beluchenko
#C Some reactions involving a glider and 105P177
#C http://www.conwaylife.com/wiki/index.php?title=104P177
x = 377, y = 173, rule = b3/s23
105bobo4bo14bo4bobo73bobo4bo14bo4bobo139b$7bobo4bo14bo4bobo68bobo3b3o
12b3o3bobo73bobo3b3o12b3o3bobo139b$7bobo3b3o12b3o3bobo69bo4bo2bo10bo2b
o4bo75bo4bo2bo10bo2bo4bo140b$8bo4bo2bo10bo2bo4bo76b3o10b3o87b3o10b3o
146b$14b3o10b3o347b3$98b2o40b2o59b2o40b2o132b$2o40b2o56bo38bo63bo38bo
134b$2bo38bo56b2o40b2o59b2o40b2o132b$2o40b2o333b3$99b2o38b2o61b2o38b2o
133b$b2o38b2o55b2obo36bob2o59b2obo36bob2o132b$2obo36bob2o55bobo36bobo
61bobo36bobo133b$bobo36bobo57b2o36b2o63b2o36b2o134b$2b2o36b2o335b10$
100b2o36b2o63b2o36b2o134b$2b2o36b2o57bobo36bobo61bobo36bobo133b$bobo
36bobo55b2obo36bob2o59b2obo36bob2o132b$2obo36bob2o55b2o38b2o61b2o38b2o
133b$b2o38b2o334b3$98b2o40b2o59b2o40b2o132b$2o40b2o56bo38bo63bo38bo
134b$2bo38bo56b2o40b2o59b2o40b2o132b$2o40b2o333b3$112b3o10b3o87b3o10b
3o146b$14b3o10b3o76bo4bo2bo10bo2bo4bo75bo4bo2bo10bo2bo4bo140b$8bo4bo2b
o10bo2bo4bo69bobo3b3o12b3o3bobo73bobo3b3o12b3o3bobo139b$7bobo3b3o12b3o
3bobo68bobo4bo14bo4bobo73bobo4bo14bo4bobo139b$7bobo4bo14bo4bobo340b12$
54b3o320b$54bo322b$55bo321b$187b2o81bo106b$186b2o81b2o106b$188bo80bobo
105b26$7bobo4bo14bo4bobo340b$7bobo3b3o12b3o3bobo274bobo4bo14bo4bobo36b
$8bo4bo2bo10bo2bo4bo68bobo4bo14bo4bobo81bobo4bo14bo4bobo66bobo3b3o12b
3o3bobo36b$14b3o10b3o74bobo3b3o12b3o3bobo81bobo3b3o12b3o3bobo67bo4bo2b
o10bo2bo4bo37b$105bo4bo2bo10bo2bo4bo83bo4bo2bo10bo2bo4bo74b3o10b3o43b$
111b3o10b3o95b3o10b3o139b2$2o40b2o333b$2bo38bo262b2o40b2o29b$2o40b2o
53b2o40b2o67b2o40b2o54bo38bo31b$99bo38bo71bo38bo54b2o40b2o29b$97b2o40b
2o67b2o40b2o125b2$b2o38b2o334b$2obo36bob2o261b2o38b2o30b$bobo36bobo55b
2o38b2o69b2o38b2o53b2obo36bob2o29b$2b2o36b2o55b2obo36bob2o67b2obo36bob
2o53bobo36bobo30b$98bobo36bobo69bobo36bobo55b2o36b2o31b$99b2o36b2o71b
2o36b2o127b5$144b3o230b$144bo232b$145bo228b3o$374bo2b$2b2o36b2o333bob$
bobo36bobo263b2o36b2o31b$2obo36bob2o55b2o36b2o71b2o36b2o55bobo36bobo
30b$b2o38b2o55bobo36bobo69bobo36bobo53b2obo36bob2o29b$97b2obo36bob2o
67b2obo36bob2o53b2o38b2o30b$98b2o38b2o69b2o38b2o126b2$2o40b2o333b$2bo
38bo262b2o40b2o29b$2o40b2o53b2o40b2o67b2o40b2o54bo38bo31b$99bo38bo71bo
38bo54b2o40b2o29b$97b2o40b2o67b2o40b2o125b2$14b3o10b3o347b$8bo4bo2bo
10bo2bo4bo282b3o10b3o43b$7bobo3b3o12b3o3bobo74b3o10b3o95b3o10b3o74bo4b
o2bo10bo2bo4bo37b$7bobo4bo14bo4bobo68bo4bo2bo10bo2bo4bo83bo4bo2bo10bo
2bo4bo67bobo3b3o12b3o3bobo36b$104bobo3b3o12b3o3bobo81bobo3b3o12b3o3bob
o66bobo4bo14bo4bobo36b$104bobo4bo14bo4bobo81bobo4bo14bo4bobo132b17$85b
2o290b$84b2o291b$86bo290b19$295b3o79b$295bo81b$296bo!
"#;
        let rle = RLE::from_str(s).unwrap();
        /*grid.set_cell(50, 50, true);
        grid.set_cell(49, 50, true);
        grid.set_cell(49, 49, true);
        grid.set_cell(48, 50, true);
        grid.set_cell(50, 51, true);*/
        rle.set_grid(&mut grid);
        loop {
            let now = Instant::now();
            let x = view_x_clone.lock().unwrap().clone();
            let y = view_y_clone.lock().unwrap().clone();
            match tx.try_send(grid.get_grid(y, x, GRID_LENGTH)) {
                Ok(_) => (),
                Err(_) => (),
            };
            grid.calc_next_generation();

            let elapsed = now.elapsed().as_micros();
            let delay_time = 50000;
            let sleep_time = if elapsed <= delay_time {(delay_time - elapsed) as u64} else { 0};

            thread::sleep(time::Duration::from_micros(sleep_time));
        }
    });


    update_vertex_buffer(vao, &vtx_arr);

    while !window.should_close() {
        let now = Instant::now();
        
        match rx.try_recv() {
            Ok(result) => {
                update_vertex_array(&mut vtx_arr, result);
                update_vertex_buffer(vao, &vtx_arr);
            },
            Err(_) => (),
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

            gl::UseProgram(shader_program);
            gl::BindVertexArray(vao);
            gl::DrawArrays(gl::TRIANGLES, 0, VERTEX_ARRAY_SIZE as i32 / 2);
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

fn setup_shaders() -> GLuint {
    let shader_program = unsafe {
    // build and compile our shader program
        // ------------------------------------
        // vertex shader
        let vertex_shader = gl::CreateShader(gl::VERTEX_SHADER);
        let c_str_vert = CString::new(VERTEX_SHADER_SOURCE.as_bytes()).unwrap();
        gl::ShaderSource(vertex_shader, 1, &c_str_vert.as_ptr(), ptr::null());
        gl::CompileShader(vertex_shader);

        // check for shader compile errors
        let mut success = gl::FALSE as GLint;
        let mut info_log = Vec::with_capacity(512);
        info_log.set_len(512 - 1); // subtract 1 to skip the trailing null character
        gl::GetShaderiv(vertex_shader, gl::COMPILE_STATUS, &mut success);
        if success != gl::TRUE as GLint {
            gl::GetShaderInfoLog(vertex_shader, 512, ptr::null_mut(), info_log.as_mut_ptr() as *mut i8);
            println!("ERROR::SHADER::VERTEX::COMPILATION_FAILED\n{}", std::str::from_utf8(&info_log).unwrap());
        }

        // fragment shader
        let fragment_shader = gl::CreateShader(gl::FRAGMENT_SHADER);
        let c_str_frag = CString::new(FRAGMENT_SHADER_SOURCE.as_bytes()).unwrap();
        gl::ShaderSource(fragment_shader, 1, &c_str_frag.as_ptr(), ptr::null());
        gl::CompileShader(fragment_shader);
        // check for shader compile errors
        gl::GetShaderiv(fragment_shader, gl::COMPILE_STATUS, &mut success);
        if success != gl::TRUE as GLint {
            gl::GetShaderInfoLog(fragment_shader, 512, ptr::null_mut(), info_log.as_mut_ptr() as *mut GLchar);
            println!("ERROR::SHADER::FRAGMENT::COMPILATION_FAILED\n{}", str::from_utf8(&info_log).unwrap());
        }

        // link shaders
        let shader_program: GLuint = gl::CreateProgram();
        gl::AttachShader(shader_program, vertex_shader);
        gl::AttachShader(shader_program, fragment_shader);
        gl::LinkProgram(shader_program);
        // check for linking errors
        gl::GetProgramiv(shader_program, gl::LINK_STATUS, &mut success);
        if success != gl::TRUE as GLint {
            gl::GetProgramInfoLog(shader_program, 512, ptr::null_mut(), info_log.as_mut_ptr() as *mut GLchar);
            println!("ERROR::SHADER::PROGRAM::COMPILATION_FAILED\n{}", str::from_utf8(&info_log).unwrap());
        }
        gl::DeleteShader(vertex_shader);
        gl::DeleteShader(fragment_shader);

        shader_program
    };

    shader_program
}

fn update_vertex_array(vtx_arr: &mut Vec<f32>, grid: Vec<bool>) {
    for row in 0..GRID_LENGTH {
        for col in 0..GRID_LENGTH {               
            let row_index = row * GRID_LENGTH * CELL_SIZE;
            let cell_index = row_index + (col * CELL_SIZE);

            let x_pos = -1.0 + ((col as f32 / GRID_LENGTH as f32) * 2.0);
            let y_pos =  1.0 - ((row as f32 / GRID_LENGTH as f32) * 2.0);

            let cell_active = grid[(row * GRID_LENGTH) + col];
            //let cell_active = ((col + row) % 2) == 0;
            let active_value = if cell_active {1.0} else {0.0};

            let step_size: f32 = 0.02;

            // FIRST TRIANGLE

            // bottom left corner
            vtx_arr[cell_index] = x_pos;
            vtx_arr[cell_index + 1] = y_pos;
            vtx_arr[cell_index + 2] = active_value;

            // top left corner
            vtx_arr[cell_index + 3] = x_pos;
            vtx_arr[cell_index + 4] = y_pos - step_size;
            vtx_arr[cell_index + 5] = active_value;

            // top right corner
            vtx_arr[cell_index + 6] = x_pos + step_size;
            vtx_arr[cell_index + 7] = y_pos - step_size;
            vtx_arr[cell_index + 8] = active_value;

            // SECOND TRIANGLE

            // top right corner
            vtx_arr[cell_index + 9] = x_pos + step_size;
            vtx_arr[cell_index + 10] = y_pos - step_size;
            vtx_arr[cell_index + 11] = active_value;

            // bottom right corner
            vtx_arr[cell_index + 12] = x_pos + step_size;
            vtx_arr[cell_index + 13] = y_pos;
            vtx_arr[cell_index + 14] = active_value;

            // top left corner
            vtx_arr[cell_index + 15] = x_pos;
            vtx_arr[cell_index + 16] = y_pos;
            vtx_arr[cell_index + 17] = active_value;
        }
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

        gl::VertexAttribPointer(0, 2, gl::FLOAT, gl::FALSE, 3 * mem::size_of::<GLfloat>() as GLsizei, ptr::null());
        gl::VertexAttribPointer(1, 1, gl::FLOAT, gl::FALSE, 3 * mem::size_of::<GLfloat>() as GLsizei, (2 * std::mem::size_of::<f32>()) as *const gl::types::GLvoid);
        gl::EnableVertexAttribArray(0);
        gl::EnableVertexAttribArray(1);

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