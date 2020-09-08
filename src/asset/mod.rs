//! Loadable stuff

use std::io::Error;
use std::path::Path;
use std::rc::Rc;

// Example Asset implementation
// TODO: move to crates when done.
use crate::gfx::{RenderContext, Texture};

#[cfg(feature = "3d")]
pub mod scene;
#[cfg(feature = "2d")]
pub mod tilemap;

// wrapper for in memory bytes because asref path things
pub struct Bytes(pub &'static [u8]);

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
            label: None,
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
            .create_buffer_with_data(&img.into_raw(), wgpu::BufferUsage::COPY_SRC);

        render_ctx.encoder.copy_buffer_to_texture(
            wgpu::BufferCopyView {
                buffer: &temp_buf,
                offset: 0,
                bytes_per_row: img_dim.0 * 4,
                rows_per_image: 0,
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

impl Asset<Texture> for Bytes {
    fn poll(
        self: Box<Self>,
        render_ctx: &mut RenderContext,
    ) -> Result<AssetState<Texture>, AssetError> {
        let img = image::load_from_memory(&self.0[..])?.to_bgra();
        let img_dim = img.dimensions();

        let texture_extent = wgpu::Extent3d {
            width: img_dim.0,
            height: img_dim.1,
            depth: 1,
        };

        let texture = render_ctx.device.create_texture(&wgpu::TextureDescriptor {
            label: None,
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
            .create_buffer_with_data(&img.into_raw(), wgpu::BufferUsage::COPY_SRC);

        render_ctx.encoder.copy_buffer_to_texture(
            wgpu::BufferCopyView {
                buffer: &temp_buf,
                offset: 0,
                bytes_per_row: img_dim.0 * 4,
                rows_per_image: 0,
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
