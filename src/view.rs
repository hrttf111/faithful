use cgmath::{Point2, Point3, Vector3, Matrix4, ortho, perspective, Rad, Deg, SquareMatrix, PerspectiveFov, Angle};

pub struct Camera {
    pub angle_x: i16,
    pub angle_y: i16,
    pub angle_z: i16,
    pub pos: Vector3<f32>,
}

pub struct Screen {
    pub width: u32,
    pub height: u32,
}

pub struct MVP {
    pub transform: Matrix4<f32>,
    pub view: Matrix4<f32>,
    pub projection: Matrix4<f32>,
    pub is_ortho: bool,
    pub eye: Vector3<f32>,
}

impl Camera {
    pub fn new() -> Self {
        Self{angle_x: 0, angle_y: 0, angle_z: 0, pos: Vector3{x: 0.0, y: 0.0, z: 0.0}}
    }
}

impl Default for Camera {
    fn default() -> Self {
        Self::new()
    }
}

impl MVP {
    pub fn new(screen: &Screen, camera: &Camera) -> MVP {
        let is_ortho = false;
        let angle_x = camera.angle_x as f32;
        let angle_y = camera.angle_y as f32;
        let angle_z = camera.angle_z as f32;
        let pos = camera.pos;

        let rot_move = Vector3{x: 0.0, y: -1.4, z: 0.0} - pos;
        let rot_move_r = -rot_move;

        let rot_transform = Matrix4::from_translation(rot_move)
                          * Matrix4::from_angle_z(Rad::from(Deg(angle_z)))
                          * Matrix4::from_translation(rot_move_r);

        let transform = Matrix4::from_angle_x(Rad::from(Deg(angle_x)))
                      * Matrix4::from_angle_y(Rad::from(Deg(angle_y)))
                      * Matrix4::from_translation(pos)
                      * rot_transform;

        let eye = Self::eye();
        let view: Matrix4<f32> = Matrix4::look_at_rh(Point3 { x: eye.x, y: eye.y, z: eye.z }
                                      , Point3 { x: 0.0, y: 0.0, z: 0.0 }
                                      , Vector3 { x: 0.0, y: 1.0, z: 0.0 });

        let projection: Matrix4<f32> = {
            let asp = screen.width as f32 / screen.height as f32;
            if is_ortho {
                let w = 1.0 * asp;
                let h = 1.0;
                ortho(-w, w, -h, h, 0.1, 30.0)
            } else {
                perspective( Rad(1.0), asp, 0.1, 10000.0 )
            }
        };

        MVP{transform, view, projection, is_ortho, eye}
    }

    pub fn trans(camera: &Camera) -> Matrix4<f32> {
        Matrix4::from_translation(camera.pos)
    }

    pub fn rm_x(camera: &Camera, pos: &mut Vector3<f32>) {
        let angle_x = camera.angle_x as f32;
        let mat = Matrix4::from_angle_x( Rad::from(Deg(angle_x)) ).invert().unwrap();
        *pos = (mat * pos.extend(1.0)).truncate()
    }

    pub fn trans1(camera: &Camera) -> Matrix4<f32> {
        let pos = camera.pos;
        let angle_x = camera.angle_x as f32;
        let angle_y = camera.angle_y as f32;
        let angle_z = camera.angle_z as f32;
        Matrix4::from_angle_x( Rad::from(Deg(angle_x)) )
                        * Matrix4::from_angle_y( Rad::from(Deg(angle_y)) )
                        * Matrix4::from_angle_z( Rad::from(Deg(angle_z)) )
                        * Matrix4::from_translation( pos )
    }

    pub fn make_fov(screen: &Screen) -> PerspectiveFov<f32> {
        let aspect = screen.width as f32 / screen.height as f32;
        PerspectiveFov{fovy: Rad(1.0), aspect, near: 1.0, far: 10000.0}
    }

    pub fn eye() -> Vector3<f32> {
        Vector3{x: 0.0, y: 0.0, z: 4.0}
    }
}

/*
 * opengl (x;y) + asp
 * (-1;1) (1;1)
 * (-1;-1) (1:-1)
 *
 * screen (w;h)
 * (0;0) (w;0)
 * (0;h) (w;h)
 */
pub fn screen_to_scene(screen: &Screen, camera: &Camera, pos_screen: &Point2<f32>) -> (Vector3<f32>, Vector3<f32>) {
    let mvp = MVP::new(screen, camera);
    let asp = screen.width as f32 / screen.height as f32;
    let x: f32 = {
        let w = screen.width as f32;
        asp * 2.0 * (pos_screen.x as f32 - w / 2.0) / w
    };
    let y: f32 = {
        let h = screen.height as f32;
        2.0*(h / 2.0 - pos_screen.y as f32) / h
    };

    if mvp.is_ortho {
        let vec_screen_s: Vector3<f32> = Vector3 { x, y, z: 10.0 };
        let vec_screen_e: Vector3<f32> = Vector3 { x, y, z: -10000.0 };
        let mvp_m = mvp.view * mvp.transform;
        let mvp_t = mvp_m.invert().unwrap();
        let v1 = (mvp_t * vec_screen_s.extend(1.0)).truncate();
        let v2 = (mvp_t * vec_screen_e.extend(1.0)).truncate();
        (v1, v2)
    } else {
        let vec_screen_s: Vector3<f32> = mvp.eye;
        let vec_screen_e: Vector3<f32> = {
            let per_fov = MVP::make_fov(screen);
            let x_norm = x / asp;
            let y_norm = y;
            let z = 10000.0;
            let far_ymax = z * Rad::tan(Rad(1.0) / 2.0);
            let far_xmax = far_ymax * per_fov.aspect;
            let x_far = far_xmax * x_norm;
            let y_far = far_ymax * y_norm;
            Vector3{x: x_far, y: y_far, z: -z}
        };
        let v1 = {
            let mvp_m = /*mvp.view * */mvp.transform;
            let mvp_t = mvp_m.invert().unwrap();
            (mvp_t * vec_screen_s.extend(1.0)).truncate()
        };
        let v2 = {
            let mvp_t1 = (mvp.view * mvp.transform).invert().unwrap();
            (mvp_t1 * vec_screen_e.extend(1.0)).truncate()
        };
        (v1, v2)
    }
}

pub fn camera_dir_to_scene(screen: &Screen, camera: &Camera) -> (Vector3<f32>, Vector3<f32>) {
    let mvp = MVP::new(screen, camera);
    let v1 = {
        let eye = mvp.eye;
        let mvp_m = mvp.transform;
        let mvp_t = mvp_m.invert().unwrap();
        (mvp_t * eye.extend(1.0)).truncate()
    };
    let v2 = {
        let pos_end: Vector3<f32> = Vector3{x: 0.0, y: 0.0, z: -0.1};
        let mvp_t = (mvp.view * mvp.transform).invert().unwrap();
        (mvp_t * pos_end.extend(1.0)).truncate()
    };
    (v1, v2)
}
