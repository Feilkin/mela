//! Loadable stuff

pub mod scene;
pub mod tilemap;

pub enum AssetState<T> {
    Loading(Box<dyn Asset<T>>),
    Done(T),
}

#[derive(Debug)]
pub enum AssetError {
    IoError(std::io::Error),
    ImageError(image::ImageError),
    SerdeXmlError(serde_xml_rs::Error),
    SerdeJsonError(serde_json::Error),
}

impl From<std::io::Error> for AssetError {
    fn from(err: Error) -> Self {
        AssetError::IoError(err)
    }
}

impl From<image::ImageError> for AssetError {
    fn from(err: image::ImageError) -> Self {
        AssetError::ImageError(err)
    }
}

impl From<serde_xml_rs::Error> for AssetError {
    fn from(err: serde_xml_rs::Error) -> Self {
        AssetError::SerdeXmlError(err)
    }
}

impl From<serde_json::Error> for AssetError {
    fn from(err: serde_json::Error) -> Self {
        AssetError::SerdeJsonError(err)
    }
}

pub trait Asset<T> {
    // Some assets, such as texture, require access to the device.
    fn poll(self: Box<Self>, render_ctx: &mut RenderContext) -> Result<AssetState<T>, AssetError>;
}

// Example Asset implementation
// TODO: move to crates when done.
use crate::gfx::{RenderContext, Texture};
use image::DynamicImage;
use std::fs::File;
use std::io::Error;
use std::path::Path;
use std::rc::Rc;

impl<T> Asset<Texture> for T
where
    T: AsRef<Path>,
{
    fn poll(
        self: Box<Self>,
        render_ctx: &mut RenderContext,
    ) -> Result<AssetState<Texture>, AssetError> {
        let img = image::open(self.as_ref())?.to_bgra();
        let img_dim = img.dimensions();

        let texture_extent = wgpu::Extent3d {
            width: img_dim.0,
            height: img_dim.1,
            depth: 1,
        };

        let texture = render_ctx.device.create_texture(&wgpu::TextureDescriptor {
            size: texture_extent,
            array_layer_count: 1,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            usage: wgpu::TextureUsage::SAMPLED | wgpu::TextureUsage::COPY_DST,
        });

        // upload image data to texture
        let temp_buf = render_ctx
            .device
            .create_buffer_mapped(img.len(), wgpu::BufferUsage::COPY_SRC)
            .fill_from_slice(&img);

        render_ctx.encoder.copy_buffer_to_texture(
            wgpu::BufferCopyView {
                buffer: &temp_buf,
                offset: 0,
                row_pitch: img_dim.0 * 4,
                image_height: 0,
            },
            wgpu::TextureCopyView {
                texture: &texture,
                mip_level: 0,
                array_layer: 0,
                origin: Default::default(),
            },
            texture_extent,
        );

        Ok(AssetState::Done(Rc::new(texture)))
    }
}
