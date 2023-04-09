use std::path::Path;
use std::fs::File;
use std::io::Read;
use std::marker::PhantomData;

/******************************************************************************/

pub trait BinDeserializer {
    fn from_reader<R: Read>(reader: &mut R) -> Option<Self> where Self: Sized;

    fn from_reader_vec<R: Read>(reader: &mut R) -> Vec<Self> where Self: Sized {
        let mut res = Vec::new();
        while let Some(obj) = Self::from_reader(reader) {
            res.push(obj);
        }
        res
    }

    fn from_file_vec(path: &Path) -> Vec<Self> where Self: Sized {
        let mut file = File::options().read(true).open(path).unwrap();
        Self::from_reader_vec(&mut file)
    }

    fn from_file(path: &Path) -> Option<Self> where Self: Sized {
        let mut file = File::options().read(true).open(path).unwrap();
        Self::from_reader(&mut file)
    }
}

pub fn from_reader<T, const S: usize, R: Read>(reader: &mut R) -> Option<T> where T: Copy {
    let mut data = [0u8; S];
    if let Ok(()) = reader.read_exact(&mut data) {
        return Some(unsafe {
            *(data.as_ptr() as *const T)
        });
    }
    None
}

pub fn from_reader_vec<T, const S: usize, R: Read>(reader: &mut R) -> Vec<T> where T: Copy {
    let mut items = Vec::new();
    let mut data = [0u8; S];
    while let Ok(()) = reader.read_exact(&mut data) {
        items.push(unsafe {
            *(data.as_ptr() as *const T)
        });
    }
    items
}

/******************************************************************************/

pub trait ImageInfo {
    fn width(&self) -> usize;
    fn height(&self) -> usize;
}

pub trait ImageStorage : ImageInfo {
    //x - width, y - height
    fn set_pixel(&mut self, x: usize, y: usize, val: u8);
    fn set_line(&mut self, x: usize, y: usize, data: &[u8]);
    fn set_image(&mut self, data: &[u8]);
}

pub trait ImageAllocator<T> {
    fn alloc<I: ImageInfo>(&self, info: &I) -> T;
}

pub trait ImageStorageSource {
    type StorageType: ImageStorage;

    fn get_storage<I: ImageInfo>(&mut self, info: &I) -> Option<&mut Self::StorageType>;
}

pub trait ImageComposer {
    fn compose<I: ImageInfo, S: ImageInfo + ImageStorage>(&mut self, storage: &mut ImageTile<S>, info: &I) -> bool;
}

pub trait AllocatorEqual<T> {
    fn alloc_equal<I: ImageInfo>(&self, info: &I, num: usize) -> T;
}

pub trait AllocatorIter<T> {
    fn alloc_iter<'a, I: ImageInfo + 'a, R: Iterator<Item=&'a I>>(&'a self, iter: &mut R) -> T;
}

/******************************************************************************/

pub struct ImageSourceComposed<C, S> {
    composer: C,
    tile: ImageTile<S>,
}

impl<C, S> ImageSourceComposed<C, S> {
    pub fn new(composer: C, image: S) -> Self {
        Self{composer, tile: ImageTile::new(image)}
    }

    pub fn get_image(self) -> S {
        self.tile.get_image()
    }
}

impl<C: ImageComposer, M: ImageInfo + ImageStorage> ImageStorageSource for ImageSourceComposed<C, M> {
    type StorageType = ImageTile<M>;

    fn get_storage<I: ImageInfo>(&mut self, info: &I) -> Option<&mut Self::StorageType> {
        if self.composer.compose(&mut self.tile, info) {
            Some(&mut self.tile)
        } else {
            None
        }
    }
}

/******************************************************************************/

pub struct ImageAllocatorComposed<C, A, R> {
    composer: C,
    image: A,
    phantom: PhantomData<R>,
}

impl<C, A, R> ImageAllocatorComposed<C, A, R> {
    pub fn new(composer: C, image: A) -> Self {
        Self{composer, image, phantom: PhantomData}
    }
}

impl<C, A, R, I> AllocatorEqual<ImageSourceComposed<R, I>> for ImageAllocatorComposed<C, A, R>
    where C: AllocatorEqual<R>, R: ImageComposer + ImageInfo, I: ImageStorage, A: ImageAllocator<I> {
    fn alloc_equal<U: ImageInfo>(&self, info: &U, num: usize) -> ImageSourceComposed<R, I> {
        let c = self.composer.alloc_equal(info, num);
        let image = self.image.alloc(&c);
        let tile = ImageTile{image, start_x: 0, start_y: 0, tile_width: 0, tile_height: 0};
        ImageSourceComposed{composer: c, tile}
    }
}

impl<C, A, R, I> AllocatorIter<ImageSourceComposed<R, I>> for ImageAllocatorComposed<C, A, R>
    where C: AllocatorIter<R>, R: ImageComposer + ImageInfo, I: ImageStorage, A: ImageAllocator<I> {
    fn alloc_iter<'a, U: ImageInfo + 'a, T: Iterator<Item=&'a U>>(&'a self, iter: &mut T) -> ImageSourceComposed<R, I> {
        let c = self.composer.alloc_iter(iter);
        let image = self.image.alloc(&c);
        let tile = ImageTile{image, start_x: 0, start_y: 0, tile_width: 0, tile_height: 0};
        ImageSourceComposed{composer: c, tile}
    }
}

impl AllocatorEqual<ImageComposer1D<ComposeHorizontal>> for () {
    fn alloc_equal<I: ImageInfo>(&self, info: &I, num: usize) -> ImageComposer1D<ComposeHorizontal> {
        ImageComposer1D{total_width: info.width() * num
             , total_height: info.height()
             , tile_width: info.width()
             , tile_height: info.height()
             , pos_x: 0
             , pos_y: 0
             , phantom: PhantomData
        }
    }
}

impl AllocatorEqual<ImageComposer1D<ComposeVertical>> for () {
    fn alloc_equal<I: ImageInfo>(&self, info: &I, num: usize) -> ImageComposer1D<ComposeVertical> {
        ImageComposer1D{total_width: info.width()
             , total_height: info.height() * num
             , tile_width: info.width()
             , tile_height: info.height()
             , pos_x: 0
             , pos_y: 0
             , phantom: PhantomData
        }
    }
}

impl AllocatorIter<ImageComposer2D> for usize {
    fn alloc_iter<'a, U: ImageInfo + 'a, T: Iterator<Item=&'a U>>(&'a self, iter: &mut T) -> ImageComposer2D {
        let max_width = *self;
        let mut cur_width = max_width;
        let mut max_height = 0;
        let mut total_height = 0;
        for info in iter {
            if cur_width < info.width() {
                cur_width = max_width;
                total_height += max_height;
                max_height = 0;
            }
            cur_width -= info.width();
            max_height = max_height.max(info.height());
        }
        total_height += max_height;
        ImageComposer2D{total_width: max_width
             , total_height
             , pos_x: 0
             , pos_y: 0
             , max_height: 0}
    }
}

/******************************************************************************/

pub fn image_allocator_1d_horizontal()
    -> ImageAllocatorComposed<(), (), ImageComposer1D<ComposeHorizontal>> {
    ImageAllocatorComposed::new((), ())
}

pub fn image_allocator_1d_vertical()
    -> ImageAllocatorComposed<(), (), ImageComposer1D<ComposeVertical>> {
    ImageAllocatorComposed::new((), ())
}

pub fn image_allocator_2d(width: usize) -> ImageAllocatorComposed<usize, (), ImageComposer2D> {
    ImageAllocatorComposed::new(width, ())
}

pub fn pal_image_allocator_1d_vertical(pal: &[u8])
    -> ImageAllocatorComposed<(), &[u8], ImageComposer1D<ComposeVertical>> {
    ImageAllocatorComposed::new((), pal)
}

/******************************************************************************/

pub struct ComposeHorizontal;
pub struct ComposeVertical;

pub struct ImageComposer1D<D> {
    total_width: usize,
    total_height: usize,
    tile_width: usize,
    tile_height: usize,
    pos_x: usize,
    pos_y: usize,
    phantom: PhantomData<D>,
}

impl ImageComposer for ImageComposer1D<ComposeHorizontal> {
    fn compose<I: ImageInfo, S: ImageInfo + ImageStorage>(&mut self, storage: &mut ImageTile<S>, info: &I) -> bool {
        if (self.total_width - self.pos_x) < info.width() {
            return false;
        }
        if self.total_height < info.height() {
            return false;
        }
        storage.start_x = self.pos_x;
        storage.start_y = self.pos_y;
        storage.tile_width = info.width();
        storage.tile_height = info.height();
        self.pos_x += self.tile_width;
        true
    }
}

impl ImageComposer for ImageComposer1D<ComposeVertical> {
    fn compose<I: ImageInfo, S: ImageInfo + ImageStorage>(&mut self, storage: &mut ImageTile<S>, info: &I) -> bool {
        if self.total_width < info.width() {
            return false;
        }
        if (self.total_height - self.pos_y) < info.height() {
            return false;
        }
        storage.start_x = self.pos_x;
        storage.start_y = self.pos_y;
        storage.tile_width = info.width();
        storage.tile_height = info.height();
        self.pos_y += self.tile_height;
        true
    }
}

impl<D> ImageInfo for ImageComposer1D<D> {
    fn width(&self) -> usize {
        self.total_width
    }

    fn height(&self) -> usize {
        self.total_height
    }
}

/******************************************************************************/

pub struct ImageComposer2D {
    total_width: usize,
    total_height: usize,
    pos_x: usize,
    pos_y: usize,
    max_height: usize,
}

impl ImageComposer for ImageComposer2D {
    fn compose<I: ImageInfo, S: ImageInfo + ImageStorage>(&mut self, storage: &mut ImageTile<S>, info: &I) -> bool {
        if (self.pos_x + info.width()) > self.total_width {
            self.pos_x = 0;
            self.pos_y += self.max_height;
            self.max_height = 0;
        }
        if self.pos_y > self.total_height {
            return false;
        }
        storage.start_x = self.pos_x;
        storage.start_y = self.pos_y;
        storage.tile_width = info.width();
        storage.tile_height = info.height();
        self.pos_x += info.width();
        self.max_height = self.max_height.max(info.height());
        true
    }
}

impl ImageInfo for ImageComposer2D {
    fn width(&self) -> usize {
        self.total_width
    }

    fn height(&self) -> usize {
        self.total_height
    }
}

/******************************************************************************/

pub struct ImageTile<I> {
    image: I,
    start_x: usize,
    start_y: usize,
    tile_width: usize,
    tile_height: usize,
}

impl<I> ImageTile<I> {
    pub fn new(image: I) -> Self {
        Self{image, start_x: 0, start_y: 0, tile_width: 0, tile_height: 0}
    }

    fn get_image(self) -> I {
        self.image
    }
}

impl<I: ImageInfo + ImageStorage> ImageInfo for ImageTile<I> {
    fn width(&self) -> usize {
        self.tile_width
    }

    fn height(&self) -> usize {
        self.tile_height
    }
}

impl<I: ImageInfo + ImageStorage> ImageStorage for ImageTile<I> {
    fn set_pixel(&mut self, x: usize, y: usize, val: u8) {
        self.image.set_pixel(self.start_x + x, self.start_y + y, val);
    }

    fn set_line(&mut self, x: usize, y: usize, data: &[u8]) {
        self.image.set_line(self.start_x + x, self.start_y + y, data);
    }

    fn set_image(&mut self, data: &[u8]) {
        for y in 0..self.tile_height {
            let offset = y * self.tile_width;
            let slice = &data[offset..(offset+self.tile_width)];
            self.set_line(0, y, slice);
        }
    }
}

/******************************************************************************/

pub trait ImageTileSource {
    type Tile: ImageStorage + ImageInfo;

    fn next_tile(&mut self, x: usize, y: usize) -> &mut Self::Tile;
}

pub struct TiledComposer {
    total_width: usize, // number of horizontal tiles
    total_height: usize, // number of vertical tiles
    tile_width: usize,
    tile_height: usize,
}

impl TiledComposer {
    pub fn new(total_width: usize, total_height: usize, tile_width: usize, tile_height: usize) -> Self {
        Self{total_width, total_height, tile_width, tile_height}
    }

    pub fn set_tile<S: ImageInfo + ImageStorage>(&mut self, storage: &mut ImageTile<S>, x: usize, y: usize) -> bool {
        if x >= self.total_width {
            return false;
        }
        if y >= self.total_height {
            return false;
        }
        storage.start_x = x * self.tile_width;
        storage.start_y = y * self.tile_height;
        storage.tile_width = self.tile_width;
        storage.tile_height = self.tile_height;
        true
    }
}

impl<M: ImageInfo + ImageStorage> ImageTileSource for ImageSourceComposed<TiledComposer, M> {
    type Tile = ImageTile<M>;

    fn next_tile(&mut self, x: usize, y: usize) -> &mut Self::Tile {
        if !self.composer.set_tile(&mut self.tile, x, y) {
            panic!("Cannot set tile {x:?},{y:?}");
        }
        &mut self.tile
    }
}

/******************************************************************************/

impl ImageStorageSource for Vec<Image> {
    type StorageType = Image;

    fn get_storage<I: ImageInfo>(&mut self, info: &I) -> Option<&mut Self::StorageType> {
        let storage = Image::alloc(info.width(), info.height());
        self.push(storage);
        let index = self.len()-1;
        self.get_mut(index)
    }
}

impl ImageInfo for (usize, usize) {
    fn width(&self) -> usize {
        self.0
    }

    fn height(&self) -> usize {
        self.1
    }
}

/******************************************************************************/

pub struct Image {
    pub width: usize,
    pub height: usize,
    pub data: Vec<u8>,
}

impl Image {
    pub fn new(width: usize, height: usize, data: Vec<u8>) -> Self {
        Self{width, height, data}
    }

    pub fn alloc(width: usize, height: usize) -> Self {
        Self{width, height, data: vec![0u8; width * height]}
    }

    fn index(&self, x: usize, y: usize) -> usize {
        (self.width * y + x).min(self.data.len())
    }
}

impl ImageInfo for Image {
    fn width(&self) -> usize {
        self.width
    }

    fn height(&self) -> usize {
        self.height
    }
}

impl ImageStorage for Image {
    fn set_pixel(&mut self, x: usize, y: usize, val: u8) {
        let index = self.index(x, y);
        self.data[index] = val;
    }

    fn set_line(&mut self, x: usize, y: usize, data: &[u8]) {
        let index = self.index(x, y);
        let to_copy = data.len().min(self.width - x);
        self.data[index..(index+to_copy)].copy_from_slice(&data[0..to_copy]);
    }

    fn set_image(&mut self, data: &[u8]) {
        let to_copy = data.len().min(self.data.len());
        self.data.copy_from_slice(&data[0..to_copy]);
    }
}

impl ImageAllocator<Image> for () {
    fn alloc<I: ImageInfo>(&self, info: &I) -> Image {
        Image::alloc(info.width(), info.height())
    }
}

/******************************************************************************/

pub struct PalImage<'a> {
    pub pal: &'a[u8],
    pub width: usize,
    pub height: usize,
    pub data: Vec<u8>,
}

impl<'a> PalImage<'a> {
    pub fn new(pal: &'a[u8], width: usize, height: usize, data: Vec<u8>) -> Self {
        Self{pal, width, height, data}
    }

    pub fn alloc(pal: &'a[u8], width: usize, height: usize) -> Self {
        Self{pal, width, height, data: vec![0u8; width * height * 4]}
    }

    fn index(&self, x: usize, y: usize) -> usize {
        (self.width * y * 4 + x * 4).min(self.data.len())
    }
}

impl<'a> ImageInfo for PalImage<'a> {
    fn width(&self) -> usize {
        self.width
    }

    fn height(&self) -> usize {
        self.height
    }
}

impl<'a> ImageStorage for PalImage<'a> {
    fn set_pixel(&mut self, x: usize, y: usize, val: u8) {
        let index = self.index(x, y);
        let pal_index = val as usize * 4;
        self.data[index..(index+4)].copy_from_slice(&self.pal[pal_index..(pal_index+4)]);
    }

    fn set_line(&mut self, x: usize, y: usize, data: &[u8]) {
        let index = self.index(x, y);
        for (i, val) in data.iter().enumerate() {
            let data_index = index + i * 4;
            let pal_index = *val as usize * 4;
            self.data[data_index..(data_index+4)].copy_from_slice(&self.pal[pal_index..(pal_index+4)]);
        }
    }

    fn set_image(&mut self, data: &[u8]) {
        let _to_copy = data.len().min(self.data.len());
        //self.data.copy_from_slice(&data[0..to_copy]);
    }
}

impl<'a> ImageAllocator<PalImage<'a>> for &'a[u8] {
    fn alloc<I: ImageInfo>(&self, info: &I) -> PalImage<'a> {
        PalImage::alloc(self, info.width(), info.height())
    }
}

/******************************************************************************/

impl<M, F> ImageAllocator<M> for F where F: Fn(&dyn ImageInfo) -> M {
    fn alloc<I: ImageInfo>(&self, info: &I) -> M {
        (self)(info)
    }
}

/******************************************************************************/
