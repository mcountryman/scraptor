use std::time::Instant;

use criterion::{criterion_group, criterion_main, Criterion};
use scraptor::{driver::dxgi::display::DxgiDisplays, errors::FrameError, Display, Frame};

pub fn bench(c: &mut Criterion) {
  c.bench_function("frame", |b| {
    b.iter_custom(|iters| {
      let mut displays = DxgiDisplays::new().unwrap();
      let mut display = displays.next().unwrap().unwrap();
      let time = Instant::now();

      for _ in 0..iters {
        match display.frame() {
          Err(FrameError::WouldBlock) => continue,
          Err(err) => panic!("{:?}", err),
          Ok(frame) => {
            frame.as_bytes().unwrap();
          }
        };
      }

      time.elapsed()
    });
  });
}

criterion_group!(benches, bench);
criterion_main!(benches);
