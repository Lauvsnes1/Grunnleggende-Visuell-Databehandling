extern crate nalgebra_glm as glm;
use std::{ mem, ptr, os::raw::c_void };
use std::thread;
use std::sync::{Mutex, Arc, RwLock};

mod shader;
mod util;

use glutin::event::{Event, WindowEvent, DeviceEvent, KeyboardInput, ElementState::{Pressed, Released}, VirtualKeyCode::{self, *}};
use glutin::event_loop::ControlFlow;

const SCREEN_W: u32 = 800;
const SCREEN_H: u32 = 600;

// == // Helper functions to make interacting with OpenGL a little bit prettier. You *WILL* need these! // == //
// The names should be pretty self explanatory
fn byte_size_of_array<T>(val: &[T]) -> isize {
    std::mem::size_of_val(&val[..]) as isize
}

// Get the OpenGL-compatible pointer to an arbitrary array of numbers
fn pointer_to_array<T>(val: &[T]) -> *const c_void {
    &val[0] as *const T as *const c_void
}

// Get the size of the given type in bytes
fn size_of<T>() -> i32 {
    mem::size_of::<T>() as i32
}

// Get an offset in bytes for n units of type T
fn offset<T>(n: u32) -> *const c_void {
    (n * mem::size_of::<T>() as u32) as *const T as *const c_void
}

// Get a null pointer (equivalent to an offset of 0)
// ptr::null()



// == // Modify and complete the function below for the first task
unsafe fn CreateVAO(vertex_coordinates: &Vec<f32>, indices_array: &Vec<u32>, color_array: &Vec<f32>) -> u32 {

    //Setup VAO
    //Genereate vertex arrays
    let mut a: u32 = 0;
    let mut b: u32 =0;
    gl::GenVertexArrays(1, &mut a);
    gl::BindVertexArray(a);

    //Creating VBO
    gl::GenBuffers(1, &mut b);
    gl::BindBuffer(gl::ARRAY_BUFFER, b);

    //Filling the VBO with data
    gl::BufferData(gl::ARRAY_BUFFER, 
        byte_size_of_array(vertex_coordinates),
        pointer_to_array(vertex_coordinates),
         gl::STATIC_DRAW);

    //Set Vertex Attribute Pointer
    gl::VertexAttribPointer(0,
        3,
        gl::FLOAT,
        gl::FALSE,
        size_of::<f32>()*3,
        ptr::null());

    //Enable the vertex attributes
    gl::EnableVertexAttribArray(0);

    //Color Buffer
    let mut f: u32 = 0;
    gl::GenBuffers(1, &mut f);
    gl::BindBuffer(gl::ARRAY_BUFFER,f);

    //fill Color Buffer
    gl::BufferData(gl::ARRAY_BUFFER,
    byte_size_of_array(color_array),
    pointer_to_array(color_array),
    gl::STATIC_DRAW);
    gl::VertexAttribPointer(2,
        4,
        gl::FLOAT,
        gl::FALSE,
        size_of::<f32>()*4,
        ptr::null());

    //enable vertex attributes
    gl::EnableVertexAttribArray(2);


    //index buffer
    let mut d: u32 = 0;
    gl::GenBuffers(1, &mut d);
    gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, d);
    gl::BufferData(gl::ELEMENT_ARRAY_BUFFER,
        byte_size_of_array(indices_array),
        pointer_to_array(indices_array),
        gl::STATIC_DRAW);


    return a;

 } 

fn main() {
    // Set up the necessary objects to deal with windows and event handling
    let el = glutin::event_loop::EventLoop::new();
    let wb = glutin::window::WindowBuilder::new()
        .with_title("Gloom-rs")
        .with_resizable(false)
        .with_inner_size(glutin::dpi::LogicalSize::new(SCREEN_W, SCREEN_H));
    let cb = glutin::ContextBuilder::new()
        .with_vsync(true);
    let windowed_context = cb.build_windowed(wb, &el).unwrap();
    // Uncomment these if you want to use the mouse for controls, but want it to be confined to the screen and/or invisible.
    // windowed_context.window().set_cursor_grab(true).expect("failed to grab cursor");
    // windowed_context.window().set_cursor_visible(false);

    // Set up a shared vector for keeping track of currently pressed keys
    let arc_pressed_keys = Arc::new(Mutex::new(Vec::<VirtualKeyCode>::with_capacity(10)));
    // Make a reference of this vector to send to the render thread
    let pressed_keys = Arc::clone(&arc_pressed_keys);

    // Set up shared tuple for tracking mouse movement between frames
    let arc_mouse_delta = Arc::new(Mutex::new((0f32, 0f32)));
    // Make a reference of this tuple to send to the render thread
    let mouse_delta = Arc::clone(&arc_mouse_delta);

    // Spawn a separate thread for rendering, so event handling doesn't block rendering
    let render_thread = thread::spawn(move || {
        // Acquire the OpenGL Context and load the function pointers. This has to be done inside of the rendering thread, because
        // an active OpenGL context cannot safely traverse a thread boundary
        let context = unsafe {
            let c = windowed_context.make_current().unwrap();
            gl::load_with(|symbol| c.get_proc_address(symbol) as *const _);
            c
        };

        // Set up openGL
        unsafe {
            gl::Enable(gl::DEPTH_TEST);
            gl::DepthFunc(gl::LESS);
            //gl::Enable(gl::CULL_FACE);
            gl::Disable(gl::MULTISAMPLE);
            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
            gl::Enable(gl::DEBUG_OUTPUT_SYNCHRONOUS);
            gl::DebugMessageCallback(Some(util::debug_callback), ptr::null());

            // Print some diagnostics
            println!("{}: {}", util::get_gl_string(gl::VENDOR), util::get_gl_string(gl::RENDERER));
            println!("OpenGL\t: {}", util::get_gl_string(gl::VERSION));
            println!("GLSL\t: {}", util::get_gl_string(gl::SHADING_LANGUAGE_VERSION));
        }

        // == // Set up your VAO here
        let mut v: u32=0;
        unsafe {
            let vertices: Vec<f32> = vec![

            //2a

            -1.0,0.0,0.1,
            0.25,0.0,0.1,
            -0.5,0.5,0.1,

            -0.5,-0.5,0.2,
            0.5,-0.5,0.2,
            0.0,0.175,0.2,

            -0.25,0.0,0.5,
            1.0,0.0,0.5,
            0.5,0.5,0.5


            /*
           //figure Letter - index
           -0.5,0.5,0.0, //H -0
           0.5,0.5,0.0,//F - 1
           0.0,1.0,0.0, //K -2
           0.0,0.0,0.0,//E - 3
           -0.5,-0.5,0.0, //I - 4
           0.0,-1.0,0.0,//J - 5
           0.5,-0.5,0.0 //G - 6

            */









            ];
            
            let indices: Vec<u32> = vec![0,1,2,6,7,8,3,4,5];
            //let indices2: Vec<u32> = vec![0..len(vertices)];
            let colors: Vec<f32> = vec![
            
           
                  
                    //blue
                    0.0,0.0,1.0,0.3,
                    0.0,0.0,1.0,0.3,
                    0.0,0.0,1.0,0.3,
                    //red
                  1.0,0.0,0.0,0.3,
                  1.0,0.0,0.0,0.3,
                  1.0,0.0,0.0,0.3,
                            
                //green
                0.0,1.0,0.0,0.3,
                0.0,1.0,0.0,0.3,
                0.0,1.0,0.0,0.3,
           

            ];
            v = CreateVAO(&vertices, &indices, &colors);

        }

        // Basic usage of shader helper:
        // The example code below returns a shader object, which contains the field `.program_id`.
        // The snippet is not enough to do the assignment, and will need to be modified (outside of
        // just using the correct path), but it only needs to be called once
        //
        //     
        
        unsafe {

       
          
    
            shader::ShaderBuilder::new()
             .attach_file("./shaders/simple.frag").attach_file("./shaders/simple.vert")
                .link().activate();
            
        }

        // Used to demonstrate keyboard handling -- feel free to remove
        let mut _arbitrary_number_hor = 0.0;
        let mut _arbitrary_number_ver = 0.0;
        let mut _arbitrary_number_depth = 0.0;
        let mut _arbitrary_number_x = 0.0;
        let mut _arbitrary_number_y = 0.0;

        let first_frame_time = std::time::Instant::now();
        let mut last_frame_time = first_frame_time;
        // The main rendering loop
        loop {
            let now = std::time::Instant::now();
            let elapsed = now.duration_since(first_frame_time).as_secs_f32();
            let delta_time = now.duration_since(last_frame_time).as_secs_f32();
            last_frame_time = now;

            let identity: glm::Mat4 = glm::identity();

            //Horizontal traslation: A - D
            let horizontal_translation: glm::Mat4 = glm::translation(&glm::vec3(_arbitrary_number_hor,0.0,0.0));

            //Vertical Translation: W - S
            let vertical_translation: glm::Mat4 = glm::translation(&glm::vec3(0.0,_arbitrary_number_ver,0.0));

            //Translation in z-direction: Q - E
            let depth_translation: glm::Mat4 = glm::translation(&glm::vec3(0.0,0.0,_arbitrary_number_depth));

            //rotation around x - axis: Up - Down
            let rotation_x: glm::Mat4 = glm::rotation(_arbitrary_number_x, &glm::vec3(1.0, 0.0, 0.0));

            //Rotation around y - axis: Left - Right
            let rotation_y: glm::Mat4 = glm::rotation(_arbitrary_number_y, &glm::vec3(0.0, 1.0, 0.0));

            //Matrix to make vertices visable in clipping box: -1 to -100
            let trans: glm::Mat4 = glm::translation(&glm::vec3(0.0,0.0,-5.0));


            let perspective: glm::Mat4 = glm::perspective(1.0, 1.05, 100.0, 1.0);
            let res = 
            identity*perspective
            *rotation_x*rotation_y
            *trans*horizontal_translation*vertical_translation*depth_translation;

            

            // Handle keyboard input
            if let Ok(keys) = pressed_keys.lock() {
                for key in keys.iter() {
                    match key {
                        VirtualKeyCode::A => {
                            _arbitrary_number_hor += delta_time;
                        },
                        VirtualKeyCode::D => {
                            _arbitrary_number_hor -= delta_time;
                        },
                        VirtualKeyCode::W => {
                            _arbitrary_number_ver -= delta_time;
                        },
                        VirtualKeyCode::S => {
                            _arbitrary_number_ver += delta_time;
                        },
                        VirtualKeyCode::Q => {
                            _arbitrary_number_depth += delta_time;
                        },
                        VirtualKeyCode::E => {
                            _arbitrary_number_depth -= delta_time;
                        },
                        VirtualKeyCode::Down => {
                            _arbitrary_number_x += delta_time;
                        },
                        VirtualKeyCode::Up => {
                            _arbitrary_number_x -= delta_time;
                        },
                        VirtualKeyCode::Left => {
                            _arbitrary_number_y -= delta_time;
                        },
                        VirtualKeyCode::Right
                         => {
                            _arbitrary_number_y += delta_time;
                        },



                        _ => { }
                    }
                }
            }
            // Handle mouse movement. delta contains the x and y movement of the mouse since last frame in pixels
            if let Ok(mut delta) = mouse_delta.lock() {



                *delta = (0.0, 0.0);
            }

            unsafe {
                gl::ClearColor(0.76862745, 0.71372549, 0.94901961, 1.0); // moon raker, full opacity
                //gl::ClearColor(0.0, 0.0, 0.0, 1.0); //black to make it cooler
                gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

                // Issue the necessary commands to draw your scene here

                //Issuing the Drawing
               // gl::BindVertexArray(v);
                gl::DrawElements(gl::TRIANGLES,
                    9,
                    gl::UNSIGNED_INT,
                    ptr::null());

                    //Send transformation vector to vertex shader
                    gl::UniformMatrix4fv(3,1,gl::FALSE,res.as_ptr());

            }
            

            context.swap_buffers().unwrap();
        }
    });

    // Keep track of the health of the rendering thread
    let render_thread_healthy = Arc::new(RwLock::new(true));
    let render_thread_watchdog = Arc::clone(&render_thread_healthy);
    thread::spawn(move || {
        if !render_thread.join().is_ok() {
            if let Ok(mut health) = render_thread_watchdog.write() {
                println!("Render thread panicked!");
                *health = false;
            }
        }
    });

    // Start the event loop -- This is where window events get handled
    el.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        // Terminate program if render thread panics
        if let Ok(health) = render_thread_healthy.read() {
            if *health == false {
                *control_flow = ControlFlow::Exit;
            }
        }

        match event {
            Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => {
                *control_flow = ControlFlow::Exit;
            },
            // Keep track of currently pressed keys to send to the rendering thread
            Event::WindowEvent { event: WindowEvent::KeyboardInput {
                input: KeyboardInput { state: key_state, virtual_keycode: Some(keycode), .. }, .. }, .. } => {

                if let Ok(mut keys) = arc_pressed_keys.lock() {
                    match key_state {
                        Released => {
                            if keys.contains(&keycode) {
                                let i = keys.iter().position(|&k| k == keycode).unwrap();
                                keys.remove(i);
                            }
                        },
                        Pressed => {
                            if !keys.contains(&keycode) {
                                keys.push(keycode);
                            }
                        }
                    }
                }

                // Handle escape separately
                match keycode {
                    Escape => {
                        *control_flow = ControlFlow::Exit;
                    },

                    _ => { }
                }
            },
            Event::DeviceEvent { event: DeviceEvent::MouseMotion { delta }, .. } => {
                // Accumulate mouse movement
                if let Ok(mut position) = arc_mouse_delta.lock() {
                    *position = (position.0 + delta.0 as f32, position.1 + delta.1 as f32);
                }
            },
            _ => { }
        }
    });
}
