use rand_distr::{Distribution, Normal};
use std::convert::TryInto;
use std::ops::Add;

#[repr(u8)]
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum MicelleState {
    Outside = 0,
    Inside = 1,
}
// TODO: replace generic AnyVec3 with Enum, using pattern matching
#[derive(Copy, Clone, PartialEq, PartialOrd, Debug)]
pub struct AnyVec3<T> {
    pub x: T,
    pub y: T,
    pub z: T,
}

impl<T> AnyVec3<T> {
    pub fn new(x: T, y: T, z: T) -> Self {
        Self { x, y, z }
    }

    pub fn try_into_vec3<U>(self) -> Result<AnyVec3<U>, T::Error>
    where
        T: TryInto<U>,
    {
        Ok(AnyVec3 {
            x: self.x.try_into()?,
            y: self.y.try_into()?,
            z: self.z.try_into()?,
        })
    }
}

impl<T> Add for AnyVec3<T>
where
    T: Add<Output = T>,
{
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

impl AnyVec3<i32> {
    pub fn cmplt(&self, other: Self) -> AnyVec3<bool> {
        AnyVec3 {
            x: self.x < other.x,
            y: self.y < other.y,
            z: self.z < other.z,
        }
    }

    pub fn cmpge(&self, other: Self) -> AnyVec3<bool> {
        AnyVec3 {
            x: self.x >= other.x,
            y: self.y >= other.y,
            z: self.z >= other.z,
        }
    }

    pub fn as_uvec3(self) -> AnyVec3<usize> {
        AnyVec3 {
            x: self.x as usize,
            y: self.y as usize,
            z: self.z as usize,
        }
    }
}

impl AnyVec3<bool> {
    pub fn all(&self) -> bool {
        self.x && self.y && self.z
    }
}
// TODO: refactor CpuVoxelGrid as a compute shader
pub struct CpuVoxelGrid {
    pub dims: AnyVec3<usize>,
    pub voxel_count: usize,
    pub lipid_species: usize,
    pub concs: Vec<f32>, // species-major
    pub temps: Vec<f32>,
    pub is_micelle: Vec<MicelleState>,
}

impl CpuVoxelGrid {
    pub fn new(size: AnyVec3<usize>, species_count: usize, start_temp: f32) -> Self {
        let voxel_count = size.x * size.y * size.z;
        let total_conc = voxel_count * species_count;

        let temps = vec![start_temp; voxel_count];
        let is_micelle = vec![MicelleState::Outside; voxel_count];

        let mut concs = Vec::with_capacity(total_conc);
        let mut rng = rand::rng();
        let normal = Normal::new(0.5, 0.1).unwrap();

        for _ in 0..total_conc {
            let val = (normal.sample(&mut rng)as f32).max(0.0) ;
            concs.push(val);
        }

        Self {
            dims: size,
            voxel_count,
            lipid_species: species_count,
            concs,
            temps,
            is_micelle,
        }
    }

    fn voxel_index(&self, pos: AnyVec3<usize>) -> Option<usize> {
        if pos.x >= self.dims.x || pos.y >= self.dims.y || pos.z >= self.dims.z {
            None
        } else {
            Some(pos.x + self.dims.x * (pos.y + self.dims.y * pos.z))
        }
    }

    pub fn get_lipid_at(&self, pos: AnyVec3<usize>, species: usize) -> Option<f32> {
        if species >= self.lipid_species {
            return None;
        }
        self.voxel_index(pos)
            .and_then(|i| self.concs.get(i * self.lipid_species + species))
            .copied()
    }

    pub fn lipid_to_voxel_index(idx: usize, species: usize) -> Option<usize> {
        if idx < species {
            None
        } else {
            Some((idx - species) / species)
        }
    }

    pub fn diffuse(&self, dt: f32) -> Vec<f32> {
        let mut new_concs = self.concs.clone();

        let offsets: [AnyVec3<i32>; 6] = [
            AnyVec3::new(1, 0, 0),
            AnyVec3::new(-1, 0, 0),
            AnyVec3::new(0, 1, 0),
            AnyVec3::new(0, -1, 0),
            AnyVec3::new(0, 0, 1),
            AnyVec3::new(0, 0, -1),
        ];

        for species in 0..self.lipid_species {
            for z in 0..self.dims.z {
                for y in 0..self.dims.y {
                    for x in 0..self.dims.x {
                        let pos = AnyVec3::new(x, y, z);
                        let i = match self.voxel_index(pos) {
                            Some(i) => i,
                            None => continue,
                        };
                        let center = self.concs[i * self.lipid_species + species];

                        let mut laplacian = 0.0;
                        for offset in &offsets {
                            let iv_pos = match pos.try_into_vec3::<i32>() {
                                Ok(v) => v,
                                Err(_) => continue,
                            };
                            let neighbor_i32 = iv_pos + *offset;
                            if neighbor_i32
                                .cmplt(self.dims.try_into_vec3().unwrap())
                                .all()
                                && neighbor_i32.cmpge(AnyVec3::new(0, 0, 0)).all()
                            {
                                let neighbor_pos = neighbor_i32.as_uvec3();
                                if let Some(val) = self.get_lipid_at(neighbor_pos, species) {
                                    laplacian += val;
                                }
                            }
                        }

                        laplacian -= 6.0 * center;
                        let temp = self.temps[i];
                        let diffusion_coeff = 0.05 * temp;
                        new_concs[i * self.lipid_species + species] +=
                            diffusion_coeff * laplacian * dt;
                    }
                }
            }
        }

        new_concs
    }
}
