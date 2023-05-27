// use chrono::{DateTime, Utc};

#[derive(::rkyv::Archive, ::rkyv::Deserialize, ::rkyv::Serialize, Debug, PartialEq)]
// #[archive(compare(PartialEq), check_bytes)]
#[archive_attr(derive(Debug))]
pub struct Meta {
    pub name: String,
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
    use memmap2::Mmap;
    use rkyv::{
        archived_value,
        ser::{serializers::AllocSerializer, Serializer},
    };

    use crate::{ArchivedMeta, Meta};

    const DIR: &str = env!("CARGO_MANIFEST_DIR");
    const SRC: &str = formatcp!("{}{}{}", DIR, '/', "archive.zip");
    const DST: &str = formatcp!("{}{}{}", DIR, '/', "archive.cust");

    const AGE: u8 = 35;

    #[allow(unused_variables)]
    fn setup() {
        let mut file = File::open(SRC).expect("open zip file");
        let mut blob = Vec::new();
        file.read_to_end(&mut blob).expect("read zip file to bytes");
        assert_eq!(blob.len(), 106_740_021);

        let meta = Meta {
            name: "some name".to_owned(),
            age: AGE,
        };
        let mut serializer = AllocSerializer::<0>::default();
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
        dbg!(pos); // 12
        let bytes = serializer.into_serializer().into_inner();
        let mut file = File::create(DST).expect("open cust file");
        file.write(bytes.as_slice()).expect("write cust meta bytes");
        file.write(blob.as_slice()).expect("write cust zip bytes");
        assert_eq!(bytes.len() + blob.len(), 106_740_045);
    }

    #[test]
    fn mmap_partial() {
        setup();

        const SIZE: usize = std::mem::size_of::<ArchivedMeta>();
        assert_eq!(SIZE, 12);
        let file = File::open(DST).expect("open cust file");
        let mmap = unsafe { Mmap::map(&file).expect("map file in memory") };
        let archived = unsafe { archived_value::<Meta>(&mmap[..SIZE], 12) };
        assert_eq!(archived.name, "some name");
        assert_eq!(archived.age, AGE);
    }
}
