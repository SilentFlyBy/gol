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
    
    
    let mut vtx_arr: Vec<f32> = vec![0.0; VERTEX_ARRAY_SIZE];
    let shader_program = setup_shaders();
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

    thread::spawn(move || {
        let s = r#"
#C "Infinite LWSS hotel"
#C A new LWSS is added to the loop every 250 generations.
#C The old LWSSs reenter the stream at a point 12/25ths of the
#C way between the new LWSSs. Unlike earlier patterns of this
#C type, this one will not work (for long) if the old spaceships
#C re-enter exactly halfway between the new ones.
#C Jason Summers, 20 October 2002; c/3 portions by David Bell.
#C From "jslife" pattern collection.
x = 515, y = 298, rule = B3/S23
196boo141boo$196bobbobo133bobobbo$198boob3o129b3oboo$204bo127bo$198boo
b3o129b3oboo$198boobo133boboo3$163boo207boo$164bo207bo$162bo211bo$162b
oo209boo$154bo227bo$154b3o223b3o$157bo221bo$156boo10bo210boo$165bo3bo
6bo176boo$165bo4bo4b4o173b3o15boo$165bo3bo3b5obo73bo29bo67b3obo14boo$
166b3o3bo6boo71bobo27bobo65b3obo$166bobobbo3bob3o73bo29bo67b4o$162b3ob
obo3bobobboo173boo$167boo$258boo17boo$233boo23boo17boo23boo$175boo57bo
31booboo31bo57boo$176bo57bobo29bo3bo21bo7bobo57bo$173b3o59boo27bobo3bo
bo20bo6boo59b3o$173bo17boo51b3o11boo4boo5boo4boo10bobobbo49boo17bo$
171bobo18bo50booboo10boo17boo9b3o3bo49bo18bobo$171boo16b3o52bobboo46bo
49b3o16boo$189bo34boo18bo4bo33boo3b5obo16boo34bo$147boo76bo18bobobboo
15booboo12b4o5boo17bo76boo$147bo77bobo19bob3o15bobo15booboo19bobo77bo$
149boboo18boo53boo5bo13boobbo10boobo5boboo34boo53boo18boobo$148booboo
18bobo59bobo11bo3bo10booboo3booboo10bobo3bo71bobo18booboo$173bo59boo9b
3o3bo34bobb3obo70bo$148booboo20boo74bo12booboo3booboo12boo3boo68boo20b
ooboo$149bobo85boo8bo15bobo5bobo14bo9boo7bo77bobo$149bobo68boo14bobbo
7bo15bobo5bobo15bo7bobbo4bobo7boo68bobo$150booboo65boo15boo25bo7bo25b
oo6boo7boo65booboo$152bobo88b3o136bobo$152bo31boo165boo31bo$151boo31bo
167bo31boo$182bobo17bo149bobo$181bobo18bo133b3o14bobo$177boo3bo19boo
150bo3boo$177boo157bobo19boo$203b3o31boo59boo36bobo$203boo32boo59boo
36b3o$184boo13boo135boboo$180boobb3obboo5bo3boo11boo107boo7bo6boo17b3o
bo$179boo4bobobbo4bobo15boo107boo6b3ob4o18bo3boboo$180boboobo3bo4bo3bo
131boob4o19boo4bo$181boo3bo8bobo33boo71boo26b3o21bob4o$196bo34bobo69bo
bo33b3o$233bo69bo35bo$233boo67boo$158boo217boo$158boo217boo$164boo205b
oo$164boo56boo89boo56boo$222bobo87bobo$224bo87bo$162boo60boo85boo60boo
$162boo5boo40boo111boo40boo5boo$169boo40boo111boo40boo3$209boo115boo$
209boo115boo11$204bo$205boo$204boo$$328bo$326boo$327boo12$376bo$377bo$
377boo$378bo$375booboo$331bo42bo3bo$330bobo43boo$332boo42bo$332bo43boo
$324bobo4bobo39bobboo$324boboob5o40bo3bo$327boobo47boo$324booboobooboo
42bobbo$324b3o6boo42bo$326bobb4o44boobo$328bo3bo44bobbo3bo$330boo46b3o
bbobo$325bo3boo46b3o4boo$323b8o47bo6bo$328bo47boboobbobo$329bobbo46bo
bbobo$17bo311bobobo43b3o3b3o$4bo11bobo314bo51boo$3bobo10bobo308bo3bobb
o42b3o6boo$bboo12bo312boob3o43boo5bo$4bo321bo4bobbo39bo10bobb3o$bboo
11boo311b6o40bo4bo3b3obobo$bb3o11bobo10bobo293bobbo43boobobobo3bo4bo$
16boo10booboboo293b3oboo38bobobobbo8bo$bbobbo11bo9bo300b3obbo38bobob3o
6bo$bbobo13bobbo3b7o3bo289bo4boo40booboobo3booboo$bbo19bobo299boo3boob
3o38bo4boobo4bo$3bo15boo3bob7o293bo6boo38bobboboo3boobo$3bobo13bobo3b
oo5b4o291bo3boo43bobobobobo$4bo13boo7bobo3bo293boobo44booboboobb3o$3b
oo14bobo7bobobbo289boobo48boobboobb3o22bo$20bo10bo292b3o49b5o28booboo$
bbobbo26bobo290boo83bo3bo$bo408bo3bo$oobbo402bo3bo3bo$bb4o399b3o3bobob
o$bboboo397bob3o3bo3bo$bbobo405bo3bo$bb3o397bobo6b3o$bboo399b3o4b3o52b
o$3boboo397b3o3bo53bobo$6bo4bobbo246bobbo141boo3boo51bobo$6bo3bo124b4o
121bo145b3obbo54bo$4b3o3bo3bo120bo3bo120bo3bo145bo55boo$bboo6b4o121bo
124b4o201bobo$bboobbo129bobbo$5bo460b3o$3boo461b3o$3b3o460bo$84b3o375b
o4boo$84boo375bobboobbo$86bo376bob3o$83b3o376b4obo$21bo58boobbobo376b
oobo$20bobo56bobobbo293bo84boobo$19boo58bobobb3o292boo85bo$20bobo56bo
6b3o283b4o3boo82boo$20boo56boo6boo261bo23bo4bo83bobbo$23bo48bo5bobo
267bo27bo3bo78boobb3o$19bobbo48bobo11bobbo262bo4bo15bobb3o3bo80bobbo$
18bo52bo5b3o4bo242bo20booboo4bo13boboobo3bo82boo$17boo51bobbo3bo5boo3b
o237bobo19bobobobobboo17b4o$18bo51b3o5bo6boobbo238boo18bobo5bo19b3o$
18b3o49bo3bo3bobb6o239bobo17b3obooboob3o13boobbo$19bo51b4o11bobbo237b
oo16b3o9boo13b3obboo$21bo52boo3boobb3obo237bo16boo3boobo3boo21bo65bo$
19boo53boo4boobo3boo77bo159bobbo15bo3boo3boobo13bobobbo65bo$19boo49b5o
6b4ob3o76bobo162bo14bo9b5o11bobo71bo4bo$21bo51bo8bo3b3o76bo164boo10boo
3bob3o3bobb3o12boo67booboo4bo$69bobbo91bobbo162bo14b5obo4bo4bo8bo4bo
66bobobobobboo$69boo8b3obboo78b3o161b3o4bo12bobo5bo4bo10b4o66bobo5bo$
69boo8b3obbo79bo3bo160bo6boo10boboo6bobbo8bob3o65b3obooboob3o$70bo8boo
5bo78b4o158bo8boo20bo10boo68b3o9boo$65bobboobboo10boo10bo71boo158boo5b
o14bo6boo10b4o42bo20boo3boobo3boo$63boo4boo6boobbo13bobo70boo158boo7bo
12bo20bobo42boo21bo3boo3boobo$63boo13bo4b3o8boo68b5o151boo5bo7bobo10bo
bbobb3o14bobo42boo11b4o6bo9b5o$70bobo5bo3b3obo9bo70bo13bo8boo127b3o15b
o11boo3b3o17bo55bo5boo3bob3o3bobb3o$64b3o3boo10bo3bo7boo67bobbo13bobo
7b3obo124boobobbo8bobo11bobo21bobo39b3o10bo5bo5b5obo4bo4bo$64b3obbobbo
6boo3bo9b3o66boo15boo6bo4b3o127bobo8bobo19boo16bo39b3o12bobboo8bobo5bo
4bo$70bo8bobbo3bo55bo20boo15bo5b7oboo121b3o7boo3b4o14boo4boo14bo53bob
oob4o7boboo6bobbo$62boo6bo7boobob3o9bobo42boo22bo16bobobo8boo124b7obbo
bobo13boobboobbo17boo42b3o6b3o6bo16bo$62bo16bo6b3o6b3o42boo17bobboobb
oo13bobobobb7o124boo8bobobo16bo22boo42bobo9b3oboboo7bo6boo$59bobbo6boo
bo7b4oboobo53bo14boo4boo14b4o3boo7b3o121boob7o5bo15boo20bo55bo3bobbo8b
o$59bo4bo5bobo8boobbo12b3o39bo16boo19bobo8bobo127b3o4bo6boo15boo66b3o
9bo3boo6bobbobb3o$59bo4bo4bob5o5bo5bo10b3o39bobo21bobo11bobo8bobboboo
124bob3o7bobo13bobbo67boo7bo3bo10boo3b3o$60b3obbo3b3obo3boo5bo55bo17b
3o3boo11bo15b3o127boo8bo13bo70bo9bob3o3bo5bobo$61b5o9bo6b4o11boo42bobo
14b3obbobbo10bobo7bo5boo151b5o68boo8b3o4bo13boo$63boboo3boo3bo21boo42b
obo20bo12bo7boo158boo70bobo13bobboo6boo4boo$65boo3boboo3boo20bo42b4o
10boo6bo14bo5boo158boo71bo10boo10boobboobbo$62boo9b3o68boo10bo20boo8bo
158b4o78bo5boo8bo$62b3obooboob3o65b3obo8bobbo6boobo10boo6bo160bo3bo79b
obb3o8boo$64bo5bobo66b4o10bo4bo5bobo12bo4b3o161b3o78boobb3o8boo$62boo
bbobobobo66bo4bo8bo4bo4bob5o14bo162bobbo91bobbo$63bo4booboo67boo12b3o
bbo3b3obo3boo10boo164bo76b3o3bo8bo51bo$64bo4bo71bobo11b5o9bo14bo10boo
150bobo76b3ob4o6b5o49boo$72bo65bobbobo13boboo3boo3bo15bobbo5boo152bo
77boo3boboo4boo53boo$71bo65bo21boo3boboo3boo16bo6bo230bob3obboo3boo52b
o$136boobb3o13boo9b3o16boo237bobbo11b4o51bo$138bobboo13b3obooboob3o17b
obo239b6obbo3bo3bo49b3o$136b3o19bo5bobo18boo238bobboo6bo5b3o51bo$135b
4o17boobbobobobo19bobo237bo3boo5bo3bobbo51boo$50boo82bo3boboobo13bo4b
ooboo20bo242bo4b3o5bo52bo$49bobbo80bo3b3obbo15bo4bo262bobbo11bobo48bo
bbo$49b3obboo78bo3bo27bo267bobo5bo48bo$49bobbo83bo4bo23bo156b3o102boo
6boo56boo$50boo82boo3b4o179bo103b3o6bo56bobo$48bo85boo64bo122bo104b3o
bbobo58boo$48boboo84bo63bo229bobbobo56bobo$48boboo147bobo226bobobboo
58bo$47bob4o145boboo227b3o$47b3obo145boo3bobo223bo$46bobboobbo143bo3bo
3boo222boo$46boo4bo144boobo6bo220b3o$48bo151bo3bo3bo4bo295b3o$46b3o
148b3o5bobo3boo297boo$46b3o147bo3b3oboo5b3o295bo$195b5o3b3o302bobboo$
47bobo143bo7bobboo55b4o246boo$47boo55bo89bo3b4ob3o54bo3bo243b3o$48bo
54bobb3o85bo3boo64bo243bo$48bobo51boo3boo151bobbo244bo$48bobo53bo3b3o
83b3o311boobo$49bo52b3o4b3o399boo$101b3o6bobo77b4obboo126boo184b3o$
100bo3bo84boboo131boo184bobo$99bo3bo3b3obo77bobboobboo311boobo$99bobob
o3b3o80boo317b4o$99bo3bo3bo214boo40boo144bobboo$100bo3bo217boo40boo5b
oo140bo$100bo3bo83boo119boo60boo107bobo26bobbo$101booboo28b5o49b3o119b
o172bo10bo$105bo22b3obboobboo48boboo119bobo167bobbobo7bobo14boo$128b3o
bbooboboo44boboo123boo56boo110bo3bobo7boo13bo$130bobobobobo43boo3bo
181boo108b4o5boo3bobo13bobo$128boboo3boobobbo38boo6bo186boo105b7obo3b
oo15bo$128bo4boboo4bo38b3oboo3boo184boo113bobo19bo$128booboo3bobooboo
40boo4bo110boo56b3o118bo3b7o3bobbo13bobo$129bo6b3obobo38bobb3o114bo54b
o4bo125bo9bo11bobbo$126bo8bobbobobo38boob3o114bobo54boboo118booboboo
10boo$126bo4bo3boboboboo43bobbo112boo51b3o125bobo10bobo11b3o$125bobob
3o3bo4bo40b6o148bo18bo143boo11boo$124b3obbo10bo39bobbo4bo131boo12bobo
173bo$129bo5boo43b3oboo134boo14bo161bo12boo$127boo6b3o42bobbo3bo145boo
161bobo10bobo$128boo51bo114boo36bobbo158bobo11bo$129b3o3b3o43bobobo
110boo35boobobo158bo$130bobobbo46bobbo147bo5bo16boo$130bobobboobo47bo
150boo13bo3boo$129bo6bo47b8o141bo4bo12bobo$129boo4b3o46boo3bo143bo3bo
12bobo$129bobobb3o46boo149boobo12bo31boo$130bo3bobbo44bo3bo119boo27boo
12boo31bo$134boboo44b4obbo118boo71bobo$137bo42boo6b3o79bo25boo8bo6boo
65booboo$134bobbo42booboobooboo78bobo23bobbo14boo68bobo$135boo47boboo
81bobo24boo85bobo$136bo3bo40b5oboobo77booboo14b3o70boo20booboo$137boo
bbo39bobo4bobo95bo4bo69bo$137boo43bo85booboo12bo5bo69bobo18booboo$138b
o42boo86boboo12bobo19boo53boo18boobo$137boo43bobo82bo16boo3b3o15bobo
77bo$136bo3bo42bo83boo14boo5bo18bo76boo$135booboo144boo23boo34bo$136bo
148bob3o53b3o16boo$136boo137boo11bo53bo18bobo$137bo131boo4boo10bo54boo
17bo$138bo129bobo17bo3boo4boo59b3o$268bo25bo3bobo57bo$267boo23boo6bo
57boo$275boo23boo$275boo$$351bobo$281bo65bo3bobo$280bobo70boo$281bo71b
o14boo$347bobobo16boo$347b3obobbo$348bo3bo24boo$377bo$378b3o$380bo$
371boo$372bo$370bo$370boo3$333boboo$331b3oboo$330bo$331b3oboo$333bobo
bbo$337boo!
"#;
        let rle = RLE::from_str(s).unwrap();
        /*grid.set_cell(50, 50, true);
        grid.set_cell(49, 50, true);
        grid.set_cell(49, 49, true);
        grid.set_cell(48, 50, true);
        grid.set_cell(50, 51, true);*/
        rle.apply(&mut grid);
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
            gl::DrawArrays(gl::TRIANGLES, 0, vtx_arr.len() as i32);
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

        gl::UseProgram(shader_program);

        let uniform_name = CString::new("gridLength").unwrap();
        let uniform_location = gl::GetUniformLocation(shader_program, uniform_name.as_ptr());
        gl::Uniform1i(uniform_location, GRID_LENGTH as i32);

        shader_program
    };

    shader_program
}

fn update_vertex_array(vtx_arr: &mut Vec<f32>, grid: Vec<bool>) {
    vtx_arr.clear();
    for val in grid {
        let num = if val { 1.0 } else { 0.0 };
        vtx_arr.push(num)
    }
    /*for row in 0..GRID_LENGTH {
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
    }*/
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