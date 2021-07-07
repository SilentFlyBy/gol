use std::{ffi::c_void, mem, ptr};

use gl::types::{GLfloat, GLsizei, GLsizeiptr, GLuint, GLvoid};
use nalgebra_glm::{TVec, Vec2, Vec3};

extern crate nalgebra_glm;

pub type Mesh3 = Mesh<Vec3>;
pub type Mesh2 = Mesh<Vec2>;

pub struct Mesh<T> {
    vertices: Vec<T>,
    indices: Vec<GLuint>,
    gl_vertex_array_object: GLuint,
    gl_vertex_buffer_object: GLuint,
    gl_vertex_element_object: GLuint
}

impl<T> Mesh<T> {
    pub fn new(vertices: Vec<T>, indices: Vec<GLuint>) -> Mesh<T> {
        let (gl_vertex_array_object, gl_vertex_buffer_object, gl_vertex_element_object) = setup_vertex_buffer::<T>();

        Mesh {
            vertices,
            indices,
            gl_vertex_array_object,
            gl_vertex_buffer_object,
            gl_vertex_element_object
        }
    }

    pub fn update_vertex_data(&self) {
        println!("{}", self.vertices.len());
        self.use_mesh();
        unsafe {
            gl::BufferData(gl::ARRAY_BUFFER,
                (self.vertices.len() * mem::size_of::<T>()) as GLsizeiptr,
                self.vertices.as_ptr() as *const GLvoid,
                gl::STATIC_DRAW);
        }
        detach_mesh();
    }

    pub fn update_index_data(&self) {
        self.use_mesh();
        unsafe {
            gl::BufferData(gl::ELEMENT_ARRAY_BUFFER,
                (self.indices.len() * mem::size_of::<GLuint>()) as GLsizeiptr,
                self.indices.as_ptr() as *const GLvoid,
                gl::STATIC_DRAW);
        }
        detach_mesh();
    }

    pub fn use_mesh(&self) {
        unsafe {
            gl::BindVertexArray(self.gl_vertex_array_object);
        }
    }
}

fn setup_vertex_buffer<T>() -> (GLuint, GLuint, GLuint) {
    unsafe {
        let (mut vbo, mut vao, mut ebo) = (0, 0, 0);
        gl::GenVertexArrays(1, &mut vao);
        gl::BindVertexArray(vao);
        gl::GenBuffers(1, &mut vbo);
        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);

        gl::EnableVertexAttribArray(0);
        
        gl::VertexAttribPointer(0, 2, gl::FLOAT, gl::FALSE, mem::size_of::<T>() as GLsizei, ptr::null());

        gl::GenBuffers(1, &mut ebo);
        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);


        detach_mesh();

        return (vao, vbo, ebo)
    };
}


pub fn detach_mesh() {
    unsafe {
        gl::BindVertexArray(0);
    }
}