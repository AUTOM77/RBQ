use arrow::compute::kernels::cmp::eq;
use arrow::util::pretty::print_batches;
use arrow_array::{Int32Array, Scalar};
use futures::TryStreamExt;
use parquet::arrow::arrow_reader::{ArrowPredicateFn, RowFilter};
use parquet::arrow::{ParquetRecordBatchStreamBuilder, ProjectionMask};
use parquet::errors::Result;
use std::time::SystemTime;
use tokio::fs::File;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let testdata = arrow::util::test_util::parquet_test_data();
    let path = format!("/dev/shm/000001.parquet");
    let file = File::open(path).await.unwrap();

    let mut builder = ParquetRecordBatchStreamBuilder::new(file)
        .await
        .unwrap()
        .with_batch_size(8192);

    let file_metadata = builder.metadata().file_metadata().clone();
    let mask = ProjectionMask::roots(file_metadata.schema_descr(), [0, 1, 2]);

    builder = builder.with_projection(mask);

    let scalar = Int32Array::from(vec![1]);
    let filter = ArrowPredicateFn::new(
        ProjectionMask::roots(file_metadata.schema_descr(), [0]),
        move |record_batch| eq(record_batch.column(0), &Scalar::new(&scalar)),
    );
    let row_filter = RowFilter::new(vec![Box::new(filter)]);
    builder = builder.with_row_filter(row_filter);

    // Build a async parquet reader.
    let stream = builder.build().unwrap();

    let start = SystemTime::now();

    let result = stream.try_collect::<Vec<_>>().await?;

    println!("took: {} ms", start.elapsed().unwrap().as_millis());

    print_batches(&result).unwrap();

    Ok(())
}