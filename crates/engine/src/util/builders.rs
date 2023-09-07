use wgpu;


pub(crate) struct BindGroup<'a, 'b> {
    label: Option<&'static str>,
    layout: Option<&'a wgpu::BindGroupLayout>,
    entries: Vec<wgpu::BindGroupEntry<'b>>,
}

impl<'a, 'b> BindGroup<'a, 'b> {
    pub(crate) fn builder() -> Self {
        Self {
            label: None,
            layout: None,
            entries: vec![],
        }
    }

    #[inline]
    pub(crate) fn label(mut self, label: &'static str) -> Self {
        self.label = Some(label);
        self
    }

    #[inline]
    pub(crate) fn layout(mut self, layout: &'a wgpu::BindGroupLayout) -> Self {
        self.layout = Some(layout);
        self
    }

    #[inline]
    pub(crate) fn texture_view(mut self, texture_view: &'b wgpu::TextureView) -> Self {
        let binding = self.entries.len() as u32;

        self.entries.push(wgpu::BindGroupEntry {
            binding,
            resource: wgpu::BindingResource::TextureView(texture_view),
        });

        self
    }

    #[inline]
    pub(crate) fn sampler(mut self, sampler: &'b wgpu::Sampler) -> Self {
        let binding = self.entries.len() as u32;

        self.entries.push(wgpu::BindGroupEntry {
            binding,
            resource: wgpu::BindingResource::Sampler(sampler),
        });

        self
    }

    #[inline]
    pub(crate) fn build(self, engine: &crate::EngineState) -> wgpu::BindGroup {
        let layout = self.layout.expect("BindGroup: missing layout");

        engine.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: self.label.map(|label| format!("{} Bind Group", label)).as_deref(),
            layout,
            entries: &self.entries,
        })
    }
}


pub(crate) struct BindGroupLayout {
    label: Option<&'static str>,
    entries: Vec<wgpu::BindGroupLayoutEntry>,
}

impl BindGroupLayout {
    pub(crate) fn builder() -> Self {
        Self {
            label: None,
            entries: vec![],
        }
    }

    #[inline]
    pub(crate) fn label(mut self, label: &'static str) -> Self {
        self.label = Some(label);
        self
    }

    #[inline]
    pub(crate) fn texture(mut self, visibility: wgpu::ShaderStages, ty: wgpu::TextureSampleType) -> Self {
        let binding = self.entries.len() as u32;

        self.entries.push(wgpu::BindGroupLayoutEntry {
            binding,
            visibility,
            ty: wgpu::BindingType::Texture {
                sample_type: ty,
                view_dimension: wgpu::TextureViewDimension::D2,
                multisampled: false,
            },
            count: None,
        });

        self
    }

    #[inline]
    pub(crate) fn sampler(mut self, visibility: wgpu::ShaderStages, ty: wgpu::SamplerBindingType) -> Self {
        let binding = self.entries.len() as u32;

        self.entries.push(wgpu::BindGroupLayoutEntry {
            binding,
            visibility: visibility,
            ty: wgpu::BindingType::Sampler(ty),
            count: None,
        });

        self
    }

    pub(crate) fn build(self, engine: &crate::EngineState) -> wgpu::BindGroupLayout {
        engine.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: self.label.map(|label| format!("{} Bind Group Layout", label)).as_deref(),
            entries: &self.entries,
        })
    }
}


pub(crate) struct Pipeline<'a, 'b, 'c> {
    label: Option<&'static str>,
    bind_groups: Option<&'a [&'a wgpu::BindGroupLayout]>,
    shader: Option<wgpu::ShaderModuleDescriptor<'c>>,
    vertex_buffers: Option<&'b [wgpu::VertexBufferLayout<'b>]>,
    topology: Option<wgpu::PrimitiveTopology>,
    strip_index_format: Option<wgpu::IndexFormat>,
    front_face: Option<wgpu::FrontFace>,
    cull_mode: Option<wgpu::Face>,
    depth_stencil: Option<wgpu::StencilState>,
}

#[allow(unused)]
impl<'a, 'b, 'c> Pipeline<'a, 'b, 'c> {
    pub(crate) fn builder() -> Self {
        Self {
            label: None,
            bind_groups: None,
            shader: None,
            vertex_buffers: None,
            topology: None,
            strip_index_format: None,
            front_face: None,
            cull_mode: None,
            depth_stencil: None,
        }
    }

    #[inline]
    pub(crate) fn label(mut self, label: &'static str) -> Self {
        self.label = Some(label);
        self
    }

    #[inline]
    pub(crate) fn bind_groups(mut self, bind_groups: &'a [&'a wgpu::BindGroupLayout]) -> Self {
        self.bind_groups = Some(bind_groups);
        self
    }

    #[inline]
    pub(crate) fn shader(mut self, shader: wgpu::ShaderModuleDescriptor<'c>) -> Self {
        self.shader = Some(shader);
        self
    }

    #[inline]
    pub(crate) fn vertex_buffers(mut self, vertex_buffers: &'b [wgpu::VertexBufferLayout<'b>]) -> Self {
        self.vertex_buffers = Some(vertex_buffers);
        self
    }

    #[inline]
    pub(crate) fn topology(mut self, topology: wgpu::PrimitiveTopology) -> Self {
        self.topology = Some(topology);
        self
    }

    #[inline]
    pub(crate) fn strip_index_format(mut self, strip_index_format: wgpu::IndexFormat) -> Self {
        self.strip_index_format = Some(strip_index_format);
        self
    }

    #[inline]
    pub(crate) fn front_face(mut self, front_face: wgpu::FrontFace) -> Self {
        self.front_face = Some(front_face);
        self
    }

    #[inline]
    pub(crate) fn cull_mode(mut self, cull_mode: wgpu::Face) -> Self {
        self.cull_mode = Some(cull_mode);
        self
    }

    #[inline]
    pub(crate) fn depth_stencil(mut self, depth_stencil: wgpu::StencilState) -> Self {
        self.depth_stencil = Some(depth_stencil);
        self
    }

    pub(crate) fn build(self, engine: &crate::EngineState) -> wgpu::RenderPipeline {
        let shader = engine.device.create_shader_module(self.shader.expect("Pipeline: missing shader"));

        let render_pipeline_layout = engine.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: self.label.map(|label| format!("{} Pipeline Layout", label)).as_deref(),
            bind_group_layouts: self.bind_groups.unwrap_or(&[]),
            push_constant_ranges: &[],
        });

        engine.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: self.label.map(|label| format!("{} Pipeline", label)).as_deref(),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: self.vertex_buffers.unwrap_or(&[]),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: engine.config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: self.topology.unwrap_or(wgpu::PrimitiveTopology::TriangleList),
                strip_index_format: self.strip_index_format,
                front_face: self.front_face.unwrap_or(wgpu::FrontFace::Ccw),
                cull_mode: self.cull_mode,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: self.depth_stencil.map(|stencil| engine.depth_stencil_state(stencil)),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        })
    }
}
