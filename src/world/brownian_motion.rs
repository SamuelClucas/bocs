use rand::prelude::*;
use rand_distr::{Normal, Distribution};

use crate::components::brownian_motion::BrownianMotion;
// TODO: write test for randomness
pub fn brownian_motion_system() {
    let mut rng = thread_rng();

    let normal = Normal::new(0.0, 1.0).unwrap();

    for (mut transform, motion) in query.iter_mut() {
        let dt = time.delta_secs();

        let dx = normal.sample(&mut rng) * motion.intensity * dt;
        let dy = normal.sample(&mut rng) * motion.intensity * dt;
        let dz = normal.sample(&mut rng) * motion.intensity * dt;

        transform.translation += Vec3::new(dx, dy, dz);
    }
}