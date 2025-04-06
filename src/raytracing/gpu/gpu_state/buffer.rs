use wgpu::util::DeviceExt;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum BufferChangeState {
    #[default]
    Unchanged,
    Changed,
    ChangedOnly,
    Appended,
}
impl BufferChangeState {
    pub const fn got_resized(&self) -> bool {
        match self {
            BufferChangeState::Unchanged => false,
            BufferChangeState::ChangedOnly => false,
            BufferChangeState::Changed => true,
            BufferChangeState::Appended => true,
        }
    }
}
pub(super) struct FrequentlyChangedBuffer<'a> {
    device: wgpu::Device,
    label: wgpu::Label<'a>,
    buffer: wgpu::Buffer,
    data: Vec<u8>,
    state: BufferChangeState,
}
macro_rules! const_bitflags {
    ($ty:ty, $($flags:path)|*) => {<$ty>::from_bits($($flags.bits())|*).unwrap()};
}
impl<'a> FrequentlyChangedBuffer<'a> {
    const BUFFER_USAGES: wgpu::BufferUsages = const_bitflags!(wgpu::BufferUsages, wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC);
    pub fn new(device: &wgpu::Device, label: wgpu::Label<'a>) -> Self {
        let buffer = Self::create_uninit_buffer(device, label, 0);
        Self {
            device: device.clone(),
            label,
            buffer,
            data: vec![],
            state: BufferChangeState::Unchanged,
        }
    }
    pub fn new_init(device: &wgpu::Device, label: wgpu::Label<'a>, data: Vec<u8>) -> Self {
        let buffer = Self::create_init_buffer(device, label, &data);
        Self {
            device: device.clone(),
            label,
            buffer,
            data,
            state: BufferChangeState::Unchanged,
        }
    }
    fn create_init_buffer(device: &wgpu::Device, label: wgpu::Label<'a>, data: &[u8]) -> wgpu::Buffer {
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label,
            usage: Self::BUFFER_USAGES,
            contents: data,
        })
    }
    fn create_uninit_buffer(device: &wgpu::Device, label: wgpu::Label<'a>, size: u64) -> wgpu::Buffer {
        device.create_buffer(&wgpu::BufferDescriptor {
            label,
            size,
            usage: Self::BUFFER_USAGES,
            mapped_at_creation: false,
        })
    }
    pub fn get_updated_buffer(&mut self, queue: &wgpu::Queue) -> &wgpu::Buffer {
        // update buffers if necessary with the new data
        match self.state {
            BufferChangeState::Unchanged => {}
            BufferChangeState::Changed => {
                self.buffer = Self::create_init_buffer(&self.device, self.label, &*self.data)
            }
            BufferChangeState::ChangedOnly => {
                queue.write_buffer(&self.buffer, 0, &self.data);
            }
            BufferChangeState::Appended => {
                let new_buffer = Self::create_uninit_buffer(&self.device, self.label, self.data.len() as u64);
                let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("raytracing temporary buffer copy command encoder") });
                encoder.copy_buffer_to_buffer(&self.buffer, 0, &new_buffer, 0, self.buffer.size());
                queue.submit([encoder.finish()]);
                queue.write_buffer(&new_buffer, self.buffer.size(), &self.data[self.buffer.size() as usize..]);
                self.buffer = new_buffer;
            }
        }
        self.state = BufferChangeState::Unchanged;
        &self.buffer
    }
    pub fn append(&mut self, data: impl IntoIterator<Item = u8>) -> () {
        self.data.extend(data);
        if self.state == BufferChangeState::Unchanged {
            self.state = BufferChangeState::Appended;
        }
    }
    pub fn set_data(&mut self, data: Vec<u8>) -> () {
        if self.data.len() == data.len() && !self.state.got_resized() {
            self.state = BufferChangeState::ChangedOnly;
        } else {
            self.state = BufferChangeState::Changed;
        }
        self.data = data;
    }
    pub fn change_data(&mut self, data: Vec<u8>, start_index: usize) {
        if data.len() + start_index > self.data.len() {
            self.data.resize(data.len() + start_index, 0);
            self.state = BufferChangeState::Changed;
        } else if !self.state.got_resized() {
            self.state = BufferChangeState::ChangedOnly;
        } else {
            self.state = BufferChangeState::Changed;
        }
        self.data[start_index..start_index + data.len()].copy_from_slice(data.as_slice());
    }
}