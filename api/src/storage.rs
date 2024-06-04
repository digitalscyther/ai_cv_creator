use std::env;
use minio::s3::args::{BucketExistsArgs, DownloadObjectArgs, MakeBucketArgs, RemoveObjectArgs, UploadObjectArgs};
use minio::s3::client::{Client, ClientBuilder};
use minio::s3::creds::StaticProvider;
use minio::s3::http::BaseUrl;

pub async fn create_client() -> Result<Client, &'static str> {
    let access_key = env::var("MINIO_ACCESS_KEY").expect("MINIO_ACCESS_KEY must be set");
    let secret_key = env::var("MINIO_SECRET_KEY").expect("MINIO_SECRET_KEY must be set");
    let base_url = env::var("MINIO_URL").expect("MINIO_URL must be set");

    let base_url = base_url.parse::<BaseUrl>().expect("failed build minio base_url");

    let static_provider = StaticProvider::new(
        &access_key,
        &secret_key,
        None,
    );

    Ok(ClientBuilder::new(base_url.clone())
        .provider(Some(Box::new(static_provider)))
        .build().expect("Failed get minio client"))
}

pub async fn save(client: &Client, bucket_name: &str, src_fp: &str, dst_name: &str) -> Result<(), &'static str> {
    create_bucket_if_not_exists(client, bucket_name).await.expect("foo");

    let upload_obj_args: UploadObjectArgs = UploadObjectArgs::new(
        bucket_name,
        dst_name,
        src_fp
    ).unwrap();

    client.upload_object(&upload_obj_args).await.expect("Failed save minio");

    Ok(())
}

pub async fn load(client: &Client, bucket_name: &str, dst_fp: &str, src_name: &str) -> Result<(), &'static str> {
    create_bucket_if_not_exists(client, bucket_name).await?;

    let download_obj_args: DownloadObjectArgs = DownloadObjectArgs::new(
        bucket_name,
        src_name,
        dst_fp
    ).unwrap();

    client.download_object(&download_obj_args).await.expect("Failed load minio");

    Ok(())
}

pub async fn delete(client: &Client, bucket_name: &str, name: &str) -> Result<(), &'static str> {
    create_bucket_if_not_exists(client, bucket_name).await?;

    let remove_obj_args: RemoveObjectArgs = RemoveObjectArgs::new(
        bucket_name,
        name,
    ).unwrap();

    client.remove_object(&remove_obj_args).await.expect("Failed delete minio");

    Ok(())
}

async fn create_bucket_if_not_exists(client: &Client, bucket_name: &str) -> Result<(), &'static str> {
    let exists: bool = client
        .bucket_exists(&BucketExistsArgs::new(&bucket_name).unwrap())
        .await
        .unwrap();

    if !exists {
        client
            .make_bucket(&MakeBucketArgs::new(&bucket_name).unwrap())
            .await
            .unwrap();
    }

    Ok(())
}