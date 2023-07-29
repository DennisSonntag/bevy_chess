#![allow(
	dead_code,
	unused,
	clippy::unreadable_literal,
	clippy::module_name_repetitions
)]

use std::{borrow::Cow, path::Path};

use bevy::{
	asset::load_internal_binary_asset,
	audio::Source,
	prelude::*,
	reflect::TypeUuid,
	render::texture::{CompressedImageFormats, ImageSampler, ImageType},
	sprite::MaterialMesh2dBundle,
	utils::Uuid,
};

pub const PIECE_HANDLE: HandleUntyped =
	HandleUntyped::weak_from_u64(Image::TYPE_UUID, 510291613494514);

pub const FONT_HANDLE: HandleUntyped =
	HandleUntyped::weak_from_u64(Font::TYPE_UUID, 436509473926038);
pub const START_SOUND_HANDLE: HandleUntyped =
	HandleUntyped::weak_from_u64(AudioSource::TYPE_UUID, 587069893843633);

pub const TAKE_SOUND_HANDLE: HandleUntyped =
	HandleUntyped::weak_from_u64(AudioSource::TYPE_UUID, 373560573015187);

pub const MOVE_SOUND_HANDLE: HandleUntyped =
	HandleUntyped::weak_from_u64(AudioSource::TYPE_UUID, 901437001210060);

pub struct BinaryPlugin;

fn font_loader(bytes: &[u8], _: Cow<str>) -> Font {
	Font::try_from_bytes(bytes.to_vec()).expect("could not load font")
}

fn ogg_loader(bytes: &[u8], _: Cow<str>) -> AudioSource {
	AudioSource {
		bytes: bytes.into(),
	}
}

fn image_loader(bytes: &[u8], _: Cow<str>) -> Image {
	let mut image = Image::from_buffer(
		bytes,
		ImageType::Extension("png"),
		CompressedImageFormats::NONE,
		true,
	)
	.expect("could not load image");

	let mut image_descriptor = ImageSampler::nearest_descriptor();
	image_descriptor.label = Some("pieces_image");
	image.sampler_descriptor = ImageSampler::Descriptor(image_descriptor);

	image
}

impl Plugin for BinaryPlugin {
	fn build(&self, app: &mut App) {
		load_internal_binary_asset!(app, FONT_HANDLE, "font/Roboto-Bold.ttf", font_loader);
		load_internal_binary_asset!(app, PIECE_HANDLE, "img/pieces.png", image_loader);
		load_internal_binary_asset!(app, START_SOUND_HANDLE, "sounds/start.ogg", ogg_loader);
		load_internal_binary_asset!(app, TAKE_SOUND_HANDLE, "sounds/take.ogg", ogg_loader);
		load_internal_binary_asset!(app, MOVE_SOUND_HANDLE, "sounds/move.ogg", ogg_loader);
	}
}
