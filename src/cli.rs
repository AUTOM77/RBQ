use arrow::compute::kernels::cmp::eq;
use arrow::util::pretty::print_batches;
use futures::TryStreamExt;
use parquet::arrow::arrow_reader::{ArrowPredicateFn, RowFilter};
use parquet::arrow::{ParquetRecordBatchStreamBuilder, ProjectionMask};
use parquet::errors::Result;
use tokio::fs::File;

#[tokio::main]
async fn main() -> Result<()> {
    let start_time = std::time::Instant::now();

    let path = format!("/dev/shm/pixel/000001.parquet");
    let file = File::open(path).await.unwrap();

    let mut builder = ParquetRecordBatchStreamBuilder::new(file).await.unwrap().with_batch_size(1);

    let file_metadata = builder.metadata().file_metadata().clone();
    let mask = ProjectionMask::roots(file_metadata.schema_descr(), [6]);

    builder = builder.with_projection(mask);

    let stream = builder.build().unwrap();

    let result = stream.try_collect::<Vec<_>>().await?;

    let elapsed_time = start_time.elapsed();
    println!("Processing time: {:?}", elapsed_time);
    Ok(())
}