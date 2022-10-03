// Uncomment these following global attributes to silence most warnings of "low" interest:
/*
#![allow(dead_code)]
#![allow(non_snake_case)]
#![allow(unreachable_code)]
#![allow(unused_mut)]
#![allow(unused_unsafe)]
#![allow(unused_variables)]
*/
extern crate nalgebra_glm as glm;
use std::sync::{Arc, Mutex, RwLock};
use std::thread;
use std::{mem, os::raw::c_void, ptr};

mod shader;
mod util;

use glutin::event::{
    DeviceEvent,
    ElementState::{Pressed, Released},
    Event, KeyboardInput,
    VirtualKeyCode::{self, *},
    WindowEvent,
};
use glutin::event_loop::ControlFlow;

// initial window size
const INITIAL_SCREEN_W: u32 = 800;
const INITIAL_SCREEN_H: u32 = 600;

#[repr(C)]
struct Vertex {
    x: f32,
    y: f32,
    z: f32,
}

impl Vertex {
    fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }
}

trait Index {}

#[repr(C)]
struct TriangleIndex {
    v1: u32,
    v2: u32,
    v3: u32,
}

impl TriangleIndex {
    fn new(v1: u32, v2: u32, v3: u32) -> Self {
        Self { v1, v2, v3 }
    }
}

impl Index for TriangleIndex {}

#[repr(C)]
struct Rgba {
    red: f32,
    green: f32,
    blue: f32,
    alpha: f32,
}

impl Rgba {
    fn new(red: f32, green: f32, blue: f32, alpha: f32) -> Self {
        Self {
            red,
            green,
            blue,
            alpha,
        }
    }
}

// == // Helper functions to make interacting with OpenGL a little bit prettier. You *WILL* need these! // == //

// Get the size of an arbitrary array of numbers measured in bytes
// Example usage:  pointer_to_array(my_array)
fn size_of_slice<T>(val: &[T]) -> isize {
    std::mem::size_of_val(&val[..]) as isize
}

// Get the OpenGL-compatible pointer to an arbitrary array of numbers
// Example usage:  pointer_to_array(my_array)
fn slice_as_void_ptr<T>(val: &[T]) -> *const c_void {
    &val[0] as *const T as *const c_void
}

// Get an offset in bytes for n units of type T, represented as a relative pointer
// Example usage:  offset::<u64>(4)
fn offset<T>(n: u32) -> *const c_void {
    (n * mem::size_of::<T>() as u32) as *const T as *const c_void
}

// Get a null pointer (equivalent to an offset of 0)
// ptr::null()

/// Create a new buffer, bind it, and fill it with the given data
unsafe fn buffer_with_data<T>(target: gl::types::GLenum, data: &[T]) -> u32 {
    let mut buf_id: u32 = 0;
    gl::GenBuffers(1, &mut buf_id);
    gl::BindBuffer(target, buf_id);
    gl::BufferData(
        target,
        size_of_slice(data),
        slice_as_void_ptr(data),
        gl::STATIC_DRAW,
    );

    buf_id
}

// == // Generate your VAO here
unsafe fn create_vao<T: Index>(vertices: &[Vertex], indices: &[T], colors: &[Rgba]) -> u32 {
    // Implement me!
    let mut vao_id = 0;
    gl::GenVertexArrays(1, &mut vao_id);
    gl::BindVertexArray(vao_id);

    buffer_with_data(gl::ARRAY_BUFFER, vertices);
    gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, 0, ptr::null());
    gl::EnableVertexAttribArray(0);

    buffer_with_data(gl::ARRAY_BUFFER, colors);
    gl::VertexAttribPointer(1, 4, gl::FLOAT, gl::FALSE, 0, ptr::null());
    gl::EnableVertexAttribArray(1);

    buffer_with_data(gl::ELEMENT_ARRAY_BUFFER, indices);

    vao_id
}

fn main() {
    // Set up the necessary objects to deal with windows and event handling
    let el = glutin::event_loop::EventLoop::new();
    let wb = glutin::window::WindowBuilder::new()
        .with_title("ama's fancy OpenGL scene")
        .with_resizable(true)
        .with_inner_size(glutin::dpi::LogicalSize::new(
            INITIAL_SCREEN_W,
            INITIAL_SCREEN_H,
        ));
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

    // Set up shared tuple for tracking changes to the window size
    let arc_window_size = Arc::new(Mutex::new((INITIAL_SCREEN_W, INITIAL_SCREEN_H, false)));
    // Make a reference of this tuple to send to the render thread
    let window_size = Arc::clone(&arc_window_size);

    // Spawn a separate thread for rendering, so event handling doesn't block rendering
    let render_thread = thread::spawn(move || {
        // Acquire the OpenGL Context and load the function pointers.
        // This has to be done inside of the rendering thread, because
        // an active OpenGL context cannot safely traverse a thread boundary
        let context = unsafe {
            let c = windowed_context.make_current().unwrap();
            gl::load_with(|symbol| c.get_proc_address(symbol) as *const _);
            c
        };

        let mut window_aspect_ratio = INITIAL_SCREEN_W as f32 / INITIAL_SCREEN_H as f32;

        // Set up openGL
        unsafe {
            gl::Enable(gl::DEPTH_TEST);
            gl::DepthFunc(gl::LESS);
            gl::Enable(gl::CULL_FACE);
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

        // Vertices for the three triangles.
        // The z-coordinates take the perspective projection into account (they were the opposite way around
        // before introducing the perspective projection).
        let verts = vec![
            Vertex::new(0.0, 0.9, -0.5),
            Vertex::new(-0.7, -0.5, -0.5),
            Vertex::new(0.7, -0.5, -0.5),
            Vertex::new(0.0, 0.4, 0.0),
            Vertex::new(-0.5, -0.7, 0.0),
            Vertex::new(0.5, -0.7, 0.0),
            Vertex::new(0.5, 0.4, 0.5),
            Vertex::new(-0.5, 0.4, 0.5),
            Vertex::new(0.0, -0.7, 0.5),
        ];
        let indices = vec![
            TriangleIndex::new(0, 1, 2),
            TriangleIndex::new(3, 4, 5),
            TriangleIndex::new(6, 7, 8),
        ];
        let colors = vec![
            Rgba::new(1.0, 0.0, 0.0, 0.5),
            Rgba::new(1.0, 0.0, 0.0, 0.5),
            Rgba::new(1.0, 0.0, 0.0, 0.5),
            Rgba::new(0.0, 0.0, 1.0, 0.5),
            Rgba::new(0.0, 0.0, 1.0, 0.5),
            Rgba::new(0.0, 0.0, 1.0, 0.5),
            Rgba::new(0.0, 1.0, 0.0, 0.5),
            Rgba::new(0.0, 1.0, 0.0, 0.5),
            Rgba::new(0.0, 1.0, 0.0, 0.5),
        ];
        let my_vao = unsafe { create_vao(&verts, &indices, &colors) };

        // Setup the simple shader
        let _simple_shader = unsafe {
            let s = shader::ShaderBuilder::new()
                .attach_file("./shaders/simple.vert")
                .attach_file("./shaders/simple.frag")
                .link();
            s.activate();
            s
        };

        let perspective: glm::Mat4 = glm::perspective(window_aspect_ratio, 1.2915, 1.0, 100.0);

        let mut translate_x = 0.0;
        let mut translate_y = 0.0;
        let mut translate_z = -3.0;
        let mut rotate_x = 0.0;
        let mut rotate_y: f32 = 0.0;

        // The main rendering loop
        let first_frame_time = std::time::Instant::now();
        let mut prevous_frame_time = first_frame_time;
        loop {
            // Compute time passed since the previous frame and since the start of the program
            let now = std::time::Instant::now();
            let elapsed = now.duration_since(first_frame_time).as_secs_f32();
            let delta_time = now.duration_since(prevous_frame_time).as_secs_f32();
            prevous_frame_time = now;

            // Handle resize events
            if let Ok(mut new_size) = window_size.lock() {
                if new_size.2 {
                    context.resize(glutin::dpi::PhysicalSize::new(new_size.0, new_size.1));
                    window_aspect_ratio = new_size.0 as f32 / new_size.1 as f32;
                    (*new_size).2 = false;
                    println!("Resized");
                    unsafe {
                        gl::Viewport(0, 0, new_size.0 as i32, new_size.1 as i32);
                    }
                }
            }

            // Handle keyboard input
            if let Ok(keys) = pressed_keys.lock() {
                for key in keys.iter() {
                    match key {
                        // The `VirtualKeyCode` enum is defined here:
                        //    https://docs.rs/winit/0.25.0/winit/event/enum.VirtualKeyCode.html
                        VirtualKeyCode::A => {
                            translate_x += delta_time * rotate_y.cos();
                            translate_z += delta_time * rotate_y.sin();
                        }
                        VirtualKeyCode::D => {
                            translate_x -= delta_time * rotate_y.cos();
                            translate_z -= delta_time * rotate_y.sin();
                        }
                        VirtualKeyCode::W => {
                            translate_x -= delta_time * rotate_y.sin();
                            translate_z += delta_time * rotate_y.cos();
                        }
                        VirtualKeyCode::S => {
                            translate_x += delta_time * rotate_y.sin();
                            translate_z -= delta_time * rotate_y.cos();
                        }
                        VirtualKeyCode::Space => {
                            translate_y -= delta_time;
                        }
                        VirtualKeyCode::LShift => {
                            translate_y += delta_time;
                        }
                        VirtualKeyCode::Left => {
                            rotate_y -= delta_time;
                        }
                        VirtualKeyCode::Right => {
                            rotate_y += delta_time;
                        }
                        VirtualKeyCode::Up => {
                            rotate_x -= delta_time;
                            rotate_x = rotate_x.clamp(-std::f32::consts::PI, std::f32::consts::PI);
                        }
                        VirtualKeyCode::Down => {
                            rotate_x += delta_time;
                            rotate_x = rotate_x.clamp(-std::f32::consts::PI, std::f32::consts::PI);
                        }
                        // default handler:
                        _ => {}
                    }
                }
            }
            // Handle mouse movement. delta contains the x and y movement of the mouse since last frame in pixels
            if let Ok(mut delta) = mouse_delta.lock() {
                // == // Optionally access the acumulated mouse movement between
                // == // frames here with `delta.0` and `delta.1`

                *delta = (0.0, 0.0); // reset when done
            }

            // == // Please compute camera transforms here (exercise 2 & 3)
            let translation = glm::translation(&glm::vec3(translate_x, translate_y, translate_z));
            let rotation_y = glm::rotation(rotate_y, &glm::vec3(0.0, 1.0, 0.0));
            let rotation_x = glm::rotation(rotate_x, &glm::vec3(1.0, 0.0, 0.0));
            let uniform: glm::Mat4 =
                perspective * rotation_x * rotation_y * translation * glm::identity();
            unsafe {
                gl::UniformMatrix4fv(0, 1, 0, uniform.as_ptr());
            }

            unsafe {
                // Clear the color and depth buffers
                gl::ClearColor(0.035, 0.046, 0.078, 1.0); // night sky, full opacity
                gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

                // Issue draw calls
                gl::BindVertexArray(my_vao);
                gl::DrawElements(gl::TRIANGLES, 9, gl::UNSIGNED_INT, ptr::null());
            }

            // Display the new color buffer on the display
            context.swap_buffers().unwrap(); // we use "double buffering" to avoid artifacts
        }
    });

    // == //
    // == // From here on down there are only internals.
    // == //

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

    // Start the event loop -- This is where window events are initially handled
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
                event: WindowEvent::Resized(physical_size),
                ..
            } => {
                println!(
                    "New window size! width: {}, height: {}",
                    physical_size.width, physical_size.height
                );
                if let Ok(mut new_size) = arc_window_size.lock() {
                    *new_size = (physical_size.width, physical_size.height, true);
                }
            }
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

                // Handle Escape and Q keys separately
                match keycode {
                    Escape => {
                        *control_flow = ControlFlow::Exit;
                    }
                    Q => {
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
