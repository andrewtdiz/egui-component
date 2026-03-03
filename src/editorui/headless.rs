use std::fs;
use std::path::Path;
use std::sync::mpsc;

use crate::editorui::component_run_options::ComponentRunOptions;
use crate::editorui::style::setup_editor_context;
use crate::editorui::views::SingleComponentSurface;
use crate::{ClayError, Result};
use image::ColorType;

pub(crate) fn run_single_component_headless(options: ComponentRunOptions) -> Result {
    let width = options.width as u32;
    let height = options.height as u32;
    let max_frames = options.max_frames.unwrap_or(2).max(1);
    let verify_png_path = options
        .verify_png_path
        .clone()
        .ok_or_else(|| ClayError::PlatformError("--verify-png is required".to_owned()))?;

    if let Some(parent) = verify_png_path.parent() {
        fs::create_dir_all(parent).map_err(|error| {
            ClayError::PlatformError(format!(
                "failed to create png output directory {}: {error}",
                parent.display()
            ))
        })?;
    }

    let mut surface = SingleComponentSurface::from_options(options);
    let image_data = render_surface_frames(&mut surface, width, height, max_frames)?;

    image::save_buffer(
        verify_png_path.as_path(),
        image_data.as_slice(),
        width,
        height,
        ColorType::Rgba8,
    )
    .map_err(|error| {
        ClayError::PlatformError(format!(
            "failed to write png to {}: {error}",
            verify_png_path.display()
        ))
    })?;

    println!("{}", verify_png_path.display());
    Ok(())
}

fn render_surface_frames(
    surface: &mut SingleComponentSurface,
    width: u32,
    height: u32,
    frames: u64,
) -> Result<Vec<u8>> {
    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::default());

    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::LowPower,
        force_fallback_adapter: true,
        compatible_surface: None,
    }))
    .or_else(|_| {
        pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            force_fallback_adapter: false,
            compatible_surface: None,
        }))
    })
    .map_err(|error| ClayError::PlatformError(error.to_string()))?;

    let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        label: Some("egui-component-headless-device"),
        required_features: wgpu::Features::empty(),
        required_limits: wgpu::Limits::default(),
        experimental_features: wgpu::ExperimentalFeatures::disabled(),
        memory_hints: wgpu::MemoryHints::Performance,
        trace: wgpu::Trace::Off,
    }))
    .map_err(|error| ClayError::PlatformError(error.to_string()))?;

    let target_format = wgpu::TextureFormat::Rgba8UnormSrgb;
    let render_texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("egui-component-headless-render-target"),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: target_format,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let render_view = render_texture.create_view(&wgpu::TextureViewDescriptor::default());

    let egui_context = egui::Context::default();
    setup_editor_context(&egui_context);

    let mut renderer =
        egui_wgpu::Renderer::new(&device, target_format, egui_wgpu::RendererOptions::default());

    for frame_index in 0..frames {
        let raw_input = make_raw_input(width, height, frame_index as f64 / 60.0);
        let output = egui_context.run(raw_input, |context| {
            surface.draw(context);
        });

        for (texture_id, image_delta) in &output.textures_delta.set {
            renderer.update_texture(&device, &queue, *texture_id, image_delta);
        }

        let pixels_per_point = output.pixels_per_point.max(f32::EPSILON);
        let paint_jobs = egui_context.tessellate(output.shapes, pixels_per_point);
        let screen_descriptor = egui_wgpu::ScreenDescriptor {
            size_in_pixels: [width, height],
            pixels_per_point,
        };

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("egui-component-headless-frame-encoder"),
        });

        let mut submission_buffers =
            renderer.update_buffers(&device, &queue, &mut encoder, &paint_jobs, &screen_descriptor);

        {
            let render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("egui-component-headless-egui-pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &render_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            renderer.render(
                &mut render_pass.forget_lifetime(),
                &paint_jobs,
                &screen_descriptor,
            );
        }

        submission_buffers.push(encoder.finish());
        queue.submit(submission_buffers);

        for texture_id in &output.textures_delta.free {
            renderer.free_texture(texture_id);
        }
    }

    read_texture_rgba(
        &device,
        &queue,
        &render_texture,
        width,
        height,
        Path::new("egui-component-headless"),
    )
}

fn make_raw_input(width: u32, height: u32, time_seconds: f64) -> egui::RawInput {
    let mut raw_input = egui::RawInput::default();
    raw_input.screen_rect = Some(egui::Rect::from_min_size(
        egui::Pos2::ZERO,
        egui::vec2(width as f32, height as f32),
    ));
    raw_input.time = Some(time_seconds);
    if let Some(viewport_info) = raw_input.viewports.get_mut(&egui::ViewportId::ROOT) {
        viewport_info.native_pixels_per_point = Some(1.0);
    }
    raw_input
}

fn read_texture_rgba(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    texture: &wgpu::Texture,
    width: u32,
    height: u32,
    label: &Path,
) -> Result<Vec<u8>> {
    let stride = 4usize;
    let bytes_per_row_unpadded = width as usize * stride;
    let alignment = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT as usize;
    let bytes_per_row_padded = ((bytes_per_row_unpadded + alignment - 1) / alignment) * alignment;
    let output_buffer_size = (bytes_per_row_padded * height as usize) as u64;

    let output_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some(label.to_string_lossy().as_ref()),
        size: output_buffer_size,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });

    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("egui-component-headless-readback-encoder"),
    });

    encoder.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo {
            texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::TexelCopyBufferInfo {
            buffer: &output_buffer,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(bytes_per_row_padded as u32),
                rows_per_image: Some(height),
            },
        },
        wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
    );

    queue.submit(Some(encoder.finish()));

    let slice = output_buffer.slice(..);
    let (tx, rx) = mpsc::channel();
    slice.map_async(wgpu::MapMode::Read, move |result| {
        let _ = tx.send(result);
    });

    let _ = device.poll(wgpu::PollType::wait_indefinitely());

    match rx.recv() {
        Ok(Ok(())) => {
            let mapped = slice.get_mapped_range();
            let mut output = vec![0u8; width as usize * height as usize * stride];

            for row in 0..height as usize {
                let src_start = row * bytes_per_row_padded;
                let src_end = src_start + bytes_per_row_unpadded;
                let dst_start = row * bytes_per_row_unpadded;
                let dst_end = dst_start + bytes_per_row_unpadded;
                output[dst_start..dst_end].copy_from_slice(&mapped[src_start..src_end]);
            }

            drop(mapped);
            output_buffer.unmap();
            Ok(output)
        }
        Ok(Err(error)) => Err(ClayError::PlatformError(error.to_string())),
        Err(error) => Err(ClayError::PlatformError(error.to_string())),
    }
}
