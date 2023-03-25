use std::path::{Path, PathBuf};

use glutin::event::{Event, WindowEvent, ElementState};
use glutin::event_loop::{ControlFlow, EventLoop};
use glutin::window::WindowBuilder;
use glutin::ContextBuilder;
use glutin::event::KeyboardInput as KI;
use glutin::event::VirtualKeyCode as VKC;

use clap::{Arg, ArgAction, Command};

use gl46::*;

use cgmath::{Vector2, Vector3, Matrix4, SquareMatrix};

use faithful::model::{VertexModel, MeshModel};
use faithful::tex_model::{TexModel, TexVertex};
use faithful::view::*;

use faithful::pop::level::{LevelPaths, GlobeTextureParams};
use faithful::pop::objects::{Object3D, Vertex};
use faithful::pop::bl320::make_bl320_tex;

use faithful::opengl::gl::{GlCtx, new_gl_ctx};
use faithful::opengl::program::*;
use faithful::opengl::uniform::{GlUniform1, GlUniform1Cell};
use faithful::opengl::texture::*;
use faithful::envelop::*;

/******************************************************************************/

fn mk_tex_vertex(tex_index: i16, v: &Vertex) -> TexVertex {
    TexVertex{coord: Vector3::new(v.x, v.y, v.z)
             , uv: Vector2::new(v.u, v.v)
             , tex_id: tex_index}
}

fn mk_pop_object(object: &Object3D) -> TexModel {
    let mut model: TexModel = MeshModel::new();
    for face in object.iter_face() {
        if face.vertex_num == 3 {
            model.push_vertex(mk_tex_vertex(face.texture_index, &face.vertex[0]));
            model.push_vertex(mk_tex_vertex(face.texture_index, &face.vertex[1]));
            model.push_vertex(mk_tex_vertex(face.texture_index, &face.vertex[2]));
        } else {
            model.push_vertex(mk_tex_vertex(face.texture_index, &face.vertex[0]));
            model.push_vertex(mk_tex_vertex(face.texture_index, &face.vertex[1]));
            model.push_vertex(mk_tex_vertex(face.texture_index, &face.vertex[2]));
            model.push_vertex(mk_tex_vertex(face.texture_index, &face.vertex[2]));
            model.push_vertex(mk_tex_vertex(face.texture_index, &face.vertex[3]));
            model.push_vertex(mk_tex_vertex(face.texture_index, &face.vertex[0]));
        }
    }
    log::debug!("POP object mesh - vertices={:?}, indices={:?}"
                , model.vertex_num(), model.index_num());
    model
}

fn mk_pop_envelope(gl: &GlCtx, mvp_model: &GlUniform1Cell<Matrix4::<f32>>, object: &Object3D) -> ModelEnvelop<TexModel> {
    let model = mk_pop_object(object);
    let m = vec![(RenderType::Triangles, model)];
    let mut e = ModelEnvelop::<TexModel>::new(gl, mvp_model, m);
    if let Some(m) = e.get(0) {
        m.location[1] = -0.5;
        m.scale = (object.coord_scale() / 300.0) * 0.5;
    }
    e
}

/******************************************************************************/

fn cli() -> Command {
    let args = [
        Arg::new("base")
            .long("base")
            .action(ArgAction::Set)
            .value_name("BASE_PATH")
            .value_parser(clap::value_parser!(PathBuf))
            .help("Path to POP3 directory"),
        Arg::new("landtype")
            .long("landtype")
            .action(ArgAction::Set)
            .value_name("LAND_TYPE")
            .value_parser(clap::builder::StringValueParser::new())
            .help("Override land type"),
        Arg::new("debug")
            .long("debug")
            .action(ArgAction::SetTrue)
            .help("Enable debug printing"),
        Arg::new("obj_num")
            .long("obj_num")
            .action(ArgAction::Set)
            .value_name("OBJ")
            .value_parser(clap::value_parser!(u8).range(0..255))
            .help("Obj number"),
    ];
    Command::new("pop-obj-view")
        .about("POP3 object viewer")
        .args(&args)
}

fn main() {
    let matches = cli().get_matches();

    let base = {
        let base = matches.get_one("base").cloned();
        base.unwrap_or_else(|| Path::new("/opt/sandbox/pop").to_path_buf())
    };
    let landtype = matches.get_one("landtype").cloned().unwrap_or_else(|| "1".to_string());
    let debug = matches.get_flag("debug");
    let obj_num: Option<u8> = matches.get_one("obj_num").copied();

    let log_level: &str = if debug {
        "debug"
    } else {
        "info"
    };
    let env = env_logger::Env::default()
        .filter_or("F_LOG_LEVEL", log_level)
        .write_style_or("F_LOG_STYLE", "always");
    env_logger::init_from_env(env);

    let (level_paths, params) = {
        let data_dir = base.join("data");
        let paths = LevelPaths::from_base(&data_dir, &landtype);
        let params = GlobeTextureParams::from_level(&paths);
        (paths, params)
    };

    let objects_3d = Object3D::from_file(&base, "0");
 
    //

    let el = EventLoop::new();
    let wb = WindowBuilder::new().with_title("pop-obj-view");

    let windowed_context = ContextBuilder::new().build_windowed(wb, &el).unwrap();
    let windowed_context = unsafe { windowed_context.make_current().unwrap() };

    //GL
    let gl = new_gl_ctx(windowed_context.context());

    let mvp = GlUniform1::new_rc(Matrix4::<f32>::identity());
    let mvp_model = GlUniform1::new_rc(Matrix4::<f32>::identity());

    let mut program_objects = {
        let mut program = GlProgram::new(&gl);
        let loader = GlShaderLoaderBinary {};
        GlShader::attach_from_file("vert", Path::new("shaders/objects_1.vert.spv"), &mut program, &loader, GL_VERTEX_SHADER);
        GlShader::attach_from_file("frag", Path::new("shaders/objects_1.frag.spv"), &mut program, &loader, GL_FRAGMENT_SHADER);
        program
    };
    program_objects.use_program();

    let _bl320_tex = {
        let (width, height, bl320_tex) = make_bl320_tex(&level_paths.bl320, &params.palette);
        log::debug!("Texture {width:?}/{height:?}");
        {
            let params = TextureParams{target: GL_TEXTURE_2D, internal_format: GL_RGBA32F, format: GL_RGBA, data_type: GL_UNSIGNED_BYTE, nearest: false};
            let uniform = Some(0);
            GlTexture::new_2d(&gl, uniform, &params, width, height, &bl320_tex)
        }.unwrap()
    };

    let mut obj_num = match obj_num {
        Some(i) => i as usize,
        _ => 0,
    };

    if obj_num >= objects_3d.len() {
        log::error!("Object number is too big {:?} >= {:?}", obj_num, objects_3d.len());
        return;
    }

    log::debug!("{:?} {:?}", base, obj_num);
    let mut pop_obj = mk_pop_envelope(&gl, &mvp_model, &objects_3d[obj_num]);
    program_objects.set_uniform(0, mvp);
    program_objects.set_uniform(1, mvp_model.clone());

    let mut camera = Camera::new();
    camera.angle_x = -75;
    camera.angle_z = 60;
    let mut screen = Screen {width: 800, height: 600};

    let mut do_render = true;
    let mut scale = 1.0;
    let mut scale_origin = pop_obj.get(0).map(|m| m.scale).unwrap();
    el.run(move |event, _, control_flow| {
        log::trace!("{:?}", event);
        *control_flow = ControlFlow::Wait;

        match event {
            Event::LoopDestroyed => (),
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::Resized(physical_size) => {
                    screen.width = physical_size.width;
                    screen.height = physical_size.height;
                    do_render = true;
                    windowed_context.resize(physical_size);
                },
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                WindowEvent::KeyboardInput { input, .. } => match input {
                    KI { state: ElementState::Pressed, virtual_keycode: Some(VKC::Q), .. } => {
                        *control_flow = ControlFlow::Exit
                    },
                    KI { state: ElementState::Pressed, virtual_keycode: Some(key), .. } => {
                        let mut loc = Vector2::new(0.0, 0.0);
                        let mut angle = Vector3::new(0, 0, 0);
                        let mut obj_num_new = obj_num;
                        match key {
                            VKC::Up => {
                                angle[0] = 5;
                            },
                            VKC::Down => {
                                angle[0] = -5;
                            },
                            VKC::Left => {
                                angle[1] = 5;
                            },
                            VKC::Right => {
                                angle[1] = -5;
                            },
                            VKC::L => {
                                loc[0] = 0.1;
                            },
                            VKC::H => {
                                loc[0] = -0.1;
                            },
                            VKC::J => {
                                loc[1] = 0.1;
                            },
                            VKC::K => {
                                loc[1] = -0.1;
                            },
                            VKC::N => {
                                scale -= scale * 0.1;
                            },
                            VKC::M => {
                                scale += scale * 0.1;
                            },
                            VKC::V => {
                                obj_num_new = if obj_num > 0 { obj_num-1 } else { obj_num };
                            },
                            VKC::B => {
                                obj_num_new = if (obj_num+1) >= objects_3d.len() {obj_num} else {obj_num+1};
                            },
                            VKC::R => {
                                scale = 1.0;
                            },
                            _ => (),
                        }
                        if obj_num_new != obj_num {
                            obj_num = obj_num_new;
                            let (l, a) = pop_obj.get(0).map(|m| (m.location, m.angles)).unwrap();
                            pop_obj = mk_pop_envelope(&gl, &mvp_model, &objects_3d[obj_num]);
                            if let Some(m) = pop_obj.get(0) {
                                m.location = l;
                                m.angles = a;
                                scale_origin = m.scale;
                                m.scale = scale_origin * scale;
                            }
                        }
                        if let Some(m) = pop_obj.get(0) {
                            m.location[0] += loc[0];
                            m.location[1] += loc[1];
                            m.angles[0] += angle[0] as f32;
                            m.angles[1] += angle[1] as f32;
                            m.angles[2] += angle[2] as f32;
                            m.scale = scale_origin * scale;
                        }
                        do_render = true;
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
            unsafe {
                gl.Enable(GL_DEPTH_TEST);
                gl.Clear(GL_COLOR_BUFFER_BIT|GL_DEPTH_BUFFER_BIT);
                gl.ClearColor(0.0, 0.0, 0.0, 0.0);
                gl.LineWidth(3.0);
            }
            program_objects.use_program();
            pop_obj.draw(1);
            windowed_context.swap_buffers().unwrap();
            do_render = false;
        }
    });
}
