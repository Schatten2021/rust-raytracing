pub trait GpuSerialize {
    fn serialize(&self) -> Vec<u8>;
}

impl GpuSerialize for f64 {
    fn serialize(&self) -> Vec<u8> {
        ((*self) as f32).to_le_bytes().to_vec()
    }
}