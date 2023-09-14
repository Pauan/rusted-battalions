use crate::util::builders;
use crate::util::macros::wgsl;


struct Texture {
    texture: wgpu::Texture,
    texture_view: wgpu::TextureView,
    bind_group: wgpu::BindGroup,
}

impl Texture {
    fn new(bind_group_layout: &wgpu::BindGroupLayout, engine: &crate::EngineState) -> Self {
        let size = wgpu::Extent3d {
            width: engine.config.width,
            height: engine.config.height,
            depth_or_array_layers: 1,
        };

        let texture = engine.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Postprocess Buffer"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: engine.config.format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some("Postprocess Buffer View"),
            ..wgpu::TextureViewDescriptor::default()
        });

        let texture_sampler = engine.device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Postprocess Texture Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let depth_sampler = engine.device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Postprocess Depth Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            compare: Some(wgpu::CompareFunction::Always),
            ..Default::default()
        });

        let bind_group = builders::BindGroup::builder()
            .label("Postprocess")
            .layout(bind_group_layout)
            .sampler(&texture_sampler)
            .sampler(&depth_sampler)
            .texture_view(&texture_view)
            .texture_view(&engine.depth_buffer.depth_view)
            //.texture_view(&engine.depth_buffer.stencil_view)
            .build(engine);

        Self { texture, texture_view, bind_group }
    }
}

impl Drop for Texture {
    fn drop(&mut self) {
        self.texture.destroy();
    }
}


pub struct Postprocess {
    texture: Texture,
    bind_group_layout: wgpu::BindGroupLayout,
    render_pipeline: wgpu::RenderPipeline,
}

#[allow(unused)]
impl Postprocess {
    pub(crate) fn new(engine: &crate::EngineState) -> Self {
        let bind_group_layout = builders::BindGroupLayout::builder()
            .label("Postprocess")
            .sampler(wgpu::ShaderStages::FRAGMENT, wgpu::SamplerBindingType::NonFiltering)
            .sampler(wgpu::ShaderStages::FRAGMENT, wgpu::SamplerBindingType::Comparison)
            .texture(wgpu::ShaderStages::FRAGMENT, wgpu::TextureSampleType::Float { filterable: false })
            .texture(wgpu::ShaderStages::FRAGMENT, wgpu::TextureSampleType::Depth)
            //.texture(wgpu::ShaderStages::FRAGMENT, wgpu::TextureSampleType::Uint)
            .build(engine);

        let render_pipeline = builders::Pipeline::builder()
            .label("Postprocess")
            // TODO lazy load this ?
            .shader(wgsl![
                "postprocess.wgsl",
                include_str!("postprocess.wgsl"),
            ])
            .bind_groups(&[&bind_group_layout])
            .topology(wgpu::PrimitiveTopology::TriangleStrip)
            .strip_index_format(wgpu::IndexFormat::Uint32)
            .build(engine);

        Self {
            texture: Texture::new(&bind_group_layout, engine),
            bind_group_layout,
            render_pipeline,
        }
    }

    pub(crate) fn view(&self) -> &wgpu::TextureView {
        &self.texture.texture_view
    }

    pub(crate) fn resize(&mut self, engine: &crate::EngineState) {
        self.texture = Texture::new(&self.bind_group_layout, engine);
    }

    pub(crate) fn render<'a, 'b>(&'a mut self, render_pass: &mut wgpu::RenderPass<'b>) where 'a: 'b {
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, &self.texture.bind_group, &[]);
        render_pass.draw(0..4, 0..1);
    }
}
