//! A triangle example

use std::{borrow::Cow, sync::Arc};

use winpls::{AppHandler, Config, GraphicsBackend, get_graphics_backend, start};

const SHADER: &str = r#"
struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) color: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>
};

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.color = in.color;
    out.position = vec4<f32>(in.position, 0.0, 1.0);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.color;
}
"#;

const VERTICIES: &[Vertex] = &[
    Vertex::new([0.0, 0.6], [1.0, 0.0, 0.0, 1.0]),
    Vertex::new([0.5, -0.5], [0.0, 1.0, 0.0, 1.0]),
    Vertex::new([-0.5, -0.5], [0.0, 0.0, 1.0, 1.0]),
];

const INDICES: &[u32] = &[0, 1, 2];

#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
struct Vertex {
    position: [f32; 2],
    color: [f32; 4],
}

impl Vertex {
    const fn new(position: [f32; 2], color: [f32; 4]) -> Self {
        Self { position, color }
    }

    fn vertex_attributes<'a>() -> &'a [wgpu::VertexAttribute] {
        &[
            // Position
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x2,
                offset: 0,
                shader_location: 0,
            },
            // Color
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x4,
                offset: size_of::<[f32; 2]>() as u64,
                shader_location: 1,
            },
        ]
    }

    fn vertex_stride() -> u64 {
        size_of::<Vertex>() as u64
    }

    fn vertex_step_mode() -> wgpu::VertexStepMode {
        wgpu::VertexStepMode::Vertex
    }
}

struct App {
    backend: Arc<GraphicsBackend>,
    pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
}

impl App {
    fn new() -> Self {
        let backend = get_graphics_backend();

        let surface_format = backend.get_surface_format();

        let pipeline_layout = backend.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[],
            immediate_size: 0,
        });

        let shader = backend.create_shader(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(SHADER)),
        });

        let pipeline = backend.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: Vertex::vertex_stride(),
                    step_mode: Vertex::vertex_step_mode(),
                    attributes: Vertex::vertex_attributes(),
                }],
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview_mask: None,
            cache: None,
        });

        let vertex_buffer = backend.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(VERTICIES),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::UNIFORM,
        });

        let index_buffer = backend.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });

        Self {
            backend,
            pipeline,
            vertex_buffer,
            index_buffer,
        }
    }
}

impl AppHandler for App {
    fn draw(&mut self) {
        let view = match self.backend.get_surface_view() {
            Some(view) => view,
            None => return,
        };

        let mut encoder = self
            .backend
            .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });

            rpass.set_pipeline(&self.pipeline);
            rpass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            rpass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            rpass.draw_indexed(0..(VERTICIES.len() as u32), 0, 0..1);
        }

        self.backend.submit_command_buffers([encoder.finish()]);
    }

    fn app_event(&mut self, _: winpls::AppEvent) {}

    fn quitting(&mut self) {}
}

fn main() {
    let conf = Config::default();
    start(|| Box::new(App::new()), conf);
}
