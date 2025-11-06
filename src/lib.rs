use {
    tokio::io::AsyncWriteExt,
    tokio_tar::{Builder, Header},
};

pub use tokio_tar::Archive;

pub struct ArchiveItem<'a> {
    pub path: &'a str,
    pub data: &'a [u8],
}

pub async fn pack<'a>(items: &[ArchiveItem<'a>]) -> std::io::Result<Box<[u8]>> {
    let mut archive_builder = Builder::new(Vec::new());
    for ArchiveItem { path, data } in items {
        let mut header = Header::new_gnu();
        header.set_size(data.len() as u64);
        header.set_mode(0o777);
        header.set_cksum();

        archive_builder
            .append_data(&mut header, path, *data)
            .await?;
    }
    archive_builder.finish().await?;
    let mut archive = archive_builder.into_inner().await?;
    archive.flush().await?;

    Ok(archive.into_boxed_slice())
}

#[tokio::test]
async fn pack_archive() {
    let mut f = tokio::fs::File::create("test1.tar").await.unwrap();
    f.write_all(
        &*pack(&*vec![ArchiveItem {
            path: "a.txt",
            data: "aboba".as_bytes(),
        }])
        .await
        .unwrap(),
    )
    .await
    .unwrap();
}

#[tokio::test]
async fn unpack() {
    use tokio::io::BufReader;
    let f = tokio::fs::File::open("test1.tar").await.unwrap();
    let mut buf = BufReader::new(f);
    let mut arc = Archive::new(&mut buf);
    arc.unpack("tests").await.unwrap();
}
