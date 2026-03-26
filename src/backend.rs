use std::{
    fmt::Debug,
    sync::{Arc, Mutex},
};

use wgpu::util::{DeviceExt, TextureFormatExt};
use winit::window::Window;

/// A wgpu graphics backend
pub struct GraphicsBackend {
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface: wgpu::Surface<'static>,
    texture: Mutex<Option<wgpu::SurfaceTexture>>,
    config: Mutex<wgpu::SurfaceConfiguration>,
}

impl Debug for GraphicsBackend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<GraphicsBackend>")
    }
}

impl GraphicsBackend {
    pub(crate) fn new(window: Arc<Window>) -> Self {
        let window_size = window.inner_size();

        let instance = wgpu::Instance::default();

        let surface = instance.create_surface(Arc::new(window)).unwrap();

        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::None,
            force_fallback_adapter: false,
            compatible_surface: Some(&surface),
        })).unwrap();

        let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
            label: None,
            required_features: wgpu::Features::BGRA8UNORM_STORAGE,
            required_limits: wgpu::Limits::downlevel_defaults(),
            experimental_features: wgpu::ExperimentalFeatures::disabled(),
            memory_hints: wgpu::MemoryHints::Performance,
            trace: wgpu::Trace::Off,
        })).unwrap();

        let capabilities = surface.get_capabilities(&adapter);

        // TODO: Better format selection
        let swapchain_format = capabilities.formats.iter()
            .copied()
            .find(|format| format.to_storage_format().is_some())
            .unwrap_or( capabilities.formats[0]);

        let alpha_mode = capabilities.alpha_modes[0];

        let config = wgpu::SurfaceConfiguration {
            usage: capabilities.usages,
            format: swapchain_format,
            width: window_size.width,
            height: window_size.height,
            present_mode: wgpu::PresentMode::AutoVsync,
            alpha_mode: alpha_mode,
            view_formats: vec![swapchain_format],
            desired_maximum_frame_latency: 2
        };

        surface.configure(&device, &config);

        Self {
            device,
            queue,
            surface,
            config: Mutex::new(config),
            texture: Mutex::new(None),
        }
    }

    /// Create a bind group layout
    pub fn create_bind_group_layout(
        &self,
        desc: &wgpu::BindGroupLayoutDescriptor,
    ) -> wgpu::BindGroupLayout {
        self.device.create_bind_group_layout(desc)
    }

    /// Create a bind group
    pub fn create_bind_group(&self, desc: &wgpu::BindGroupDescriptor) -> wgpu::BindGroup {
        self.device.create_bind_group(desc)
    }

    /// Create a texture
    pub fn create_texture(&self, desc: &wgpu::TextureDescriptor) -> wgpu::Texture {
        self.device.create_texture(desc)
    }

    /// Create a texture with provided data
    pub fn create_texture_init(
        &self, 
        desc: &wgpu::TextureDescriptor, 
        order: wgpu::wgt::TextureDataOrder,
        data: &[u8] 
    ) -> wgpu::Texture {
        self.device.create_texture_with_data(&self.queue, desc, order, data)
    }

    /// Create a buffer
    pub fn create_buffer(&self, desc: &wgpu::BufferDescriptor) -> wgpu::Buffer {
        self.device.create_buffer(desc)
    }

    /// Create a buffer and fill it up with data immediately
    pub fn create_buffer_init(&self, desc: &wgpu::util::BufferInitDescriptor) -> wgpu::Buffer {
        self.device.create_buffer_init(desc)
    }

    /// Create a shader
    pub fn create_shader(&self, desc: wgpu::ShaderModuleDescriptor) -> wgpu::ShaderModule {
        self.device.create_shader_module(desc)
    }

    /// Create a sampler
    pub fn create_sampler(&self, desc: &wgpu::SamplerDescriptor) -> wgpu::Sampler {
        self.device.create_sampler(desc)
    }

    /// Create a pipeline
    pub fn create_pipeline_layout(
        &self,
        desc: &wgpu::PipelineLayoutDescriptor,
    ) -> wgpu::PipelineLayout {
        self.device.create_pipeline_layout(desc)
    }

    /// Create a render pipeline
    pub fn create_render_pipeline(
        &self,
        desc: &wgpu::RenderPipelineDescriptor,
    ) -> wgpu::RenderPipeline {
        self.device.create_render_pipeline(desc)
    }

    /// Create a command encoder
    pub fn create_command_encoder(
        &self,
        desc: &wgpu::CommandEncoderDescriptor,
    ) -> wgpu::CommandEncoder {
        self.device.create_command_encoder(desc)
    }

    /// Submit command buffers to the queue
    pub fn submit_command_buffers<I>(&self, command_buffers: I) -> wgpu::SubmissionIndex
    where
        I: IntoIterator<Item = wgpu::CommandBuffer>,
    {
        self.queue.submit(command_buffers)
    }

    /// Write to a buffer
    pub fn write_buffer(&self, buffer: &wgpu::Buffer, offset: usize, data: &[u8]) {
        self.queue.write_buffer(buffer, offset as u64, data);
    }

    /// Write to a texture
    pub fn write_texture(
        &self,
        texture: &wgpu::Texture,
        data: &[u8],
        layout: wgpu::TexelCopyBufferLayout,
        size: (u32, u32),
    ) {
        self.queue.write_texture(
            texture.as_image_copy(),
            data,
            layout,
            wgpu::Extent3d {
                width: size.0,
                height: size.1,
                depth_or_array_layers: 1,
            },
        );
    }

    /// Get the screen's texture view (if present)
    ///
    /// # Panics
    /// This could panic if another thread is calling GraphicsBackend methods, which internally lock the underlying surface texture.
    /// Please don't.
    pub fn get_surface_view(&self) -> Option<wgpu::TextureView> {
        let format = self.get_surface_format();
        let texture = self.texture.lock().unwrap();

        texture.as_ref().map(|texture| {
            texture.texture
                .create_view(&wgpu::TextureViewDescriptor {
                    format: Some(format),
                    ..Default::default()
                })
        })
    }

    /// Get the surface format. Useful for initilization, even when no frames are available
    /// 
    /// # Panics
    /// For the same reason [GraphicsBackend::get_surface_view] will
    pub fn get_surface_format(&self) -> wgpu::TextureFormat {
        let config = self.config.lock().unwrap();
        
        config.format
    }

    /// Resize this backend's surface
    pub(crate) fn resize(&self, width: u32, height: u32) {
        // Remove our surface texture
        self.texture.lock().unwrap().take();

        let mut config = self.config.lock().unwrap();
        config.width = width;
        config.height = height;
        self.surface.configure(&self.device, &config);
    }

    /// Flip the screen (by presenting its contents and getting the next texture)
    pub(crate) fn flip(&self) {
        let mut texture = self.texture.lock().unwrap();
        let config = self.config.lock().unwrap();

        if let Some(texture) = texture.take() {
            texture.present();
        }

        // Get the next texture
        *texture = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(surface) => Some(surface),

            // We will reconfigure anyway thanks to window events
            wgpu::CurrentSurfaceTexture::Suboptimal(surface) => Some(surface),
            wgpu::CurrentSurfaceTexture::Outdated => {
                self.surface.configure(&self.device, &config);
                None
            }
            wgpu::CurrentSurfaceTexture::Lost => todo!("Device loss"),
            _ => None,
        };
    }

    /// Clear the screen with provided color values
    /// 
    /// Under the hood this will allocate a new command buffer and create an empty render pass.
    /// If you're overwriting the surface with something else anyway - you could avoid this operation entirely.
    pub fn clear_screen(&self, r: f32, g: f32, b: f32, a: f32) {
        let view = if let Some(view) = self.get_surface_view() {
            view
        } else {
            return;
        };

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

        {
            let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: r as f64,
                            g: g as f64,
                            b: b as f64,
                            a: a as f64,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
                multiview_mask: None,
            });
        }

        self.queue.submit([encoder.finish()]);
    }

    /// Set a device lost callback
    pub fn set_device_lost_callback<F>(&self, f: F)
    where
        F: Fn(wgpu::DeviceLostReason, String) + Send + Sync + 'static,
    {
        self.device.set_device_lost_callback(f);
    }
}
