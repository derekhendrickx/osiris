mod common;

#[tokio::test]
async fn it_works() {
    common::setup().await.unwrap();
    // assert_eq!(4, adder::add_two(2));
}
