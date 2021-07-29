use crossterm::{cursor::MoveToColumn, ExecutableCommand};
use scraptor::{driver::dxgi::display::DxgiDisplays, errors::FrameError, Display};
use std::{io::stdout, time::Instant};

fn main() -> anyhow::Result<()> {
  let mut stdout = stdout();

  // capture
  let mut displays = DxgiDisplays::new().unwrap();
  let mut display = displays.next().unwrap().unwrap();

  // encode
  // ...

  // time
  let mut time = Instant::now();
  let mut total = Instant::now();

  // fps
  let mut fps = 0.0;
  let mut fps_max = f32::MIN;
  let mut fps_min = f32::MAX;
  let fps_factor = 0.9;

  println!("-------------------------");

  loop {
    if total.elapsed().as_secs() > 30 {
      return Ok(());
    }

    match display.frame() {
      Err(FrameError::WouldBlock) => continue,
      Err(err) => panic!("{:?}", err),
      Ok(frame) => {
        frame.as_bytes().unwrap();

        let elapsed = time.elapsed();
        let secs = elapsed.as_secs_f32();
        let ms = elapsed.as_nanos() as f32 / 1000000.0;

        fps = (fps * fps_factor) + ((1.0 / secs) * (1.0 - fps_factor));
        fps_min = fps_min.min(fps);
        fps_max = fps_max.max(fps);

        stdout.execute(MoveToColumn(0))?;

        print!("*");
        print!("time: {:.2}ms ", ms);
        print!("fps: {:.2} ", fps);
        print!("fps_min: {:.2} ", fps_min);
        print!("fps_max: {:.2} ", fps_max);

        time = Instant::now();
      }
    };
  }
}
