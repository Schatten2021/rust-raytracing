pub trait GpuSerialize {
    fn serialize(&self) -> Vec<u8>;
    fn serialized_size(&self) -> usize {
        self.serialize().len()
    }
}

impl GpuSerialize for f64 {
    fn serialize(&self) -> Vec<u8> {
        ((*self) as f32).to_le_bytes().to_vec()
    }
}