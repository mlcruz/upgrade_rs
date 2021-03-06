#![allow(dead_code)]

use std::os::raw::c_void;
use std::path::Path;

use gl;
use image;
use image::DynamicImage;
use image::GenericImage;
use rayon::iter::IntoParallelRefIterator;
use rayon::iter::ParallelIterator;

// A função de carregamento de imagems da implementação original da biblioteca images é muito lenta,
// Reimplementamos a mesma sem deferênciação dos valores e com paralelismo
unsafe fn image_to_bytes(image: &DynamicImage) -> Vec<&u8> {
    match *image {
        DynamicImage::ImageLuma8(ref a) => a.par_iter().collect(),
        DynamicImage::ImageLumaA8(ref a) => a.par_iter().collect(),
        DynamicImage::ImageRgb8(ref a) => a.par_iter().collect(),
        DynamicImage::ImageRgba8(ref a) => a.par_iter().collect(),
    }
}

pub unsafe fn load_texture(path: &str) -> (u32, u32) {
    // Le arquivo de imagem
    let img = image::open(&Path::new(&path))
        .expect("Falha ao carregar textura")
        .rotate180();

    let data = image_to_bytes(&img);

    // let data2 = img.flipv().pixels().map(|pixel|pixel)
    let mut texture_id = 0;
    let mut sampler_id = 0;

    gl::GenTextures(1, &mut texture_id);
    gl::GenSamplers(1, &mut sampler_id);

    // Veja slide 100 do documento "Aula_20_e_21_Mapeamento_de_Texturas.pdf"
    gl::SamplerParameteri(sampler_id, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
    gl::SamplerParameteri(sampler_id, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);

    // Parâmetros de amostragem da textura.
    gl::SamplerParameteri(
        sampler_id,
        gl::TEXTURE_MIN_FILTER,
        gl::LINEAR_MIPMAP_LINEAR as i32,
    );

    gl::SamplerParameteri(sampler_id, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);

    gl::PixelStorei(gl::UNPACK_ALIGNMENT, 1);
    gl::PixelStorei(gl::UNPACK_ROW_LENGTH, 0);
    gl::PixelStorei(gl::UNPACK_SKIP_PIXELS, 0);
    gl::PixelStorei(gl::UNPACK_SKIP_ROWS, 0);

    // Agora enviamos a imagem lida do disco para a GPU
    gl::ActiveTexture(gl::TEXTURE0 + texture_id);
    gl::BindTexture(gl::TEXTURE_2D, texture_id);

    gl::TexImage2D(
        gl::TEXTURE_2D,
        0,
        gl::SRGB8 as i32,
        img.width() as i32,
        img.height() as i32,
        0,
        gl::RGB,
        gl::UNSIGNED_BYTE,
        data[0] as *const u8 as *const c_void,
    );

    gl::GenerateMipmap(gl::TEXTURE_2D);
    gl::BindSampler(texture_id, sampler_id);

    (texture_id, sampler_id)
}
