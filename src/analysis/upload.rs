use super::analysis::Analysis;

pub struct Upload;

impl Upload {
    pub async fn upload() {
        Analysis::new().await.upload().await.expect("upload");
    }
}
