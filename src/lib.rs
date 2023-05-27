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

#[derive(::rkyv::Archive, ::rkyv::Deserialize, ::rkyv::Serialize, Debug, PartialEq)]
// #[archive(compare(PartialEq), check_bytes)]
#[archive_attr(derive(Debug))]
pub struct Zip {
    pub meta: Meta,
    pub blob: Vec<u8>,
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
        archived_root, archived_value,
        ser::{serializers::AllocSerializer, Serializer},
        AlignedVec,
    };

    use crate::{ArchivedMeta, Meta, Zip};

    const DIR: &str = env!("CARGO_MANIFEST_DIR");
    const SRC: &str = formatcp!("{}{}{}", DIR, '/', "archive.zip");
    const DST: &str = formatcp!("{}{}{}", DIR, '/', "archive.cust");

    const AGE: u8 = 35;

    fn setup() -> AlignedVec {
        let mut file = File::open(SRC).expect("open zip file");
        let mut blob = Vec::new();
        file.read_to_end(&mut blob).expect("read zip file to bytes");
        let archive = Zip {
            meta: Meta {
                name: "some name".to_owned(),
                age: AGE,
            },
            blob,
        };
        let bytes = rkyv::to_bytes::<_, 1024>(&archive).unwrap();
        let mut file = File::create(DST).expect("open cust file");
        file.write_all(bytes.as_slice()).expect("write cust bytes");
        bytes
    }

    fn setup_custom() {
        let mut file = File::open(SRC).expect("open zip file");
        let mut blob = Vec::new();
        file.read_to_end(&mut blob).expect("read zip file to bytes");
        let name: String = "some name".to_owned();
        let mut file = File::create(DST).expect("open cust file");
        file.write(&name.len().to_le_bytes())
            .expect("write only name len bytes");
        file.write(name.as_bytes()).expect("write only name bytes");
        file.write(&AGE.to_le_bytes())
            .expect("write only age bytes");
        file.write(blob.as_slice())
            .expect("write only src as bytes");
    }

    #[allow(unused_variables)]
    fn setup_other_way() {
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
        // dbg!(pos); // 12
        // dbg!(&serializer); // inner: 24
        let pos = serializer
            .serialize_value(&blob)
            .expect("failed to archive zip");
        // dbg!(pos);
        let bytes = serializer.into_serializer().into_inner();
        assert_eq!(bytes.len(), 106_740_056);
        let mut file = File::create(DST).expect("open cust file");
        file.write_all(bytes.as_slice()).expect("write cust bytes");
    }

    #[test]
    fn read_whole() {
        setup();

        let mut file = File::open(DST).expect("open cust file");
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes)
            .expect("read cust file to bytes");

        let archived = unsafe { archived_root::<Zip>(bytes.as_ref()) };
        assert_eq!(archived.meta.age, AGE);
    }

    #[test]
    fn mmap_partial_initial() {
        setup();

        let file = File::open(DST).expect("open cust file");
        let mmap = unsafe { Mmap::map(&file).expect("map file in memory") };
        let archived = unsafe {
            rkyv::from_bytes_unchecked::<Meta>(&mmap[..]).expect("failed to deserialize meta")
        };
        assert_eq!(archived.age, AGE);
    }

    #[test]
    fn mmap_partial() {
        setup_other_way();

        const SIZE: usize = std::mem::size_of::<ArchivedMeta>();
        assert_eq!(SIZE, 12);
        let file = File::open(DST).expect("open cust file");
        let mmap = unsafe { Mmap::map(&file).expect("map file in memory") };
        let archived = unsafe { archived_value::<Meta>(&mmap[..SIZE], 12) };
        assert_eq!(archived.name, "some name");
        assert_eq!(archived.age, AGE);
    }

    #[test]
    fn custom_mmap_partial() {
        setup_custom();

        let file = File::open(DST).expect("open cust file");
        let mmap = unsafe { Mmap::map(&file).expect("map file in memory") };
        let mut offset = 0;
        let len: [u8; 8] = mmap[..std::mem::size_of::<usize>()].try_into().unwrap();
        let len = usize::from_le_bytes(len);
        offset += len - 1;
        let _name = std::str::from_utf8(&mmap[offset..offset + len]).unwrap();
        offset += len;
        let age = mmap[offset];
        assert_eq!(age, AGE);
    }
}
