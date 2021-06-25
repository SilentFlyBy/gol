extern crate glfw;

extern crate gl;
use gl::types::*;

use std::ffi::CString;
use std::os::raw::c_void;
use glfw::{Action, Context, Key};
use std::{ptr, sync::mpsc::Receiver, mem, str};
use chunk::ChunkGrid;

mod chunk;

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

const VERTEX_SIZE: usize = 2;
const COLOR_SIZE: usize = 1;
const CELL_SIZE: usize = 2 * 3 * (VERTEX_SIZE + COLOR_SIZE);
const VERTEX_ARRAY_SIZE: usize = 100 * 100 * CELL_SIZE;

fn main() {
    let chunk_grid = ChunkGrid::new();
    chunk_grid.compute_next_generation(true);

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

    gl::load_with(|symbol| window.get_proc_address(symbol) as *const _);


    let mut vtx_arr: [f32; VERTEX_ARRAY_SIZE] = [0.0; VERTEX_ARRAY_SIZE];
    update_vertex_array(&mut vtx_arr, &chunk_grid);

    let shader_program = setup_shaders();
    let vao = setup_vertex_buffer();
    update_vertex_buffer(vao, &vtx_arr);



    while !window.should_close() {
        process_events(&mut window, &events);
        
        unsafe {
            gl::ClearColor(0.2, 0.3, 0.3, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);

            // draw our first triangle
            gl::UseProgram(shader_program);
            gl::BindVertexArray(vao);
            gl::DrawArrays(gl::TRIANGLES, 0, VERTEX_ARRAY_SIZE as i32 / 2);
            gl::BindVertexArray(0);
        }
        
        window.swap_buffers();
        glfw.poll_events();
    }
}

fn process_events(window: &mut glfw::Window, events: &Receiver<(f64, glfw::WindowEvent)>) {
    for (_, event) in glfw::flush_messages(events) {
        match event {
            glfw::WindowEvent::FramebufferSize(width, height) => {
                // make sure the viewport matches the new window dimensions; note that width and
                // height will be significantly larger than specified on retina displays.
                unsafe { gl::Viewport(0, 0, width, height) }
            }
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

fn update_vertex_array(vtx_arr: &mut [f32; VERTEX_ARRAY_SIZE], chunk_grid: &ChunkGrid) {
    for row in 0..100 {
        for col in 0..100 {               
            let row_index = row * 100 * CELL_SIZE;
            let cell_index = row_index + (col * CELL_SIZE);

            let x_pos = col as f32 / 100.0;
            let y_pos = row as f32 / 100.0;


            let cell_active = chunk_grid.get_cell(col as i64, row as i64, true) == Some(true);
            let active_value = if cell_active {1.0} else {0.0};

            // FIRST TRIANGLE

            // bottom left corner
            vtx_arr[cell_index] = x_pos;
            vtx_arr[cell_index + 1] = y_pos;
            vtx_arr[cell_index + 2] = active_value;

            // top left corner
            vtx_arr[cell_index + 3] = x_pos;
            vtx_arr[cell_index + 4] = y_pos + 0.01;
            vtx_arr[cell_index + 5] = active_value;

            // top right corner
            vtx_arr[cell_index + 6] = x_pos + 0.01;
            vtx_arr[cell_index + 7] = y_pos + 0.01;
            vtx_arr[cell_index + 8] = active_value;

            // SECOND TRIANGLE

            // top right corner
            vtx_arr[cell_index + 9] = x_pos + 0.01;
            vtx_arr[cell_index + 10] = y_pos + 0.01;
            vtx_arr[cell_index + 11] = active_value;

            // bottom right corner
            vtx_arr[cell_index + 12] = x_pos + 0.01;
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

fn update_vertex_buffer(vao: GLuint, vtx_arr: &[f32; VERTEX_ARRAY_SIZE]) {
    unsafe {
        gl::BindVertexArray(vao);
        gl::BufferData(gl::ARRAY_BUFFER,
                       (vtx_arr.len() * mem::size_of::<GLfloat>()) as GLsizeiptr,
                       &vtx_arr[0] as *const f32 as *const c_void,
                       gl::STATIC_DRAW);

        gl::BindVertexArray(0);
    }
}