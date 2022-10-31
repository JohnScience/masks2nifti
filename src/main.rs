use image::{RgbImage, Rgb, ImageFormat::Png, io::Reader as ImageReader, GenericImageView, Rgba};
use rand::random;
use ndarray::{ArrayBase, OwnedRepr, Dim};
use nifti::writer::WriterOptions;

const MASKS_COUNT: usize = 10;
const MASK_SIZE: (usize, usize) = (11, 12);

fn gen_masks() {
    let mut mask = RgbImage::new(MASK_SIZE.0 as u32, MASK_SIZE.1 as u32);
    // gen masks and save them to files
    for i in 0..MASKS_COUNT {
        for x in 0..MASK_SIZE.0 {
            for y in 0..MASK_SIZE.1 {
                let is_set = random::<bool>();
                let pixel = Rgb([if is_set { 255 } else { 0 }; 3]);
                mask.put_pixel(x as u32, y as u32, pixel);
            }
        }
        mask.save_with_format(format!("mask_{}.png", i), Png).unwrap();
    }
}

fn initialize_mask_layer(volume: &mut ArrayBase::<OwnedRepr<u8>, Dim<[usize;3]>>, i: usize, mask_filename: &str) {
    let img = ImageReader::open(mask_filename)
            .unwrap()
            .decode()
            .unwrap();
    
    let (width, height, _depth) = volume.dim();
    let img_width = img.width() as usize;
    let img_height = img.height() as usize;
    assert!(width == img_width);
    assert!(height == img_height);

    for (x,y, pixel) in img.pixels() {
        let is_set = pixel == Rgba([255,255,255, 255]);
        volume[(x as usize, y as usize, i)] = if is_set { 255 } else { 0 };
    }
}

fn masks2nifti() {
    let mut mask_it = (0..MASKS_COUNT)
        .map(|i| (i,format!("mask_{}.png", i)));

    assert!(MASKS_COUNT >= 1);
    let (i, mask_filename) = mask_it.next().unwrap();

    let img = ImageReader::open(&mask_filename)
        .unwrap()
        .decode()
        .unwrap();
    
    let (width, height) = (img.width() as usize, img.height() as usize);

    let vec = vec![0; (width * height) as usize * MASKS_COUNT];
    let shape = Dim([width as usize, height as usize, MASKS_COUNT]);
    let mut volume: ArrayBase<OwnedRepr<u8>, _> = ArrayBase::from_shape_vec(
        shape,
        vec,
    ).unwrap();

    initialize_mask_layer(&mut volume, i, &mask_filename);

    for (i, mask_filename) in mask_it {
        initialize_mask_layer(&mut volume, i, &mask_filename);
    }
    WriterOptions::new("mask.nii")
        .compress(true)
        .write_nifti(&volume)
        .unwrap();
}

fn main() {
    gen_masks();
    masks2nifti();
}
