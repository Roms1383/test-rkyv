// use chrono::{DateTime, Utc};

#[derive(::rkyv::Archive, ::rkyv::Deserialize, ::rkyv::Serialize, Debug, PartialEq)]
// #[archive(compare(PartialEq), check_bytes)]
#[archive_attr(derive(Debug))]
pub struct Meta {
    pub name: String,
    pub one: String,
    pub two: String,
    pub three: String,
    pub age: u8,
    // blocked on chrono PR (support for PartialEq/PartialOrd)
    // pub finalized: Option<DateTime<Utc>>,
}

#[cfg(test)]
mod tests {
    use std::{
        fs::File,
        io::{Read, Write},
    };

    use const_format::formatcp;
    use fake::{Fake, Faker};
    use memmap2::Mmap;
    use rkyv::{
        archived_value,
        ser::{serializers::AllocSerializer, Serializer},
    };

    use crate::{ArchivedMeta, Meta};

    const DIR: &str = env!("CARGO_MANIFEST_DIR");
    const SRC: &str = formatcp!("{}{}{}", DIR, '/', "archive.zip");
    const DST: &str = formatcp!("{}{}{}", DIR, '/', "archive.cust");

    const NAME: &str = "Jean-Guy";
    const AGE: u8 = 35;

    #[cfg(target_pointer_width = "32")]
    const SIZE_POS: usize = 4;
    #[cfg(target_pointer_width = "64")]
    const SIZE_POS: usize = 8;

    #[allow(unused_variables)]
    fn setup() {
        let mut file = File::open(SRC).expect("open zip file");
        let mut blob = Vec::new();
        file.read_to_end(&mut blob).expect("read zip file to bytes");

        let meta = Meta {
            name: NAME.to_owned(),
            one: Faker.fake(),
            two: Faker.fake(),
            three: Faker.fake(),
            age: AGE,
        };
        let mut serializer = AllocSerializer::<1024>::default();
        let pos = serializer
            .serialize_value(&meta)
            .expect("failed to archive meta");
        // dbg!(unsafe {
        //     ::core::slice::from_raw_parts(
        //         (&meta as *const Meta) as *const u8,
        //         ::core::mem::size_of::<Meta>(),
        //     )
        // }); // 32
        // dbg!(&serializer); // inner: 24
        dbg!(&pos);
        let bytes = serializer.into_serializer().into_inner();
        let mut file = File::create(DST).expect("open cust file");
        // pos varies depending on content, so let's store it
        file.write(&pos.to_le_bytes())
            .expect("write cust pos bytes");
        file.write(bytes.as_slice()).expect("write cust meta bytes");
        file.write(blob.as_slice()).expect("write cust zip bytes");
    }

    #[test]
    fn mmap_partial() {
        setup();

        const SIZE_META: usize = std::mem::size_of::<ArchivedMeta>();
        let file = File::open(DST).expect("open cust file");
        let mmap = unsafe { Mmap::map(&file).expect("map file in memory") };
        let pos: [u8; SIZE_POS] = mmap[..SIZE_POS].try_into().expect("read pos");
        let pos = usize::from_le_bytes(pos);
        let archived = unsafe { archived_value::<Meta>(&mmap[..SIZE_META], SIZE_POS + pos) };
        assert_eq!(archived.name, NAME);
        assert_eq!(archived.age, AGE);
    }
}
