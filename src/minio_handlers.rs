use std::sync::Arc;
use axum::{
    extract::Extension,
    http::StatusCode,
    response::IntoResponse,
};
use minio::s3::args::{BucketExistsArgs, MakeBucketArgs, UploadObjectArgs};

use crate::minio_client::MinioClient;

pub async fn upload_file(
    Extension(minio): Extension<Arc<MinioClient>>
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let bucket_name = "newbucket";
    
    // Check 'newbucket' bucket exist or not.
    let exists = minio.client
        .bucket_exists(&BucketExistsArgs::new(&bucket_name).unwrap())
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Make 'newbucket' bucket if not exist.
    if !exists {
        minio.client
            .make_bucket(&MakeBucketArgs::new(&bucket_name).unwrap())
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    }

    minio.client.upload_object(
        &UploadObjectArgs::new(
            &bucket_name, 
            "test.png", 
            "/Users/ighfarhasbiash/Desktop/test.png")
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        )
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Return a successful response
    Ok("Berhasil upload")
}

pub async fn cocoba(minio: Extension<Arc<MinioClient>>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("ini cocoba bro");
    let bucket_name = "newbucket2";
    
    // Check 'newbucket' bucket exist or not.
    let exists = minio.client
        .bucket_exists(&BucketExistsArgs::new(&bucket_name).unwrap())
        .await
        .unwrap();

    // Make 'newbucket' bucket if not exist.
    if !exists {
        minio.client
            .make_bucket(&MakeBucketArgs::new(&bucket_name).unwrap())
            .await
            .unwrap();
    }

    minio.client
        .upload_object(
            &UploadObjectArgs::new(
                bucket_name, 
                "test.png", 
                "/Users/ighfarhasbiash/Documents/rust/axum_learn_duckdb/test.png")
            .unwrap()
    )
    .await
    .unwrap();

    // Ok(result)

    Ok(())
}
