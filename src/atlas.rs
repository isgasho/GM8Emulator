use rect_packer::DensePacker;

pub struct AtlasBuilder {
    max_size: i32,
    packers: Vec<DensePacker>,
}

pub struct AtlasRef {
    pub atlas_id: u32,

    pub w: i32,
    pub h: i32,
    pub x: i32,
    pub y: i32,
}

impl AtlasBuilder {
    pub fn new(max_wh: i32) -> Self {
        AtlasBuilder {
            max_size: max_wh,
            packers: Vec::with_capacity(1),
        }
    }

    pub fn add(&mut self, w: i32, h: i32) -> AtlasRef {
        for (i, pk) in self.packers.iter_mut().enumerate() {
            if let Some(rect) = pk.pack(w as _, h as _, false) {
                return AtlasRef {
                    atlas_id: i as _,
                    w: rect.width,
                    h: rect.height,
                    x: rect.x,
                    y: rect.y,
                };
            }
        }

        if w > self.max_size || h > self.max_size {
            panic!("what the fuck that's big"); // TODO lol
        } else {
            self.packers.push(DensePacker::new(self.max_size, self.max_size));
            self.add(w, h)
        }
    }
}
