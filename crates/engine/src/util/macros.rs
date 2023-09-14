macro_rules! wgsl {
    ($label:literal, $path:expr$(,)?) => {
        wgpu::ShaderModuleDescriptor {
            label: Some($label),
            source: wgpu::ShaderSource::Wgsl($path.into()),
        }
    };
    ($label:literal, $($path:expr),+$(,)?) => {
        wgpu::ShaderModuleDescriptor {
            label: Some($label),
            source: wgpu::ShaderSource::Wgsl([
                $($path),+
            ].join("\n").into()),
        }
    };
}

pub(crate) use wgsl;
