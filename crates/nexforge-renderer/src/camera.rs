pub struct Camera {
    pub position: [f32; 3],
    pub yaw: f32,
    pub pitch: f32,
    pub fov: f32,
    pub near: f32,
    pub far: f32,
    pub aspect: f32,
    pub speed: f32,
    pub sensitivity: f32,
}

impl Camera {
    pub fn new(aspect: f32) -> Self {
        Self {
            position: [0.0, 1.6, 5.0],
            yaw: -90.0f32.to_radians(),
            pitch: 0.0,
            fov: 70.0f32.to_radians(),
            near: 0.1,
            far: 1000.0,
            aspect,
            speed: 6.0,
            sensitivity: 0.002,
        }
    }

    pub fn forward(&self) -> [f32; 3] {
        let cos_pitch = self.pitch.cos();
        [
            self.yaw.cos() * cos_pitch,
            self.pitch.sin(),
            self.yaw.sin() * cos_pitch,
        ]
    }

    pub fn right(&self) -> [f32; 3] {
        let fwd = self.forward();
        let up = [0.0, 1.0, 0.0];
        [
            fwd[1] * up[2] - fwd[2] * up[1],
            fwd[2] * up[0] - fwd[0] * up[2],
            fwd[0] * up[1] - fwd[1] * up[0],
        ]
    }

    pub fn look_at_matrix(&self) -> [[f32; 4]; 4] {
        let fwd = self.forward();
        let side = self.right();
        let up = [
            side[1] * fwd[2] - side[2] * fwd[1],
            side[2] * fwd[0] - side[0] * fwd[2],
            side[0] * fwd[1] - side[1] * fwd[0],
        ];
        let tx = -(self.position[0] * side[0] + self.position[1] * side[1] + self.position[2] * side[2]);
        let ty = -(self.position[0] * up[0] + self.position[1] * up[1] + self.position[2] * up[2]);
        let tz = -(self.position[0] * fwd[0] + self.position[1] * fwd[1] + self.position[2] * fwd[2]);
        [
            [side[0], up[0], fwd[0], 0.0],
            [side[1], up[1], fwd[1], 0.0],
            [side[2], up[2], fwd[2], 0.0],
            [tx, ty, tz, 1.0],
        ]
    }

    pub fn perspective_matrix(&self) -> [[f32; 4]; 4] {
        let f = 1.0 / (self.fov * 0.5).tan();
        let range_inv = 1.0 / (self.near - self.far);
        [
            [f / self.aspect, 0.0, 0.0, 0.0],
            [0.0, f, 0.0, 0.0],
            [0.0, 0.0, (self.near + self.far) * range_inv, -1.0],
            [0.0, 0.0, self.near * self.far * range_inv * 2.0, 0.0],
        ]
    }

    pub fn vp_matrix(&self) -> [[f32; 4]; 4] {
        let view = self.look_at_matrix();
        let proj = self.perspective_matrix();
        let mut result = [[0.0f32; 4]; 4];
        for i in 0..4 {
            for j in 0..4 {
                result[i][j] = proj[i][0] * view[0][j] + proj[i][1] * view[1][j] + proj[i][2] * view[2][j] + proj[i][3] * view[3][j];
            }
        }
        result
    }

    pub fn update_mouse(&mut self, dx: f32, dy: f32) {
        self.yaw += dx * self.sensitivity;
        self.pitch = (self.pitch - dy * self.sensitivity).clamp(
            -89.0f32.to_radians(),
            89.0f32.to_radians(),
        );
    }

    pub fn update_keyboard(&mut self, horizontal: f32, vertical: f32, sprint: bool) {
        let speed = if sprint { self.speed * 2.0 } else { self.speed };
        let fwd = self.forward();
        let right = self.right();
        let len = (fwd[0] * fwd[0] + fwd[2] * fwd[2]).sqrt();
        let flat_fwd = if len > 0.0 { [fwd[0] / len, 0.0, fwd[2] / len] } else { [0.0, 0.0, 1.0] };
        let rlen = (right[0] * right[0] + right[2] * right[2]).sqrt();
        let flat_right = if rlen > 0.0 { [right[0] / rlen, 0.0, right[2] / rlen] } else { [1.0, 0.0, 0.0] };
        let dt = 0.016;
        self.position[0] += flat_fwd[0] * vertical * speed * dt;
        self.position[2] += flat_fwd[2] * vertical * speed * dt;
        self.position[0] += flat_right[0] * horizontal * speed * dt;
        self.position[2] += flat_right[2] * horizontal * speed * dt;
    }
}
