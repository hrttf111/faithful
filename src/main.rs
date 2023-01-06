use std::rc::Rc;
use std::cell::RefCell;
use std::path::Path;

use glutin::event::{Event, WindowEvent, ElementState};
use glutin::event_loop::{ControlFlow, EventLoop};
use glutin::window::WindowBuilder;
use glutin::ContextBuilder;
use glutin::event::KeyboardInput as KI;
use glutin::event::VirtualKeyCode as VKC;

use gl46::*;

use cgmath::{Point2, Vector3, Vector4, Matrix4, SquareMatrix};

use faithful::model::{VertexModel, MeshModel};
use faithful::default_model::DefaultModel;
use faithful::view::*;

use faithful::intersect::intersect_iter;

use faithful::landscape::{LandscapeMesh, LandscapeModel};
use faithful::pop::landscape::land::{make_texture_land, LevelRes, draw_texture};

use faithful::opengl::gl::{GlCtx, new_gl_ctx};
use faithful::opengl::program::*;
use faithful::opengl::uniform::{GlUniform1, GlUniform1Cell};
use faithful::opengl::texture::*;
use faithful::envelop::*;

/******************************************************************************/

type LandscapeMeshS = LandscapeMesh<128>;

#[derive(Debug, PartialEq, Clone)]
enum ActionMode {
    GlobalMoveXY,
    GlobalMoveXZ,
    GlobalRotateXZ,
    GlobalRotateXY,
    GlobalMoveRot,
}

impl ActionMode {
    fn process_key(&mut self, key: VKC, camera: &mut Camera, cam: &mut Vector3<f32>) -> bool {
        match self {
            Self::GlobalRotateXZ =>
                match key {
                    VKC::Up => {
                        camera.angle_x += 5;
                    },
                    VKC::Down => {
                        camera.angle_x -= 5;
                    },
                    VKC::Left => {
                        camera.angle_z += 5;
                    },
                    VKC::Right => {
                        camera.angle_z -= 5;
                    },
                    VKC::P => {
                        *self = Self::GlobalRotateXY;
                        println!("{:?}", self);
                    },
                    _ => (),
                },
            Self::GlobalRotateXY =>
                match key {
                    VKC::Up => {
                        camera.angle_x += 5;
                    },
                    VKC::Down => {
                        camera.angle_x -= 5;
                    },
                    VKC::Left => {
                        camera.angle_y += 5;
                    },
                    VKC::Right => {
                        camera.angle_y -= 5;
                    },
                    VKC::P => {
                        *self = Self::GlobalMoveXY;
                        println!("{:?}", self);
                    },
                    _ => (),
                },
            Self::GlobalMoveXY =>
                match key {
                    VKC::Up => {
                        camera.pos.x += 0.1;
                    },
                    VKC::Down => {
                        camera.pos.x -= 0.1;
                    },
                    VKC::Left => {
                        camera.pos.y += 0.1;
                    },
                    VKC::Right => {
                        camera.pos.y -= 0.1;
                    },
                    VKC::P => {
                        *self = Self::GlobalMoveXZ;
                        println!("{:?}", self);
                    },
                    _ => (),
                },
            Self::GlobalMoveXZ =>
                match key {
                    VKC::Up => {
                        camera.pos.z += 0.1;
                    },
                    VKC::Down => {
                        camera.pos.z -= 0.1;
                    },
                    VKC::Left => {
                        camera.pos.z += 0.1;
                    },
                    VKC::Right => {
                        camera.pos.z -= 0.1;
                    },
                    VKC::P => {
                        *self = Self::GlobalMoveRot;
                        println!("{:?}", self);
                    },
                    _ => (),
                },
            Self::GlobalMoveRot =>
                match key {
                    VKC::Up => {
                        cam.z = -1.5;
                    },
                    VKC::Down => {
                        cam.z = 1.5;
                    },
                    VKC::Left => {
                        camera.angle_z -= 5;
                    },
                    VKC::Right => {
                        camera.angle_z += 5;
                    },
                    VKC::P => {
                        *self = Self::GlobalRotateXZ;
                        println!("{:?}", self);
                    },
                    _ => (),
                },
            //_ => (),
        }
        true
    }
}

struct Scene {
    model_main: ModelEnvelop<LandscapeModel>,
    model_select: ModelEnvelop<DefaultModel>,
    select_frag: i32,
}

#[allow(dead_code)]
struct LevelAssets {
    tex_palette: GlTexture,
    tex_disp: GlTexture,
    tex_bigf: GlTexture,
    tex_sla: GlTexture,
}

impl LevelAssets {
    fn new(gl: &GlCtx, level_res: &LevelRes) -> Self {
        let tex_palette = {
            let params = TextureParams{target: GL_TEXTURE_1D, internal_format: GL_RGBA8UI, format: GL_RGBA_INTEGER, data_type: GL_UNSIGNED_BYTE, nearest: true};
            let uniform = Some(1);
            let width = level_res.landscape.land_size();
            GlTexture::new_1d(gl, uniform, &params, width, level_res.params.palette.as_slice())
        }.unwrap();

        let tex_disp = {
            let uniform = Some(2);
            let width = level_res.params.disp0.len();
            GlTexture::new_buffered(gl, uniform, GL_R8I, width, level_res.params.disp0.as_slice())
        }.unwrap();

        let tex_bigf = {
            let uniform = Some(3);
            let width = level_res.params.bigf0.len();
            GlTexture::new_buffered(gl, uniform, GL_R8UI, width, level_res.params.bigf0.as_slice())
        }.unwrap();

        let tex_sla = {
            let uniform = Some(4);
            let width = level_res.params.static_landscape_array.len() * std::mem::size_of::<u16>();
            GlTexture::new_buffered(gl, uniform, GL_R16UI, width, level_res.params.static_landscape_array.as_slice())
        }.unwrap();

        LevelAssets{tex_palette, tex_disp, tex_bigf, tex_sla}
    }

    pub fn update(&mut self, level_res: &LevelRes) {
        self.tex_palette.set_data(level_res.params.palette.as_slice());
        self.tex_disp.set_data(level_res.params.disp0.as_slice());
        self.tex_bigf.set_data(level_res.params.bigf0.as_slice());
    }
}

struct LevelUniforms {
    uniform_mvp: GlUniform1Cell<Matrix4::<f32>>,
    uniform_selected: GlUniform1Cell<i32>,
    uniform_selected_color: GlUniform1Cell<Vector4::<f32>>,
    uniform_mvp_model: GlUniform1Cell<Matrix4::<f32>>,
    level_shift: GlUniform1Cell<Vector4::<f32>>,
    height_scale: GlUniform1Cell<f32>,
    heights: GlUniform1Cell<Vec<u32>>,
}

trait LandscapeProgram {
    fn gl_program(&self, index: usize) -> &GlProgram;
    fn update(&mut self, level_res: &LevelRes);
}

struct LandscapeProgramContainer {
    programs: Vec<Rc<RefCell<dyn LandscapeProgram>>>,
    index: usize,
}

impl LandscapeProgramContainer {
    fn new() -> Self {
        Self{ programs: Vec::new(), index: 0 }
    }

    fn add_program(&mut self, program: Rc<RefCell<dyn LandscapeProgram>>) {
        self.programs.push(program);
    }

    fn next(&mut self) {
        self.index = (self.index + 1) % self.programs.len();
    }

    fn prev(&mut self) {
        if self.programs.is_empty() {
            return;
        }
        self.index = if self.index == 0 {
            self.programs.len() - 1
        } else {
            (self.index - 1) % self.programs.len()
        };
    }

    fn update_programs(&mut self, level_res: &LevelRes) {
        for program in self.programs.iter_mut() {
            program.borrow_mut().update(level_res);
        }
    }

    fn get_program(&self) -> Option<Rc<RefCell<dyn LandscapeProgram>>> {
        self.programs.get(self.index).cloned()
    }
}

struct MainLandscapeProgram {
    program: GlProgram,
    textures: LevelAssets,
}

impl MainLandscapeProgram {
    fn new(gl: &GlCtx, level_res: &LevelRes, uniforms: &LevelUniforms) -> Self {
        let mut program = {
            let mut program = GlProgram::new(gl);
            let loader = GlShaderLoaderBinary {};
            GlShader::attach_from_file("vert", "shaders/landscape.vert.spv", &mut program, &loader, GL_VERTEX_SHADER);
            GlShader::attach_from_file("frag", "shaders/landscape.frag.spv", &mut program, &loader, GL_FRAGMENT_SHADER);
            program
        };
        program.use_program();
        let assets = LevelAssets::new(gl, level_res);
        program.set_uniform(0, uniforms.uniform_mvp.clone());
        program.set_uniform(1, uniforms.uniform_mvp_model.clone());
        program.set_uniform(2, uniforms.level_shift.clone());
        program.set_uniform(3, uniforms.height_scale.clone());
        program.set_uniform(4, uniforms.uniform_selected_color.clone());
        program.set_uniform(6, uniforms.uniform_selected.clone());
        program.set_uniform(7, uniforms.heights.clone());

        //let program_info = program.get_info().unwrap();
        //println!("Program info {}", program_info);

        //let program_log = program.get_log().unwrap();
        //println!("Program log: {}", program_log);

        MainLandscapeProgram{program, textures: assets}
    }

    fn new_rc_ref(gl: &GlCtx, level_res: &LevelRes, uniforms: &LevelUniforms) -> Rc<RefCell<dyn LandscapeProgram>> {
        Rc::new(RefCell::new(Self::new(gl, level_res, uniforms)))
    }
}

impl LandscapeProgram for MainLandscapeProgram {
    fn gl_program(&self, _index: usize) -> &GlProgram {
        &self.program
    }

    fn update(&mut self, level_res: &LevelRes) {
        self.textures.update(level_res);
    }
}

struct CpuLandscapeProgram {
    program: GlProgram,
    texture: GlTexture,
    tex_palette: GlTexture,
}

impl CpuLandscapeProgram {
    fn new(gl: &GlCtx, level_res: &LevelRes, landscape: &[u8], uniforms: &LevelUniforms) -> Self {
        let mut program = {
            let mut program = GlProgram::new(gl);
            let loader = GlShaderLoaderBinary {};
            GlShader::attach_from_file("vert", "shaders/landscape.vert.spv", &mut program, &loader, GL_VERTEX_SHADER);
            GlShader::attach_from_file("frag", "shaders/landscape_cpu.frag.spv", &mut program, &loader, GL_FRAGMENT_SHADER);
            program
        };

        program.use_program();

        let texture = {
            let params = TextureParams{target: GL_TEXTURE_2D, internal_format: GL_R8UI, format: GL_RED_INTEGER, data_type: GL_UNSIGNED_BYTE, nearest: true};
            let uniform = Some(0);
            let size = level_res.landscape.land_size() * 32;
            let width = size;
            let height = size;
            let texture = landscape;
            GlTexture::new_2d(gl, uniform, &params, width, height, texture)
        }.unwrap();

        let tex_palette = {
            let params = TextureParams{target: GL_TEXTURE_1D, internal_format: GL_RGBA8UI, format: GL_RGBA_INTEGER, data_type: GL_UNSIGNED_BYTE, nearest: true};
            let uniform = Some(1);
            let width = level_res.landscape.land_size();
            GlTexture::new_1d(gl, uniform, &params, width, level_res.params.palette.as_slice())
        }.unwrap();

        program.set_uniform(0, uniforms.uniform_mvp.clone());
        program.set_uniform(1, uniforms.uniform_mvp_model.clone());
        program.set_uniform(2, uniforms.level_shift.clone());
        program.set_uniform(3, uniforms.height_scale.clone());
        program.set_uniform(4, uniforms.uniform_selected_color.clone());
        program.set_uniform(6, uniforms.uniform_selected.clone());
        program.set_uniform(7, uniforms.heights.clone());

        CpuLandscapeProgram{program, texture, tex_palette}
    }

    fn new_rc_ref(gl: &GlCtx, level_res: &LevelRes, uniforms: &LevelUniforms) -> Rc<RefCell<dyn LandscapeProgram>> {
        let land_texture = make_texture_land(level_res, None);
        Rc::new(RefCell::new(Self::new(gl, level_res, &land_texture, uniforms)))
    }
}

impl LandscapeProgram for CpuLandscapeProgram {
    fn gl_program(&self, _index: usize) -> &GlProgram {
        &self.program
    }

    fn update(&mut self, level_res: &LevelRes) {
        let land_texture = make_texture_land(level_res, None);
        self.texture.set_data(&land_texture);
        self.tex_palette.set_data(level_res.params.palette.as_slice());
    }
}

struct CpuFullLandscapeProgram {
    program: GlProgram,
    texture: GlTexture,
}

impl CpuFullLandscapeProgram {
    fn new(gl: &GlCtx, level_res: &LevelRes, landscape: &[u8], uniforms: &LevelUniforms) -> Self {
        let mut program = {
            let mut program = GlProgram::new(gl);
            let loader = GlShaderLoaderBinary {};
            GlShader::attach_from_file("vert", "shaders/landscape.vert.spv", &mut program, &loader, GL_VERTEX_SHADER);
            GlShader::attach_from_file("frag", "shaders/landscape_full.frag.spv", &mut program, &loader, GL_FRAGMENT_SHADER);
            program
        };

        program.use_program();

        let texture = {
            let params = TextureParams{target: GL_TEXTURE_2D, internal_format: GL_RGB32F, format: GL_RGB, data_type: GL_FLOAT, nearest: false};
            let uniform = Some(6);
            let size = level_res.landscape.land_size() * 32;
            let width = size;
            let height = size;
            let texture = draw_texture(&level_res.params.palette, width, landscape);
            GlTexture::new_2d(gl, uniform, &params, width, height, &texture)
        }.unwrap();

        program.set_uniform(0, uniforms.uniform_mvp.clone());
        program.set_uniform(1, uniforms.uniform_mvp_model.clone());
        program.set_uniform(2, uniforms.level_shift.clone());
        program.set_uniform(3, uniforms.height_scale.clone());
        program.set_uniform(4, uniforms.uniform_selected_color.clone());
        program.set_uniform(6, uniforms.uniform_selected.clone());
        program.set_uniform(7, uniforms.heights.clone());

        CpuFullLandscapeProgram{program, texture}
    }

    fn new_rc_ref(gl: &GlCtx, level_res: &LevelRes, uniforms: &LevelUniforms) -> Rc<RefCell<dyn LandscapeProgram>> {
        let land_texture = make_texture_land(level_res, None);
        Rc::new(RefCell::new(Self::new(gl, level_res, &land_texture, uniforms)))
    }
}

impl LandscapeProgram for CpuFullLandscapeProgram {
    fn gl_program(&self, _index: usize) -> &GlProgram {
        &self.program
    }

    fn update(&mut self, level_res: &LevelRes) {
        let land_texture = make_texture_land(level_res, None);
        let size = level_res.landscape.land_size() * 32;
        let texture = draw_texture(&level_res.params.palette, size, &land_texture);
        self.texture.set_data(&texture);
    }
}

struct GradLandscapeProgram {
    program: GlProgram,
}

impl GradLandscapeProgram {
    fn new(gl: &GlCtx, uniforms: &LevelUniforms) -> Self {
        let mut program = {
            let mut program = GlProgram::new(gl);
            let loader = GlShaderLoaderBinary {};
            GlShader::attach_from_file("vert", "shaders/landscape.vert.spv", &mut program, &loader, GL_VERTEX_SHADER);
            GlShader::attach_from_file("frag", "shaders/landscape_grad.frag.spv", &mut program, &loader, GL_FRAGMENT_SHADER);
            program
        };

        program.use_program();

        program.set_uniform(0, uniforms.uniform_mvp.clone());
        program.set_uniform(1, uniforms.uniform_mvp_model.clone());
        program.set_uniform(2, uniforms.level_shift.clone());
        program.set_uniform(3, uniforms.height_scale.clone());
        program.set_uniform(4, uniforms.uniform_selected_color.clone());
        program.set_uniform(6, uniforms.uniform_selected.clone());
        program.set_uniform(7, uniforms.heights.clone());

        GradLandscapeProgram{program}
    }

    fn new_rc_ref(gl: &GlCtx, uniforms: &LevelUniforms) -> Rc<RefCell<dyn LandscapeProgram>> {
        Rc::new(RefCell::new(Self::new(gl, uniforms)))
    }
}

impl LandscapeProgram for GradLandscapeProgram {
    fn gl_program(&self, _index: usize) -> &GlProgram {
        &self.program
    }

    fn update(&mut self, _level_res: &LevelRes) {
    }
}

fn make_landscape_mode(gl: &GlCtx, uniforms: &LevelUniforms, landscape_mesh: &LandscapeMeshS) -> ModelEnvelop<LandscapeModel> {
    let mut model_main = {
        let mut model: LandscapeModel = MeshModel::new();
        landscape_mesh.to_model(&mut model);
        println!("Landscape mesh - vertices={:?}, indices={:?}"
                 , model.vertex_num(), model.index_num());
        ModelEnvelop::<LandscapeModel>::new(gl, &uniforms.uniform_mvp_model, vec![(RenderType::Triangles, model)])
    };
    if let Some(m) = model_main.get(0) {
        m.location.x = -2.0;
        m.location.y = -2.0;
        m.scale = 2.5;
    }
    model_main
}

fn update_level(base: &Path, level_num: u8, landscape_mesh: &mut LandscapeMeshS, uniforms: &LevelUniforms, program_container: &mut LandscapeProgramContainer) -> RefCell<LevelRes> {
    let level_res = {
        let level_type = None;
        LevelRes::new(base, level_num, level_type)
    };
    landscape_mesh.set_heights(&level_res.landscape.height);
    uniforms.heights.borrow_mut().set({
        let landscape = level_res.landscape.make_shores();
        landscape.to_vec()
    });
    program_container.update_programs(&level_res);
    RefCell::new(level_res)
}

fn render(gl: &GlCtx, program_landscape: &GlProgram, program_objects: &GlProgram, scene: &Scene) {
    unsafe {
        gl.Enable(GL_DEPTH_TEST);
        gl.Clear(GL_COLOR_BUFFER_BIT|GL_DEPTH_BUFFER_BIT);
        gl.ClearColor(0.0, 0.0, 0.0, 0.0);
        gl.LineWidth(3.0);
    }
    program_landscape.use_program();
    scene.model_main.draw(1);
    program_objects.use_program();
    scene.model_select.draw(1);
}

fn main() {
    let el = EventLoop::new();
    let wb = WindowBuilder::new().with_title("Rust OpenGL test");

    let windowed_context = ContextBuilder::new().build_windowed(wb, &el).unwrap();
    let windowed_context = unsafe { windowed_context.make_current().unwrap() };

    println!("Pixel format of the window's GL context: {:?}", windowed_context.get_pixel_format());
    let mut level_num = 1;
    let base = Path::new("/opt/sandbox/pop");

    let mut level_res = {
        let level_type = None;
        RefCell::new(LevelRes::new(base, level_num, level_type))
    };

    let mut landscape_mesh: LandscapeMeshS = {
        let mut landscape_mesh = LandscapeMesh::new(1.0/16.0, 1.0/65536.0 * 16.0);
        let lr = &level_res.borrow_mut();
        landscape_mesh.set_heights(&lr.landscape.height);
        landscape_mesh
    };

    //GL
    let gl = new_gl_ctx(windowed_context.context());

    let uniforms = LevelUniforms {
        uniform_mvp: GlUniform1::new_rc(Matrix4::<f32>::identity()),
        uniform_selected: GlUniform1::new_rc(0),
        uniform_selected_color: GlUniform1::new_rc(Vector4::<f32>::new(1.0, 0.0, 0.0, 0.0)),
        uniform_mvp_model: GlUniform1::new_rc(Matrix4::<f32>::identity()),
        level_shift: GlUniform1::new_rc(landscape_mesh.get_shift_vector()),
        height_scale: GlUniform1::new_rc(landscape_mesh.height_scale()),
        heights: {
            let landscape = level_res.borrow_mut().landscape.make_shores();
            let vec = landscape.to_vec();
            GlUniform1::new_rc(vec)
        },
    };

    let mut program_objects = {
        let mut program = GlProgram::new(&gl);
        let loader = GlShaderLoaderBinary {};
        GlShader::attach_from_file("vert", "shaders/objects.vert.spv", &mut program, &loader, GL_VERTEX_SHADER);
        GlShader::attach_from_file("frag", "shaders/objects.frag.spv", &mut program, &loader, GL_FRAGMENT_SHADER);
        program
    };

    let uniform_frag: GlUniform1Cell<i32> = GlUniform1::new_rc(1);
    program_objects.use_program();
    program_objects.set_uniform(0, uniforms.uniform_mvp.clone());
    program_objects.set_uniform(1, uniforms.uniform_mvp_model.clone());
    program_objects.set_uniform(2, uniforms.uniform_selected.clone());
    program_objects.set_uniform(3, uniform_frag);

    let mut program_container = LandscapeProgramContainer::new();
    program_container.add_program(MainLandscapeProgram::new_rc_ref(&gl, &level_res.borrow_mut(), &uniforms));
    program_container.add_program(GradLandscapeProgram::new_rc_ref(&gl, &uniforms));

    let model_main = make_landscape_mode(&gl, &uniforms, &landscape_mesh);

    let model_select = {
        let mut model: DefaultModel = MeshModel::new();
        model.push_vertex(Vector3::new(0.0, 0.0, 0.0));
        model.push_vertex(Vector3::new(0.0, 0.0, 0.0));
        let m = vec![(RenderType::Lines, model)];
        ModelEnvelop::<DefaultModel>::new(&gl, &uniforms.uniform_mvp_model, m)
    };

    let mut scene = Scene {
        model_main,
        model_select,
        select_frag: -1,
    };

    let mut camera = Camera::new();
    camera.angle_x = -75;
    camera.angle_z = 60;
    let mut screen = Screen {width: 800, height: 600};

    println!("OpenGL init done");

    let mut cpu_programs_created = false;
    let mut do_render = true;
    let mut mouse_pos = Point2::<f32>::new(0.0, 0.0);
    let mut mode = ActionMode::GlobalMoveRot;
    el.run(move |event, _, control_flow| {
        //println!("{:?}", event);
        *control_flow = ControlFlow::Wait;

        match event {
            Event::LoopDestroyed => (),
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CursorMoved { position, .. } => {
                    mouse_pos = Point2::<f32>::new(position.x as f32, position.y as f32);
                },
                WindowEvent::MouseInput { state, .. } => {
                    if state == ElementState::Pressed {
                        let (v1, v2) = screen_to_scene(&screen, &camera, &mouse_pos);
                        //println!("\nIntersect({mouse_pos:?}): {v1:?} - {v2:?}");
                        if let Some(m) = scene.model_select.get(0) {
                            m.model.set_vertex(0, v1);
                            m.model.set_vertex(1, v2);
                        }

                        let mvp = scene.model_main.get(0).map(|m| m.transform()).unwrap();
                        let iter = landscape_mesh.iter();
                        match intersect_iter(iter, &mvp, v1, v2) {
                            Some((n, _)) => scene.select_frag = n as i32,
                            None => scene.select_frag = -1,
                        }
                        do_render = true;
                    }
                },
                WindowEvent::Resized(physical_size) => {
                    screen.width = physical_size.width;
                    screen.height = physical_size.height;
                    do_render = true;
                    windowed_context.resize(physical_size);
                },
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                WindowEvent::KeyboardInput { input, .. } => match input {
                    KI { state: ElementState::Pressed, virtual_keycode: Some(VKC::R), .. } => {
                        camera.angle_x = 0;
                        camera.angle_y = 0;
                        camera.angle_z = 0;
                        camera.pos = Vector3{x: 0.0, y: 0.0, z: 0.0};
                        do_render = true;
                    },
                    KI { state: ElementState::Pressed, virtual_keycode: Some(VKC::T), .. } => {
                        camera.angle_x = -90;
                        do_render = true;
                    },
                    KI { state: ElementState::Pressed, virtual_keycode: Some(VKC::Q), .. } => {
                        *control_flow = ControlFlow::Exit
                    },
                    KI { state: ElementState::Pressed, virtual_keycode: Some(VKC::N), .. } => {
                        program_container.next();
                        do_render = true;
                    },
                    KI { state: ElementState::Pressed, virtual_keycode: Some(VKC::M), .. } => {
                        program_container.prev();
                        do_render = true;
                    },
                    KI { state: ElementState::Pressed, virtual_keycode: Some(VKC::U), .. } => {
                        if !cpu_programs_created {
                            program_container.add_program(CpuFullLandscapeProgram::new_rc_ref(&gl, &level_res.borrow_mut(), &uniforms));
                            program_container.add_program(CpuLandscapeProgram::new_rc_ref(&gl, &level_res.borrow_mut(), &uniforms));
                            cpu_programs_created = true;
                        }
                    },
                    KI { state: ElementState::Pressed, virtual_keycode: Some(VKC::B), .. } => {
                        level_num = (level_num + 1) % 26;
                        if level_num == 0 {
                            level_num = 1;
                        }
                        level_res = update_level(base, level_num, &mut landscape_mesh, &uniforms, &mut program_container);
                        do_render = true;
                    },
                    KI { state: ElementState::Pressed, virtual_keycode: Some(VKC::V), .. } => {
                        level_num = if level_num == 1 { 25 } else { level_num - 1 };
                        level_res = update_level(base, level_num, &mut landscape_mesh, &uniforms, &mut program_container);
                        do_render = true;
                    },
                    KI { state: ElementState::Pressed, virtual_keycode: Some(VKC::L), .. } => {
                        landscape_mesh.shift_y(1);
                        uniforms.level_shift.borrow_mut().set(landscape_mesh.get_shift_vector());
                        do_render = true;
                    },
                    KI { state: ElementState::Pressed, virtual_keycode: Some(VKC::H), .. } => {
                        landscape_mesh.shift_y(-1);
                        uniforms.level_shift.borrow_mut().set(landscape_mesh.get_shift_vector());
                        do_render = true;
                    },
                    KI { state: ElementState::Pressed, virtual_keycode: Some(VKC::J), .. } => {
                        landscape_mesh.shift_x(1);
                        uniforms.level_shift.borrow_mut().set(landscape_mesh.get_shift_vector());
                        do_render = true;
                    },
                    KI { state: ElementState::Pressed, virtual_keycode: Some(VKC::K), .. } => {
                        landscape_mesh.shift_x(-1);
                        uniforms.level_shift.borrow_mut().set(landscape_mesh.get_shift_vector());
                        do_render = true;
                    },
                    KI { state: ElementState::Pressed, virtual_keycode: Some(key), .. } => {
                        let mut pos: Vector3<f32> = Vector3{x: 0.0, y: 0.0, z: 0.0};
                        do_render = mode.process_key(key, &mut camera, &mut pos);
                        if pos.z != 0.0 {
                            let (mut v1, mut v2) = camera_dir_to_scene(&screen, &camera);
                            v1.z = 0.0;
                            v2.z = 0.0;
                            camera.pos += pos.z * (v2 - v1);
                            do_render = true;
                        }
                    },
                    _ => (),
                },
                _ => (),
            },
            Event::RedrawRequested(_) => {
                do_render = true;
                windowed_context.swap_buffers().unwrap();
            }
            _ => (),
        }
        if do_render {
            {
                scene.model_select.update_model(0);
                let mvp = MVP::new(&screen, &camera);
                let mvp_m = mvp.projection * mvp.view * mvp.transform;
                uniforms.uniform_mvp.borrow_mut().set(mvp_m);
                uniforms.uniform_selected.borrow_mut().set(scene.select_frag);
            }
            if let Some(p) = program_container.get_program() {
                render(&gl, p.borrow_mut().gl_program(0), &program_objects, &scene);
            } else {
                panic!("No program to render");
            }
            windowed_context.swap_buffers().unwrap();
            do_render = false;
        }
    });
}
