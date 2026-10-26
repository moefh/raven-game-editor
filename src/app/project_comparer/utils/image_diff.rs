use crate::data_store::{
    DataHash,
    DataHasher,
};
use crate::image::{
    ImageCollectionIO,
};

struct DiffBuilder {
    diff: Vec<ImageDiff>,
}

impl DiffBuilder {
    fn new() -> Self {
        DiffBuilder {
            diff: Vec::new(),
        }
    }
}

impl diffs::Diff for DiffBuilder {
    type Error = ();

    fn equal(&mut self, old: usize, new: usize, len: usize) -> Result<(), Self::Error> {
        for i in 0..len {
            self.diff.push(ImageDiff::Equal { a: (old + i) as u32, b: (new + i) as u32 });
        }
        Ok(())
    }

    fn delete(&mut self, old: usize, len: usize, _new: usize) -> Result<(), Self::Error> {
        for i in 0..len {
            self.diff.push(ImageDiff::A { a: (old + i) as u32 });
        }
        Ok(())
    }

    fn insert(&mut self, _old: usize, new: usize, new_len: usize) -> Result<(), Self::Error> {
        for i in 0..new_len {
            self.diff.push(ImageDiff::B { b: (new + i) as u32 });
        }
        Ok(())
    }
}

pub enum ImageDiff {
    A { a: u32 },
    B { b: u32 },
    Equal { a: u32, b: u32 },
}

impl ImageDiff {

    fn calc_hashes(images: &impl ImageCollectionIO) -> Vec<DataHash> {
        let item_size = (images.width() * images.height()) as usize;
        (0..images.num_items()).into_iter().map(|index| {
            let index = index as usize;
            let mut hasher = DataHasher::new();
            hasher.add_bytes(&images.data()[index*item_size..(index+1)*item_size]);
            hasher.finalize()
        }).collect()
    }

    pub fn compare(a_images: &impl ImageCollectionIO, b_images: &impl ImageCollectionIO) -> Vec<ImageDiff> {
        let a_hashes = Self::calc_hashes(a_images);
        let b_hashes = Self::calc_hashes(b_images);

        let mut d = DiffBuilder::new();
        let _ = diffs::myers::diff(&mut d, &a_hashes, 0, a_hashes.len(), &b_hashes, 0, b_hashes.len());

        d.diff
    }
}
