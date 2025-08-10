/// For creation of buffers, including:
/// Voxel grid buffer: T = stored value; usize = dimension length
pub enum Access {
    ReadOnly,
    ReadWrite,
}

pub enum OffsetBehaviour {
    Static,
    Dynamic
}
