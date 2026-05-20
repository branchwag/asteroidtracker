use leptos::prelude::*;

#[component]
pub fn StarryBackground() -> impl IntoView {
    let canvas_ref = NodeRef::<leptos::html::Canvas>::new();

    #[cfg(feature = "hydrate")]
    canvas_ref.on_load(|canvas| {
        client::init(canvas.into());
    });

    view! {
        <canvas node_ref=canvas_ref class="starry-canvas" />
    }
}

#[cfg(feature = "hydrate")]
mod client {
    use std::cell::RefCell;
    use std::f32::consts::PI;
    use std::rc::Rc;

    use js_sys::Math;
    use wasm_bindgen::prelude::*;
    use wasm_bindgen::JsCast;
    use web_sys::{
        HtmlCanvasElement, WebGl2RenderingContext as GL, WebGlBuffer, WebGlProgram, WebGlShader,
        WebGlUniformLocation, WebGlVertexArrayObject,
    };

    const FOV_RAD: f32 = 75.0 * (PI / 180.0);
    const NEAR: f32 = 0.1;
    const FAR: f32 = 1000.0;
    const CAMERA_Z: f32 = 5.0;

    const STAR_VS: &str = r#"#version 300 es
        in vec3 aPosition;
        in float aOpacity;
        in float aTwinkleSpeed;
        in float aTwinkleOffset;

        uniform mat4 uMVP;
        uniform float uTime;
        uniform float uPointSize;

        out float vOpacity;

        void main() {
            float twinkle = sin(uTime * aTwinkleSpeed + aTwinkleOffset);
            vOpacity = aOpacity + twinkle * 0.5;
            gl_Position = uMVP * vec4(aPosition, 1.0);
            gl_PointSize = uPointSize;
        }
    "#;

    const STAR_FS: &str = r#"#version 300 es
        precision mediump float;
        in float vOpacity;
        out vec4 outColor;

        void main() {
            vec2 coord = gl_PointCoord - vec2(0.5);
            float dist = length(coord);
            if (dist > 0.5) discard;
            float edge = smoothstep(0.5, 0.4, dist);
            float alpha = clamp(vOpacity, 0.0, 1.5) * edge;
            outColor = vec4(1.0, 1.0, 1.0, alpha);
        }
    "#;

    const LINE_VS: &str = r#"#version 300 es
        in vec3 aPosition;
        uniform mat4 uMVP;
        void main() {
            gl_Position = uMVP * vec4(aPosition, 1.0);
        }
    "#;

    const LINE_FS: &str = r#"#version 300 es
        precision mediump float;
        uniform float uAlpha;
        out vec4 outColor;
        void main() {
            outColor = vec4(1.0, 1.0, 1.0, uAlpha);
        }
    "#;

    struct StarLayer {
        vao: WebGlVertexArrayObject,
        count: i32,
        point_size: f32,
    }

    struct ShootingStar {
        vao: WebGlVertexArrayObject,
        _buffer: WebGlBuffer,
        life_time: f32,
        max_life: f32,
    }

    struct Renderer {
        gl: GL,
        canvas: HtmlCanvasElement,
        star_program: WebGlProgram,
        star_u_mvp: WebGlUniformLocation,
        star_u_time: WebGlUniformLocation,
        star_u_point_size: WebGlUniformLocation,
        line_program: WebGlProgram,
        line_u_mvp: WebGlUniformLocation,
        line_u_alpha: WebGlUniformLocation,
        star_layers: Vec<StarLayer>,
        shooting_stars: Vec<ShootingStar>,
        rotation_x: f32,
        rotation_y: f32,
        last_time_ms: f64,
        shooting_timer: f32,
        width: u32,
        height: u32,
    }

    pub fn init(canvas: HtmlCanvasElement) {
        let window = web_sys::window().expect("no window");
        let dpr = window.device_pixel_ratio().max(1.0);
        let css_w = window.inner_width().ok().and_then(|v| v.as_f64()).unwrap_or(800.0);
        let css_h = window.inner_height().ok().and_then(|v| v.as_f64()).unwrap_or(600.0);
        let pixel_w = (css_w * dpr) as u32;
        let pixel_h = (css_h * dpr) as u32;
        canvas.set_width(pixel_w);
        canvas.set_height(pixel_h);

        let gl: GL = match canvas
            .get_context("webgl2")
            .ok()
            .flatten()
            .and_then(|c| c.dyn_into::<GL>().ok())
        {
            Some(gl) => gl,
            None => return,
        };

        let star_program = compile_program(&gl, STAR_VS, STAR_FS);
        let star_u_mvp = gl.get_uniform_location(&star_program, "uMVP").unwrap();
        let star_u_time = gl.get_uniform_location(&star_program, "uTime").unwrap();
        let star_u_point_size = gl
            .get_uniform_location(&star_program, "uPointSize")
            .unwrap();

        let line_program = compile_program(&gl, LINE_VS, LINE_FS);
        let line_u_mvp = gl.get_uniform_location(&line_program, "uMVP").unwrap();
        let line_u_alpha = gl.get_uniform_location(&line_program, "uAlpha").unwrap();

        let small = build_star_layer(&gl, &star_program, 1500, 2000.0, 0.001, 0.005, 2.5);
        let medium = build_star_layer(&gl, &star_program, 750, 1500.0, 0.01, 0.05, 3.0);
        let large = build_star_layer(&gl, &star_program, 3000, 1000.0, 0.005, 0.01, 2.0);

        gl.enable(GL::BLEND);
        gl.blend_func(GL::SRC_ALPHA, GL::ONE_MINUS_SRC_ALPHA);
        gl.disable(GL::DEPTH_TEST);
        gl.clear_color(0.0, 0.0, 0.0, 1.0);

        let renderer = Rc::new(RefCell::new(Renderer {
            gl,
            canvas: canvas.clone(),
            star_program,
            star_u_mvp,
            star_u_time,
            star_u_point_size,
            line_program,
            line_u_mvp,
            line_u_alpha,
            star_layers: vec![small, medium, large],
            shooting_stars: Vec::new(),
            rotation_x: 0.0,
            rotation_y: 0.0,
            last_time_ms: 0.0,
            shooting_timer: 1.0,
            width: pixel_w,
            height: pixel_h,
        }));

        install_resize_handler(renderer.clone());
        start_loop(renderer);
    }

    fn install_resize_handler(renderer: Rc<RefCell<Renderer>>) {
        let window = web_sys::window().unwrap();
        let cb = Closure::wrap(Box::new(move || {
            let win = web_sys::window().unwrap();
            let dpr = win.device_pixel_ratio().max(1.0);
            let css_w = win.inner_width().ok().and_then(|v| v.as_f64()).unwrap_or(800.0);
            let css_h = win.inner_height().ok().and_then(|v| v.as_f64()).unwrap_or(600.0);
            let pw = (css_w * dpr) as u32;
            let ph = (css_h * dpr) as u32;
            let mut r = renderer.borrow_mut();
            r.canvas.set_width(pw);
            r.canvas.set_height(ph);
            r.width = pw;
            r.height = ph;
        }) as Box<dyn FnMut()>);
        window
            .add_event_listener_with_callback("resize", cb.as_ref().unchecked_ref())
            .unwrap();
        cb.forget();
    }

    fn start_loop(renderer: Rc<RefCell<Renderer>>) {
        let f: Rc<RefCell<Option<Closure<dyn FnMut(f64)>>>> = Rc::new(RefCell::new(None));
        let g = f.clone();
        let r = renderer;

        *g.borrow_mut() = Some(Closure::wrap(Box::new(move |time_ms: f64| {
            r.borrow_mut().frame(time_ms);
            let _ = web_sys::window()
                .unwrap()
                .request_animation_frame(f.borrow().as_ref().unwrap().as_ref().unchecked_ref());
        }) as Box<dyn FnMut(f64)>));

        let _ = web_sys::window()
            .unwrap()
            .request_animation_frame(g.borrow().as_ref().unwrap().as_ref().unchecked_ref());
    }

    impl Renderer {
        fn frame(&mut self, time_ms: f64) {
            let delta = if self.last_time_ms == 0.0 {
                0.0
            } else {
                ((time_ms - self.last_time_ms) / 1000.0) as f32
            };
            self.last_time_ms = time_ms;

            self.rotation_x += 0.0001 * (delta * 60.0).max(0.0);
            self.rotation_y += 0.0002 * (delta * 60.0).max(0.0);

            let aspect = self.width as f32 / self.height.max(1) as f32;
            let proj = perspective(FOV_RAD, aspect, NEAR, FAR);
            let view = translation(0.0, 0.0, -CAMERA_Z);
            let model = mat_mul(&rotation_y(self.rotation_y), &rotation_x(self.rotation_x));
            let mvp = mat_mul(&mat_mul(&proj, &view), &model);

            let gl = &self.gl;
            gl.viewport(0, 0, self.width as i32, self.height as i32);
            gl.clear(GL::COLOR_BUFFER_BIT);

            gl.use_program(Some(&self.star_program));
            gl.uniform_matrix4fv_with_f32_array(Some(&self.star_u_mvp), false, &mvp);
            gl.uniform1f(Some(&self.star_u_time), (time_ms * 0.001) as f32);

            for layer in &self.star_layers {
                gl.bind_vertex_array(Some(&layer.vao));
                gl.uniform1f(Some(&self.star_u_point_size), layer.point_size);
                gl.draw_arrays(GL::POINTS, 0, layer.count);
            }
            gl.bind_vertex_array(None);

            self.shooting_timer -= delta;
            if self.shooting_timer <= 0.0 {
                if let Some(s) = build_shooting_star(gl, &self.line_program) {
                    self.shooting_stars.push(s);
                }
                self.shooting_timer = (Math::random() as f32) * 5.0 + 1.0;
            }

            gl.use_program(Some(&self.line_program));
            gl.uniform_matrix4fv_with_f32_array(Some(&self.line_u_mvp), false, &mvp);

            let mut i = 0;
            while i < self.shooting_stars.len() {
                let s = &mut self.shooting_stars[i];
                s.life_time += delta;
                let alpha = (1.0 - s.life_time / s.max_life).max(0.0);
                if s.life_time >= s.max_life {
                    gl.delete_vertex_array(Some(&s.vao));
                    self.shooting_stars.swap_remove(i);
                    continue;
                }
                gl.uniform1f(Some(&self.line_u_alpha), alpha);
                gl.bind_vertex_array(Some(&s.vao));
                gl.draw_arrays(GL::LINES, 0, 2);
                i += 1;
            }
            gl.bind_vertex_array(None);
        }
    }

    fn build_star_layer(
        gl: &GL,
        program: &WebGlProgram,
        count: usize,
        spread: f32,
        speed_min: f32,
        speed_max: f32,
        point_size: f32,
    ) -> StarLayer {
        let mut positions = Vec::with_capacity(count * 3);
        let mut opacities = Vec::with_capacity(count);
        let mut twinkle_speeds = Vec::with_capacity(count);
        let mut twinkle_offsets = Vec::with_capacity(count);

        for _ in 0..count {
            positions.push(((Math::random() as f32) - 0.5) * spread);
            positions.push(((Math::random() as f32) - 0.5) * spread);
            positions.push(((Math::random() as f32) - 0.5) * spread);
            opacities.push((Math::random() as f32) * 0.5 + 0.5);
            twinkle_speeds.push((Math::random() as f32) * (speed_max - speed_min) + speed_min);
            twinkle_offsets.push((Math::random() as f32) * PI * 2.0);
        }

        let vao = gl.create_vertex_array().expect("vao");
        gl.bind_vertex_array(Some(&vao));

        bind_attrib(gl, program, "aPosition", &positions, 3);
        bind_attrib(gl, program, "aOpacity", &opacities, 1);
        bind_attrib(gl, program, "aTwinkleSpeed", &twinkle_speeds, 1);
        bind_attrib(gl, program, "aTwinkleOffset", &twinkle_offsets, 1);

        gl.bind_vertex_array(None);

        StarLayer {
            vao,
            count: count as i32,
            point_size,
        }
    }

    fn build_shooting_star(gl: &GL, program: &WebGlProgram) -> Option<ShootingStar> {
        let x1 = ((Math::random() as f32) - 0.5) * 800.0;
        let y1 = (Math::random() as f32) * 400.0;
        let x2 = x1 - (Math::random() as f32) * 200.0 - 100.0;
        let y2 = y1 - (Math::random() as f32) * 200.0 - 100.0;
        let positions: [f32; 6] = [x1, y1, -500.0, x2, y2, -500.0];

        let vao = gl.create_vertex_array()?;
        gl.bind_vertex_array(Some(&vao));

        let buffer = gl.create_buffer()?;
        gl.bind_buffer(GL::ARRAY_BUFFER, Some(&buffer));
        let view = unsafe { js_sys::Float32Array::view(&positions) };
        gl.buffer_data_with_array_buffer_view(GL::ARRAY_BUFFER, &view, GL::STATIC_DRAW);
        let loc = gl.get_attrib_location(program, "aPosition");
        if loc < 0 {
            return None;
        }
        let loc = loc as u32;
        gl.enable_vertex_attrib_array(loc);
        gl.vertex_attrib_pointer_with_i32(loc, 3, GL::FLOAT, false, 0, 0);

        gl.bind_vertex_array(None);

        Some(ShootingStar {
            vao,
            _buffer: buffer,
            life_time: 0.0,
            max_life: (Math::random() as f32) * 1.5 + 0.5,
        })
    }

    fn bind_attrib(gl: &GL, program: &WebGlProgram, name: &str, data: &[f32], components: i32) {
        let buffer = gl.create_buffer().expect("buffer");
        gl.bind_buffer(GL::ARRAY_BUFFER, Some(&buffer));
        let view = unsafe { js_sys::Float32Array::view(data) };
        gl.buffer_data_with_array_buffer_view(GL::ARRAY_BUFFER, &view, GL::STATIC_DRAW);
        let loc = gl.get_attrib_location(program, name);
        if loc < 0 {
            return;
        }
        let loc = loc as u32;
        gl.enable_vertex_attrib_array(loc);
        gl.vertex_attrib_pointer_with_i32(loc, components, GL::FLOAT, false, 0, 0);
    }

    fn compile_program(gl: &GL, vs_src: &str, fs_src: &str) -> WebGlProgram {
        let vs = compile_shader(gl, GL::VERTEX_SHADER, vs_src);
        let fs = compile_shader(gl, GL::FRAGMENT_SHADER, fs_src);
        let program = gl.create_program().expect("program");
        gl.attach_shader(&program, &vs);
        gl.attach_shader(&program, &fs);
        gl.link_program(&program);
        if !gl
            .get_program_parameter(&program, GL::LINK_STATUS)
            .as_bool()
            .unwrap_or(false)
        {
            let log = gl.get_program_info_log(&program).unwrap_or_default();
            web_sys::console::error_1(&format!("program link failed: {log}").into());
        }
        program
    }

    fn compile_shader(gl: &GL, ty: u32, src: &str) -> WebGlShader {
        let sh = gl.create_shader(ty).expect("shader");
        gl.shader_source(&sh, src);
        gl.compile_shader(&sh);
        if !gl
            .get_shader_parameter(&sh, GL::COMPILE_STATUS)
            .as_bool()
            .unwrap_or(false)
        {
            let log = gl.get_shader_info_log(&sh).unwrap_or_default();
            web_sys::console::error_1(&format!("shader compile failed: {log}").into());
        }
        sh
    }

    fn perspective(fov: f32, aspect: f32, near: f32, far: f32) -> [f32; 16] {
        let f = 1.0 / (fov / 2.0).tan();
        let nf = 1.0 / (near - far);
        [
            f / aspect, 0.0, 0.0, 0.0,
            0.0, f, 0.0, 0.0,
            0.0, 0.0, (far + near) * nf, -1.0,
            0.0, 0.0, 2.0 * far * near * nf, 0.0,
        ]
    }

    fn translation(x: f32, y: f32, z: f32) -> [f32; 16] {
        [
            1.0, 0.0, 0.0, 0.0,
            0.0, 1.0, 0.0, 0.0,
            0.0, 0.0, 1.0, 0.0,
            x,   y,   z,   1.0,
        ]
    }

    fn rotation_x(angle: f32) -> [f32; 16] {
        let c = angle.cos();
        let s = angle.sin();
        [
            1.0, 0.0, 0.0, 0.0,
            0.0,  c,   s,  0.0,
            0.0, -s,   c,  0.0,
            0.0, 0.0, 0.0, 1.0,
        ]
    }

    fn rotation_y(angle: f32) -> [f32; 16] {
        let c = angle.cos();
        let s = angle.sin();
        [
              c, 0.0, -s, 0.0,
            0.0, 1.0, 0.0, 0.0,
              s, 0.0,  c, 0.0,
            0.0, 0.0, 0.0, 1.0,
        ]
    }

    fn mat_mul(a: &[f32; 16], b: &[f32; 16]) -> [f32; 16] {
        let mut out = [0.0f32; 16];
        for col in 0..4 {
            for row in 0..4 {
                let mut sum = 0.0;
                for k in 0..4 {
                    sum += a[k * 4 + row] * b[col * 4 + k];
                }
                out[col * 4 + row] = sum;
            }
        }
        out
    }
}
