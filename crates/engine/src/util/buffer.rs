use wgpu;
use wgpu::util::DeviceExt;
use image;
use std::ops::{Deref, DerefMut};
use std::marker::PhantomData;


pub trait IntoTexture {
    type Item;

    fn label(&self) -> &'static str;

    fn dimensions(&self) -> (u32, u32);

    fn format(&self) -> wgpu::TextureFormat;

    fn bytes(&self) -> &[u8];
}


pub(crate) struct TextureBuffer {
    pub(crate) texture: wgpu::Texture,
    pub(crate) view: wgpu::TextureView,
}

impl TextureBuffer {
    pub(crate) fn new<T>(engine: &crate::EngineState, image: &T) -> Self where T: IntoTexture {
        let label = image.label();

        let (width, height) = image.dimensions();

        let size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };

        let texture = engine.device.create_texture(&wgpu::TextureDescriptor {
            label: Some(&label),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: image.format(),
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        engine.queue.write_texture(
            wgpu::ImageCopyTexture {
                aspect: wgpu::TextureAspect::All,
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            image.bytes(),
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(width * (std::mem::size_of::<T::Item>() as u32)),
                rows_per_image: Some(height),
            },
            size,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some(&label),
            format: None,
            dimension: None,
            aspect: wgpu::TextureAspect::All,
            base_mip_level: 0,
            mip_level_count: None,
            base_array_layer: 0,
            array_layer_count: None,
        });

        Self { texture, view }
    }
}

impl Drop for TextureBuffer {
    fn drop(&mut self) {
        self.texture.destroy();
    }
}


pub struct RgbaImage {
    label: &'static str,
    pub image: image::RgbaImage,
}

impl RgbaImage {
    pub(crate) const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8UnormSrgb;

    pub fn from_fn<F>(label: &'static str, width: u32, height: u32, f: F) -> Self
        where F: FnMut(u32, u32) -> image::Rgba<u8> {

        let image = image::RgbaImage::from_fn(width, height, f);

        Self { label, image }
    }

    pub fn from_bytes(label: &'static str, bytes: &[u8]) -> Self {
        let image = image::load_from_memory(bytes).unwrap().into_rgba8();

        Self { label, image }
    }
}

impl IntoTexture for RgbaImage {
    type Item = image::Rgba<u8>;

    #[inline]
    fn label(&self) -> &'static str {
        &self.label
    }

    #[inline]
    fn format(&self) -> wgpu::TextureFormat {
        Self::FORMAT
    }

    #[inline]
    fn dimensions(&self) -> (u32, u32) {
        self.image.dimensions()
    }

    #[inline]
    fn bytes(&self) -> &[u8] {
        &self.image
    }
}


pub struct IndexedImage {
    label: &'static str,
    pub image: image::GrayAlphaImage,
}

impl IndexedImage {
    pub(crate) const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rg8Uint;

    pub fn from_fn<F>(label: &'static str, width: u32, height: u32, f: F) -> Self
        where F: FnMut(u32, u32) -> image::LumaA<u8> {

        let image = image::GrayAlphaImage::from_fn(width, height, f);

        Self { label, image }
    }

    pub fn from_bytes(label: &'static str, bytes: &[u8]) -> Self {
        let image = image::load_from_memory(bytes).unwrap().into_luma_alpha8();

        Self { label, image }
    }
}

impl IntoTexture for IndexedImage {
    type Item = image::LumaA<u8>;

    #[inline]
    fn label(&self) -> &'static str {
        &self.label
    }

    #[inline]
    fn format(&self) -> wgpu::TextureFormat {
        Self::FORMAT
    }

    #[inline]
    fn dimensions(&self) -> (u32, u32) {
        self.image.dimensions()
    }

    #[inline]
    fn bytes(&self) -> &[u8] {
        &self.image
    }
}


pub struct GrayscaleImage {
    label: &'static str,
    pub image: image::GrayImage,
}

impl GrayscaleImage {
    pub(crate) const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::R8Uint;

    pub fn from_fn<F>(label: &'static str, width: u32, height: u32, f: F) -> Self
        where F: FnMut(u32, u32) -> image::Luma<u8> {

        let image = image::GrayImage::from_fn(width, height, f);

        Self { label, image }
    }

    pub fn from_bytes(label: &'static str, bytes: &[u8]) -> Self {
        let image = image::load_from_memory(bytes).unwrap().into_luma8();

        Self { label, image }
    }
}

impl IntoTexture for GrayscaleImage {
    type Item = image::Luma<u8>;

    #[inline]
    fn label(&self) -> &'static str {
        &self.label
    }

    #[inline]
    fn format(&self) -> wgpu::TextureFormat {
        Self::FORMAT
    }

    #[inline]
    fn dimensions(&self) -> (u32, u32) {
        self.image.dimensions()
    }

    #[inline]
    fn bytes(&self) -> &[u8] {
        &self.image
    }
}


pub(crate) struct Uniform<T> {
    bind_group_layout: Option<wgpu::BindGroupLayout>,
    bind_group: Option<wgpu::BindGroup>,
    buffer: Option<wgpu::Buffer>,
    visibility: wgpu::ShaderStages,
    changed: bool,
    value: T,
}

#[allow(unused)]
impl<T> Uniform<T> where T: bytemuck::Pod {
    pub(crate) fn new(visibility: wgpu::ShaderStages, value: T) -> Self {
        Self {
            bind_group_layout: None,
            bind_group: None,
            buffer: None,
            visibility,
            changed: true,
            value,
        }
    }

    fn get_bind_group_layout<'a>(
        this: &'a mut Option<wgpu::BindGroupLayout>,
        visibility: wgpu::ShaderStages,
        engine: &crate::EngineState
    ) -> &'a wgpu::BindGroupLayout {

        this.get_or_insert_with(|| {
            engine.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    }
                ],
                label: Some("Uniform bind group layout"),
            })
        })
    }

    #[inline]
    pub(crate) fn bind_group_layout<'a>(this: &'a mut Self, engine: &crate::EngineState) -> &'a wgpu::BindGroupLayout {
        Self::get_bind_group_layout(&mut this.bind_group_layout, this.visibility, engine)
    }

    fn init<'a>(this: &'a mut Self, engine: &crate::EngineState) -> &'a wgpu::BindGroup {
        this.bind_group.get_or_insert_with(|| {
            let bind_group_layout = Self::get_bind_group_layout(&mut this.bind_group_layout, this.visibility, engine);

            let buffer = this.buffer.get_or_insert_with(|| {
                engine.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Uniform buffer"),
                    contents: bytemuck::cast_slice(&[this.value]),
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                })
            });

            engine.device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: buffer.as_entire_binding(),
                    }
                ],
                label: Some("Uniform bind group"),
            })
        })
    }

    pub(crate) fn write<'a>(this: &'a mut Self, engine: &crate::EngineState) -> &'a wgpu::BindGroup {
        if this.changed {
            this.changed = false;

            if let Some(buffer) = &this.buffer {
                // TODO use StagingBelt
                engine.queue.write_buffer(&buffer, 0, bytemuck::cast_slice(&[this.value]));
            }
        }

        Self::init(this, engine)
    }
}

impl<T> Deref for Uniform<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<T> DerefMut for Uniform<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.changed = true;
        &mut self.value
    }
}

impl<T> Drop for Uniform<T> {
    fn drop(&mut self) {
        if let Some(buffer) = &self.buffer {
            buffer.destroy();
        }
    }
}


pub(crate) struct VecBufferSettings<'a> {
    pub(crate) label: Option<&'a str>,
    pub(crate) usage: wgpu::BufferUsages,
}


/// Utility for writing a `Vec<T>` into a `wgpu::Buffer`.
///
/// It will automatically resize the buffer to match the Vec's capacity.
#[repr(transparent)]
pub(crate) struct VecBuffer<T> {
    buffer: Option<wgpu::Buffer>,
    _phantom: PhantomData<Vec<T>>,
}

impl<T> VecBuffer<T> where T: bytemuck::Pod  {
    pub(crate) fn new() -> Self {
        Self {
            buffer: None,
            _phantom: PhantomData,
        }
    }

    fn byte_capacity(values: &Vec<T>) -> u64 {
        (values.capacity() * std::mem::size_of::<T>()) as u64
    }

    fn byte_len(values: &Vec<T>) -> u64 {
        (values.len() * std::mem::size_of::<T>()) as u64
    }

    /// This should only be called if vec_capacity > 0
    fn make_buffer<'a>(vec_capacity: u64, values: &Vec<T>, engine: &crate::EngineState, settings: VecBufferSettings<'a>) -> wgpu::Buffer {
        let vec_len = Self::byte_len(values);

        assert!(vec_capacity >= vec_len);

        let buffer = engine.device.create_buffer(&wgpu::BufferDescriptor {
            label: settings.label,
            size: vec_capacity,
            usage: settings.usage | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: true,
        });

        if vec_len > 0 {
            buffer.slice(..vec_len)
                .get_mapped_range_mut()
                .copy_from_slice(bytemuck::cast_slice(values.as_slice()));
        }

        buffer.unmap();

        buffer
    }

    fn to_slice(&self, values: &Vec<T>) -> Option<wgpu::BufferSlice<'_>> {
        let vec_len = Self::byte_len(values);

        if vec_len == 0 {
            None

        } else {
            self.buffer.as_ref().map(|buffer| buffer.slice(..vec_len))
        }
    }

    pub(crate) fn write<'a>(&mut self, values: &Vec<T>, engine: &crate::EngineState, settings: VecBufferSettings<'a>) -> Option<wgpu::BufferSlice<'_>> {
        let vec_capacity = Self::byte_capacity(values);

        if let Some(buffer) = &self.buffer {
            let buffer_size = buffer.size();

            if buffer_size == vec_capacity {
                // TODO use StagingBelt
                engine.queue.write_buffer(buffer, 0, bytemuck::cast_slice(values.as_slice()));

            } else {
                buffer.destroy();

                if vec_capacity == 0 {
                    self.buffer = None;

                } else {
                    self.buffer = Some(Self::make_buffer(vec_capacity, values, engine, settings));
                }
            }

        } else if vec_capacity != 0 {
            self.buffer = Some(Self::make_buffer(vec_capacity, values, engine, settings));
        }

        self.to_slice(values)
    }
}

impl<T> Drop for VecBuffer<T> {
    fn drop(&mut self) {
        if let Some(buffer) = &self.buffer {
            buffer.destroy();
        }
    }
}


pub struct InstanceVecOptions<'a> {
    pub label: Option<&'a str>,
}

/// Similar to a [`Vec<T>`] except it can be used as a [`wgpu::Buffer`] for instanced data.
///
/// It will automatically resize the [`wgpu::Buffer`] as needed, and will automatically
/// copy the data into the [`wgpu::Buffer`].
///
/// This makes it much easier to pass instanced data to the shader.
pub struct InstanceVec<T> {
    values: Vec<T>,
    buffer: VecBuffer<T>,
    changed: bool,
}

#[allow(unused)]
impl<T> InstanceVec<T> where T: bytemuck::Pod {
    #[inline]
    pub fn new() -> Self {
        Self::with_values(vec![])
    }

    #[inline]
    pub fn with_values(values: Vec<T>) -> Self {
        Self {
            changed: values.capacity() > 0,
            buffer: VecBuffer::new(),
            values,
        }
    }

    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self::with_values(Vec::with_capacity(capacity))
    }

    pub(crate) fn update_buffer(&mut self, engine: &crate::EngineState, options: &InstanceVecOptions) -> Option<wgpu::BufferSlice<'_>> {
        if self.changed {
            self.changed = false;

            self.buffer.write(&self.values, engine, VecBufferSettings {
                label: options.label,
                usage: wgpu::BufferUsages::VERTEX,
            })

        } else {
            self.buffer.to_slice(&self.values)
        }
    }

    pub fn resize_with<F>(&mut self, new_len: usize, create: F) where F: FnMut() -> T {
        let old_len = self.values.len();

        if old_len != new_len {
            self.changed = true;
            self.values.resize_with(new_len, create);
        }
    }
}

impl<T> Deref for InstanceVec<T> {
    type Target = Vec<T>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.values
    }
}

impl<T> DerefMut for InstanceVec<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.changed = true;
        &mut self.values
    }
}

impl<T> std::fmt::Debug for InstanceVec<T> where T: std::fmt::Debug {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        self.values.fmt(f)
    }
}
