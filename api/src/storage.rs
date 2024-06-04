use std::env;
use std::io::Write;
use std::path::Path;
use aws_sdk_s3 as s3;
use aws_sdk_s3::config::{Credentials, Region};
use aws_sdk_s3::config::endpoint::{Endpoint, EndpointFuture, Params, ResolveEndpoint};
use aws_sdk_s3::primitives::ByteStream;
use s3::Client;
use tempfile::NamedTempFile;

#[derive(Debug)]
struct S3EndpointResolver {
    url: String,
}
impl ResolveEndpoint for S3EndpointResolver {
    fn resolve_endpoint(&self, params: &Params) -> EndpointFuture<'_> {
        let full_url = format!("{}/{}", self.url, params.bucket().unwrap_or(""));
        EndpointFuture::ready(Ok(Endpoint::builder().url(full_url).build()))
    }
}

pub async fn create_client() -> Result<Client, &'static str> {
    let access_key = env::var("MINIO_ACCESS_KEY").expect("MINIO_ACCESS_KEY must be set");
    let secret_key = env::var("MINIO_SECRET_KEY").expect("MINIO_SECRET_KEY must be set");
    let base_url = env::var("MINIO_URL").expect("MINIO_URL must be set");

    let profile_creds = Credentials::new(
        access_key,
        secret_key,
        None,
        None,
        "Static",
    );
    let conf = s3::Config::builder()
        .behavior_version_latest()
        .region(Region::new("us-east-1"))
        .endpoint_resolver(S3EndpointResolver {
            url: base_url,
        })
        .credentials_provider(profile_creds)
        .build();

    let aws_s3_client = Client::from_conf(conf);

    Ok(aws_s3_client)
}

pub async fn save(client: &Client, bucket_name: &str, src_fp: &str, dst_name: &str) -> Result<(), &'static str> {
    client
            .put_object()
            .bucket(bucket_name.to_string())
            .key(dst_name)
            .body(ByteStream::from_path(Path::new(src_fp)).await.expect("foo"))
            .send()
            .await.expect("foo");

    Ok(())
}

#[allow(dead_code)]
pub async fn load(client: &Client, bucket_name: &str, dst: &mut NamedTempFile, src_name: &str) -> Result<(), &'static str> {
    let obj = client
            .get_object()
            .bucket(bucket_name.to_string())
            .key(src_name)
            .send()
            .await.expect("foo");

    dst.write_all(&*obj.body.collect().await.expect("foo").into_bytes()).unwrap();

    Ok(())
}

#[allow(dead_code)]
pub async fn delete(client: &Client, bucket_name: &str, name: &str) -> Result<(), &'static str> {
    client
            .delete_object()
            .bucket(bucket_name.to_string())
            .key(name)
            .send()
            .await.expect("foo");
    Ok(())
}