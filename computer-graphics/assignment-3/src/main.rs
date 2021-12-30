extern crate nalgebra_glm as glm;
use std::sync::{Arc, Mutex, RwLock};
use std::thread;
use std::{mem, os::raw::c_void, ptr};

mod shader;
mod toolbox;
use toolbox::Heading;
mod mesh;
mod scene_graph;
mod util;
use scene_graph::SceneNode;

use glutin::event::{
    DeviceEvent,
    ElementState::{Pressed, Released},
    Event, KeyboardInput,
    VirtualKeyCode::{self, *},
    WindowEvent,
};
use glutin::event_loop::ControlFlow;

const SCREEN_W: u32 = 1000;
const SCREEN_H: u32 = 800;

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
unsafe fn CreateVAO(
    vertex_coordinates: &Vec<f32>,
    indices_array: &Vec<u32>,
    color_array: &Vec<f32>,
    normals_array: &Vec<f32>,
) -> u32 {
    //Setup VAO
    //Genereate vertex arrays
    let mut a: u32 = 0;
    let mut b: u32 = 0;
    gl::GenVertexArrays(1, &mut a);
    gl::BindVertexArray(a);

    //Creating VBO
    gl::GenBuffers(1, &mut b);
    gl::BindBuffer(gl::ARRAY_BUFFER, b);

    //Filling the VBO with data
    gl::BufferData(
        gl::ARRAY_BUFFER,
        byte_size_of_array(vertex_coordinates),
        pointer_to_array(vertex_coordinates),
        gl::STATIC_DRAW,
    );

    //Set Vertex Attribute Pointer
    gl::VertexAttribPointer(
        0,
        3,
        gl::FLOAT,
        gl::FALSE,
        size_of::<f32>() * 3,
        ptr::null(),
    );

    //Enable the vertex attributes
    gl::EnableVertexAttribArray(0);

    //Color Buffer
    let mut f: u32 = 0;
    gl::GenBuffers(1, &mut f);
    gl::BindBuffer(gl::ARRAY_BUFFER, f);

    //fill Color Buffer
    gl::BufferData(
        gl::ARRAY_BUFFER,
        byte_size_of_array(color_array),
        pointer_to_array(color_array),
        gl::STATIC_DRAW,
    );
    gl::VertexAttribPointer(
        2,
        4,
        gl::FLOAT,
        gl::FALSE,
        size_of::<f32>() * 4,
        ptr::null(),
    );

    //enable vertex attributes
    gl::EnableVertexAttribArray(2);

    //Normals buffer
    let mut n: u32 = 0;
    gl::GenBuffers(1, &mut n);
    gl::BindBuffer(gl::ARRAY_BUFFER, n);

    //fill normals buffer
    gl::BufferData(
        gl::ARRAY_BUFFER,
        byte_size_of_array(normals_array),
        pointer_to_array(normals_array),
        gl::STATIC_DRAW,
    );

    gl::VertexAttribPointer(
        6,
        3,
        gl::FLOAT,
        gl::FALSE,
        size_of::<f32>() * 3,
        ptr::null(),
    );

    gl::EnableVertexAttribArray(6); //memory pointer: 6

    //index buffer
    let mut d: u32 = 0;
    gl::GenBuffers(1, &mut d);
    gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, d);
    gl::BufferData(
        gl::ELEMENT_ARRAY_BUFFER,
        byte_size_of_array(indices_array),
        pointer_to_array(indices_array),
        gl::STATIC_DRAW,
    );

    return a;
}

fn main() {
    // Set up the necessary objects to deal with windows and event handling
    let el = glutin::event_loop::EventLoop::new();
    let wb = glutin::window::WindowBuilder::new()
        .with_title("Gloom-rs")
        .with_resizable(false)
        .with_inner_size(glutin::dpi::LogicalSize::new(SCREEN_W, SCREEN_H));
    let cb = glutin::ContextBuilder::new().with_vsync(true);
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

        let lunars = mesh::Terrain::load("./resources/lunarsurface.obj");
        let helicopter = mesh::Helicopter::load("./resources/helicopter.obj");

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
            println!(
                "{}: {}",
                util::get_gl_string(gl::VENDOR),
                util::get_gl_string(gl::RENDERER)
            );
            println!("OpenGL\t: {}", util::get_gl_string(gl::VERSION));
            println!(
                "GLSL\t: {}",
                util::get_gl_string(gl::SHADING_LANGUAGE_VERSION)
            );
        }

        // == // Set up your VAO here
        let mut v: u32 = 0;
        let mut body: u32 = 0;
        let mut tail_rotor: u32 = 0;
        let mut main_rotor: u32 = 0;
        let mut door: u32 = 0;
        unsafe {
            //Terrain
            v = CreateVAO(
                &lunars.vertices,
                &lunars.indices,
                &lunars.colors,
                &lunars.normals,
            );
            //Helicopter
            body = CreateVAO(
                &helicopter.body.vertices,
                &helicopter.body.indices,
                &helicopter.body.colors,
                &helicopter.body.normals,
            );
            tail_rotor = CreateVAO(
                &helicopter.tail_rotor.vertices,
                &helicopter.tail_rotor.indices,
                &helicopter.tail_rotor.colors,
                &helicopter.tail_rotor.normals,
            );
            main_rotor = CreateVAO(
                &helicopter.main_rotor.vertices,
                &helicopter.main_rotor.indices,
                &helicopter.main_rotor.colors,
                &helicopter.main_rotor.normals,
            );
            door = CreateVAO(
                &helicopter.door.vertices,
                &helicopter.door.indices,
                &helicopter.tail_rotor.colors,
                &helicopter.door.normals,
            );
        }
        //Scene Graph setup
        //Parent nodes
        let mut root = SceneNode::new();

        //Nodes from VAO`s
        let mut terrain_node = SceneNode::from_vao(v, lunars.index_count);
        let mut h_body_node = SceneNode::from_vao(body, helicopter.body.index_count);
        let mut h_trotor_node = SceneNode::from_vao(tail_rotor, helicopter.tail_rotor.index_count);
        let mut h_mrotor_node = SceneNode::from_vao(main_rotor, helicopter.main_rotor.index_count);
        let mut h_door_node = SceneNode::from_vao(door, helicopter.door.index_count);

        //Initializing values for nodes
        terrain_node.position = glm::vec3(0.0, 0.0, 0.0);
        terrain_node.reference_point = glm::vec3(0.0, 0.0, 0.0);

        h_mrotor_node.rotation = glm::vec3(0.0, 0.0, 0.0);
        h_mrotor_node.reference_point = glm::vec3(0.0, 0.0, 0.0);
        h_mrotor_node.position = glm::vec3(0.0, 0.0, 0.0);

        h_trotor_node.rotation = glm::vec3(0.0, 0.0, 0.0);
        h_trotor_node.reference_point = glm::vec3(0.35, 2.3, 10.4);
        h_trotor_node.position = glm::vec3(0.0, 0.0, 0.0);

        h_body_node.position = glm::vec3(0.0, 0.0, 0.0);
        h_body_node.reference_point = glm::vec3(0.0, 0.0, 0.0);

        h_door_node.position = glm::vec3(0.0, 0.0, 0.0);
        h_door_node.reference_point = glm::vec3(0.0, 0.0, 0.0);

        //Setting the tree
        root.add_child(&terrain_node);
        terrain_node.add_child(&h_body_node);
        h_body_node.add_child(&h_mrotor_node);
        h_body_node.add_child(&h_trotor_node);
        h_body_node.add_child(&h_door_node);

        // Basic usage of shader helper:
        // The example code below returns a shader object, which contains the field `.program_id`.
        // The snippet is not enough to do the assignment, and will need to be modified (outside of
        // just using the correct path), but it only needs to be called once
        //
        //

        unsafe {
            shader::ShaderBuilder::new()
                .attach_file("./shaders/simple.frag")
                .attach_file("./shaders/simple.vert")
                .link()
                .activate();
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

            //Matrix to make vertices visable in clipping box: -1 to -100
            let model: glm::Mat4 = glm::translation(&glm::vec3(0.0, 0.0, -5.0));

            let mut movement = glm::vec3(
                _arbitrary_number_hor,
                _arbitrary_number_ver,
                _arbitrary_number_depth,
            );
            let mut rotation = glm::vec3(_arbitrary_number_y, _arbitrary_number_x, 0.0);

            let mov_xyz: glm::Mat4 = glm::translation(&movement);
            let rot_xy: glm::Mat4 = glm::rotation(-rotation[1], &glm::vec3(1.0, 0.0, 0.0))
                * glm::rotation(rotation[0], &glm::vec3(0.0, 1.0, 0.0));
            let view: glm::Mat4 = rot_xy * mov_xyz;

            let perspective: glm::Mat4 =
                glm::perspective(SCREEN_W as f32 / SCREEN_H as f32, 1.05, 1.0, 1000.0);

            let res = identity * perspective * view * model;

            let movement_const = 20.0;

            // Handle keyboard input
            if let Ok(keys) = pressed_keys.lock() {
                for key in keys.iter() {
                    match key {
                        VirtualKeyCode::A => {
                            _arbitrary_number_hor += delta_time * movement_const;
                        }
                        VirtualKeyCode::D => {
                            _arbitrary_number_hor -= delta_time * movement_const;
                        }
                        VirtualKeyCode::W => {
                            _arbitrary_number_ver -= delta_time * movement_const;
                        }
                        VirtualKeyCode::S => {
                            _arbitrary_number_ver += delta_time * movement_const;
                        }
                        VirtualKeyCode::Q => {
                            _arbitrary_number_depth += delta_time * movement_const;
                        }
                        VirtualKeyCode::E => {
                            _arbitrary_number_depth -= delta_time * movement_const;
                        }
                        VirtualKeyCode::Down => {
                            _arbitrary_number_x += delta_time;
                        }
                        VirtualKeyCode::Up => {
                            _arbitrary_number_x -= delta_time;
                        }
                        VirtualKeyCode::Left => {
                            _arbitrary_number_y -= delta_time;
                        }
                        VirtualKeyCode::Right => {
                            _arbitrary_number_y += delta_time;
                        }

                        _ => {}
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

                unsafe fn draw_scene(
                    node: &scene_graph::SceneNode,
                    view_projection_matrix: &glm::Mat4,
                ) {
                    // Check if node is drawable, set uniforms, draw
                    if node.index_count > 0 {
                        gl::BindVertexArray(node.vao_id);
                        gl::UniformMatrix4fv(
                            3,
                            1,
                            gl::FALSE,
                            (view_projection_matrix * node.current_transformation_matrix).as_ptr(),
                        );
                        gl::UniformMatrix4fv(
                            7,
                            1,
                            gl::FALSE,
                            node.current_transformation_matrix.as_ptr(),
                        );
                        gl::DrawElements(
                            gl::TRIANGLES,
                            node.index_count,
                            gl::UNSIGNED_INT,
                            ptr::null(),
                        );
                    }
                    // Recurse
                    for &child in &node.children {
                        draw_scene(&*child, view_projection_matrix);
                    }
                }

                unsafe fn update_node_transformations(
                    node: &mut scene_graph::SceneNode,
                    transformation_so_far: &glm::Mat4,
                    elapsed: f32,
                ) {
                    // Construct the correct transformation matrix
                    let position: glm::Mat4 = glm::translation(&node.position);
                    let reference: glm::Mat4 = glm::translation(&node.reference_point);
                    let reference_: glm::Mat4 = glm::translation(&-node.reference_point);

                    //node.current_transformation_matrix = transformation_so_far* glm::translation(&node.reference_point);
                    let mut rotation = glm::rotation(node.rotation.x, &glm::vec3(1.0, 0.0, 0.0))
                        * glm::rotation(node.rotation.y, &glm::vec3(0.0, 1.0, 0.0))
                        * glm::rotation(node.rotation.z, &glm::vec3(0.0, 0.0, 1.0));

                    rotation = reference * rotation * reference_;

                    let model: glm::Mat4 = position * rotation;
                    // Update the node's transformation matrix
                    node.current_transformation_matrix = transformation_so_far * model;

                    // Recurse
                    for &child in &node.children {
                        update_node_transformations(
                            &mut *child,
                            &node.current_transformation_matrix,
                            elapsed,
                        );
                    }
                }

                //Matrix for task 5 so view stays the same
                let task5viewmat: glm::Mat4 = glm::mat4(
                    0.121, 0.0, 1.375, -2.51, 0.0574, 1.725, -0.005, 1.01, 0.99, -0.033, -0.087,
                    15.58, 0.99, -0.033, -0.087, 17.55,
                );
                for i in 0..5 {
                    let heading = toolbox::simple_heading_animation(elapsed + (i as f32) * 1.5);
                    h_trotor_node.rotation.x = elapsed * 1000.0;
                    h_mrotor_node.rotation.y = elapsed * 1000.0;
                    h_body_node.rotation.x = heading.pitch;
                    h_body_node.rotation.y = heading.yaw;
                    h_body_node.rotation.z = heading.roll;
                    h_body_node.position.x = heading.x;
                    h_body_node.position.z = heading.z;
                    update_node_transformations(&mut root, &identity, elapsed);
                    draw_scene(&mut root, &res);
                }

                //Send transformation vector to vertex shader
                // gl::UniformMatrix4fv(3,1,gl::FALSE,res_try.as_ptr()); //memory pointer: 3
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
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                *control_flow = ControlFlow::Exit;
            }
            // Keep track of currently pressed keys to send to the rendering thread
            Event::WindowEvent {
                event:
                    WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                state: key_state,
                                virtual_keycode: Some(keycode),
                                ..
                            },
                        ..
                    },
                ..
            } => {
                if let Ok(mut keys) = arc_pressed_keys.lock() {
                    match key_state {
                        Released => {
                            if keys.contains(&keycode) {
                                let i = keys.iter().position(|&k| k == keycode).unwrap();
                                keys.remove(i);
                            }
                        }
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
                    }

                    _ => {}
                }
            }
            Event::DeviceEvent {
                event: DeviceEvent::MouseMotion { delta },
                ..
            } => {
                // Accumulate mouse movement
                if let Ok(mut position) = arc_mouse_delta.lock() {
                    *position = (position.0 + delta.0 as f32, position.1 + delta.1 as f32);
                }
            }
            _ => {}
        }
    });
}
