use std::path::Path;
use std::fs::File;
use std::io::Read;
use core::mem::size_of;

use crate::pop::types::{BinDeserializer, from_reader, ImageInfo};

/******************************************************************************/

#[derive(Debug, Copy, Clone)]
#[repr(C, packed)]
pub struct VeleRaw {
    pub sprite_index: u16,
    pub coord_x: i16,
    pub coord_y: i16,
    pub flags: u16,
    pub next_index: u16,
}

impl BinDeserializer for VeleRaw {
    fn from_reader<R: Read>(reader: &mut R) -> Option<Self> {
        from_reader::<VeleRaw, {size_of::<VeleRaw>()}, R>(reader)
    }
}

/******************************************************************************/

#[derive(Debug, Copy, Clone)]
#[repr(C, packed)]
pub struct VfraRaw {
    pub index: u16,
    pub width: u8,
    pub height: u8,
    pub f3: u8,
    pub f4: u8,
    pub next_vfra: u16,
}

impl BinDeserializer for VfraRaw {
    fn from_reader<R: Read>(reader: &mut R) -> Option<Self> {
        from_reader::<VfraRaw, {size_of::<VfraRaw>()}, R>(reader)
    }
}

/******************************************************************************/

#[derive(Debug, Copy, Clone)]
#[repr(C, packed)]
pub struct VstartRaw {
    pub index: u16,
    pub f1: u8,
    pub f2: u8,
}

impl BinDeserializer for VstartRaw {
    fn from_reader<R: Read>(reader: &mut R) -> Option<Self> {
        from_reader::<VstartRaw, {size_of::<VstartRaw>()}, R>(reader)
    }
}

/******************************************************************************/

#[derive(Debug)]
pub struct AnimationsData {
    pub vele: Vec<VeleRaw>,
    pub vfra: Vec<VfraRaw>,
    pub vstart: Vec<VstartRaw>,
}

impl AnimationsData {

    pub fn from_reader<R: Read>(reader_vele: &mut R
                               , reader_vfra: &mut R
                               , reader_vstart: &mut R) -> Self {
        AnimationsData {
            vele: VeleRaw::from_reader_vec(reader_vele),
            vfra: VfraRaw::from_reader_vec(reader_vfra),
            vstart: VstartRaw::from_reader_vec(reader_vstart),
        }
    }

    pub fn from_path(path: &Path) -> Self {
        let mut file_vele = File::options().read(true).open(path.join("VELE-0.ANI")).unwrap();
        let mut file_vfra = File::options().read(true).open(path.join("VFRA-0.ANI")).unwrap();
        let mut file_vstart = File::options().read(true).open(path.join("VSTART-0.ANI")).unwrap();
        Self::from_reader(&mut file_vele, &mut file_vfra, &mut file_vstart)
    }
}

/******************************************************************************/

pub enum ElementRotate {
    NoRotate,
    RotateHorizontal,
    RotateVertical,
}

#[derive(Debug, Copy, Clone)]
pub struct AnimationElement {
    pub sprite_index: usize,
    pub coord_x: i16,
    pub coord_y: i16,
    pub tribe: u8,
    pub flags: u16,
    pub uvar5: u16,
    pub original_flags: u16,
}

#[derive(Debug, Clone)]
pub struct AnimationFrame {
    pub index: usize,
    pub width: usize,
    pub height: usize,
    pub sprites: Vec<AnimationElement>,
}

pub struct AnimationSequence {
    pub index: usize,
    pub frames: Vec<AnimationFrame>,
}

impl AnimationElement {
    pub fn get_tribe(&self) -> u8 {
        self.tribe
    }

    pub fn is_hidden(&self) -> bool {
        self.flags == 0x4
    }

    pub fn is_common(&self) -> bool {
        !(self.is_tribe_specific() || self.is_type_specific())
    }

    pub fn is_tribe_specific(&self) -> bool {
        self.uvar5 == 0x1 && self.flags == 0x10
    }

    pub fn is_type_specific(&self) -> bool {
        self.uvar5 > 1
    }

    pub fn get_rotate(&self) -> ElementRotate {
        if (self.flags & 0x1) != 0 {
            ElementRotate::RotateHorizontal
        } else if (self.flags & 0x2) != 0 {
            ElementRotate::RotateVertical
        } else {
            ElementRotate::NoRotate
        }
    }

    pub fn from_data(index: u16, vele: &[VeleRaw]) -> Vec<Self> {
        let mut sprites = Vec::new();
        let mut vele_index = index as usize;
        while vele_index != 0 {
            let vele_sprite = &vele[vele_index];
            sprites.push(AnimationElement{
                sprite_index: vele_sprite.sprite_index as usize / 6,
                coord_x: vele_sprite.coord_x,
                coord_y: vele_sprite.coord_y,
                tribe: (vele_sprite.flags >> 9) as u8,
                flags: vele_sprite.flags & 0x1f,
                uvar5: (vele_sprite.flags & 0x1f0) >> 4,
                original_flags: vele_sprite.flags,
            });
            vele_index = vele_sprite.next_index as usize;
            if sprites.len() > 255 {
                break;
            }
        }
        sprites
    }
}

impl AnimationFrame {
    pub fn get_permutations(&self, with_tribe: bool, with_type: bool) -> Vec<Vec<AnimationElement>> {
        let mut common_elems = Vec::new();
        let mut tribe_elems = Vec::new();
        let mut type_elems = Vec::new();
        for elem in &self.sprites {
            if elem.is_hidden() {
                continue;
            }
            if elem.is_common() {
                common_elems.push(*elem);
                if with_type {
                    type_elems.push(*elem);
                }
                if with_tribe {
                    tribe_elems.push(*elem);
                }
            } else if with_tribe && elem.is_tribe_specific() {
                tribe_elems.push(*elem);
            } else if with_type && elem.is_type_specific() {
                type_elems.push(*elem);
            }
        }
        let mut res = Vec::new();
        if tribe_elems.is_empty() && type_elems.is_empty() {
            res.push(common_elems);
        } else if !tribe_elems.is_empty() {
            for tribe_elem in tribe_elems {
                let mut res_tribe = common_elems.clone();
                res_tribe.push(tribe_elem);
                if type_elems.is_empty() {
                    res.push(res_tribe);
                } else {
                    for type_elem in &type_elems {
                        let mut res_type = res_tribe.clone();
                        res_type.push(*type_elem);
                        res.push(res_type);
                    }
                }
            }
        } else {
            for type_elem in &type_elems {
                let mut res_type = common_elems.clone();
                res_type.push(*type_elem);
                res.push(res_type);
            }
        }
        res
    }

    pub fn from_data(index: u16, vfra: &[VfraRaw], vele: &[VeleRaw]) -> Vec<Self> {
        let mut frames = Vec::new();
        let mut vfra_index = index as usize;
        while vfra_index != 0 {
            let vfra_frame = &vfra[vfra_index];
            frames.push(AnimationFrame{
                index: vfra_index,
                width: vfra_frame.width as usize,
                height: vfra_frame.height as usize,
                sprites: AnimationElement::from_data(vfra_frame.index, vele),
            });
            vfra_index = vfra_frame.next_vfra as usize;
            if frames.len() > 255 {
                break;
            }
            if vfra_index == (index as usize) {
                break;
            }
        }
        frames
    }
}

impl ImageInfo for AnimationFrame {
    fn width(&self) -> usize {
        self.width
    }

    fn height(&self) -> usize {
        self.height
    }
}

impl AnimationSequence {
    pub fn from_data(anim_data: &AnimationsData) -> Vec<Self> {
        let mut res = Vec::<Self>::with_capacity(anim_data.vstart.len());
        for (index, vstart) in (0..).zip(&anim_data.vstart) {
            let frames = AnimationFrame::from_data(vstart.index, &anim_data.vfra, &anim_data.vele);
            res.push(AnimationSequence{index, frames});
        }
        res
    }

    pub fn get_frames(anim_seq_vec: &Vec<Self>) -> Vec<AnimationFrame> {
        let mut frames = Vec::new();
        for anim_seq in anim_seq_vec {
            frames.extend_from_slice(&anim_seq.frames);
        }
        frames
    }
}

/******************************************************************************/
