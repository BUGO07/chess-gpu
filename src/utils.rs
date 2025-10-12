use glam::Vec3;

pub struct Quad;

impl Quad {
    #[inline]
    pub const fn from(pos: Vec3, size: Vec3) -> [[f32; 3]; 4] {
        [
            [pos.x + size.x, pos.y, pos.z],
            [pos.x + size.x, pos.y + size.y, pos.z],
            [pos.x, pos.y + size.y, pos.z],
            [pos.x, pos.y, pos.z],
        ]
    }

    pub fn generate_indices(vertices: usize) -> Vec<u32> {
        (0..vertices)
            .step_by(4)
            .flat_map(|i| {
                let idx = i as u32;
                [idx, idx + 1, idx + 2, idx, idx + 2, idx + 3]
            })
            .collect::<Vec<_>>()
    }
}
