use stereokit::Settings;

fn main() {
  let stereokit = Settings::default().init().unwrap();
	stereokit.run(|_, _| {}, |_| {});
}