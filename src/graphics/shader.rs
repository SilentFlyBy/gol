use std::{ffi::CString, ptr};

use gl::types::{GLchar, GLint, GLuint};

pub struct Shader {
    vertex_code: String,
    fragment_code: String,
    gl_shader_program: GLuint
}

impl Shader {
    pub fn new(vertex: &str, fragment: &str) -> Shader {
        let gl_shader_program = setup_gl_shaders(vertex, fragment);

        Shader {
            vertex_code: vertex.to_string(),
            fragment_code: fragment.to_string(),
            gl_shader_program
        }
    }

    pub fn use_shader(&self) {
        unsafe {
            gl::UseProgram(self.gl_shader_program);
        }
    }

    pub fn set_int(&self, uniform_name: &str, value: GLint) {
        let uniform_location = self.get_uniform_location(uniform_name);
        self.use_shader();

        unsafe {
            gl::Uniform1i(uniform_location, value);
        }
        detach_shader();
    }

    fn get_uniform_location(&self, name: &str) -> GLint {
        self.use_shader();
        let location = unsafe {
            let uniform_cstr = CString::new(name).unwrap();
            gl::GetUniformLocation(self.gl_shader_program, uniform_cstr.as_ptr())
        };
        detach_shader();

        return location
    }
}

fn setup_gl_shaders(vertex_code: &str, fragment_code: &str) -> GLuint {
    let shader_program = unsafe {
    // build and compile our shader program
        // ------------------------------------
        // vertex shader
        let vertex_shader = gl::CreateShader(gl::VERTEX_SHADER);
        let c_str_vert = CString::new(vertex_code.as_bytes()).unwrap();
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
        let c_str_frag = CString::new(fragment_code.as_bytes()).unwrap();
        gl::ShaderSource(fragment_shader, 1, &c_str_frag.as_ptr(), ptr::null());
        gl::CompileShader(fragment_shader);
        // check for shader compile errors
        gl::GetShaderiv(fragment_shader, gl::COMPILE_STATUS, &mut success);
        if success != gl::TRUE as GLint {
            gl::GetShaderInfoLog(fragment_shader, 512, ptr::null_mut(), info_log.as_mut_ptr() as *mut GLchar);
            println!("ERROR::SHADER::FRAGMENT::COMPILATION_FAILED\n{}", std::str::from_utf8(&info_log).unwrap());
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
            println!("ERROR::SHADER::PROGRAM::COMPILATION_FAILED\n{}", std::str::from_utf8(&info_log).unwrap());
        }
        gl::DeleteShader(vertex_shader);
        gl::DeleteShader(fragment_shader);

        shader_program
    };

    shader_program
}

pub fn detach_shader() {
    unsafe {
        gl::UseProgram(0);
    }
}