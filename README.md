## StereoKit
StereoKit-rs high level bindings to StereoKit

<https://StereoKit.net>

StereoKit is an easy-to-use Mixed Realty engine, designed for creating VR, AR, and XR experiences

![Alt Link](https://media.giphy.com/media/v1.Y2lkPTc5MGI3NjExYmE3MWI5ZjI5Mzk3YjFlNTVlZTM0YWEzYzYxMGJkNjY3ZjQ2YjQ4MiZlcD12MV9pbnRlcm5hbF9naWZzX2dpZklkJmN0PWc/tDPnLNOjTRio520V5s/giphy-downsized-large.gif)
![Alt Link](https://media.giphy.com/media/v1.Y2lkPTc5MGI3NjExMDA5YTBjY2FhNWEyMGNjZGI5NmI1YTRjOWRhOWNjMjI2MWZkNDYxMyZlcD12MV9pbnRlcm5hbF9naWZzX2dpZklkJmN0PWc/5MIrslIRJlBCqjP0oY/giphy-downsized-large.gif)
### Features

* Platforms: HoloLens 2, Oculus Quest, Windows Mixed Reality, Oculus Desktop, SteamVR, Varjo, Monado (Linux), and eventually everywhere OpenXR is!
* Mixed Reality inputs like hands and eyes are trivial to access
* Easy and powerful UI and interactions
* Model formats: .gltf, .glb, .obj, .stl, ASCII .ply
* Texture formats: .jpg, .png, .tga, .bmp, .psd, .gif, .hdr, .pic, .qoi, cubemaps
* Flexible shader/material system with built-in PBR
* Performance-by-default instanced render pipeline
* Skeletal/skinned animation
* Flat screen MR simulator with input emulation for easy development
* Builds your application to device in seconds, not minutes
* Runtime asset loading and cross-platform file picking
* Physics

### About
StereoKit prioritizes mixed reality application development above all else! This allows us to focus on features such as a first class mixed reality input system, fast performance by default even on mobile devices, quick iteration time on-device, and a runtime asset pipeline that lets users and developers load real assets from the file-system. All of this and more are packaged in a terse API that’s well documented, easy to learn, and easy to write.

StereoKit is ready to use, but still early in its life! Keep track on Twitter for development news and gifs, or check this blog for more substantial updates! Can’t find a feature you need for your project? Request it on the issues page, and we’ll prioritize getting you up and running!

While StereoKit is primarily intended to be consumed from C#, all core functionality is implemented in native code, and a C compatible header file is also available for C/C++ developers!
These bindings are created from that header file!

For better documentation and tutorials check out <https://stereokit.net>

### Help or Contributing
Reach out on the official StereoKit discord server if you have any questions!
<https://discord.com/invite/jtZpfS7nyK>

### Example

```rust
fn basic() {
    let sk = crate::SettingsBuilder::new()
        .app_name("StereoKit Example App")
        .init()
        .unwrap();
    let model = sk.model_create_mesh(crate::Mesh::CUBE, crate::Material::DEFAULT);
    let mut position = glam::Vec3::default();
    let mut redness = 0.0;
    sk.run(|sk| {
        position.x = sk.time_total_f32().sin();
        position.y = sk.time_total_f32().cos();
        redness = sk.time_total_f32().sin() - 0.3;
        sk.model_draw(
            &model,
            glam::Mat4::from_scale_rotation_translation(
                glam::Vec3::new(1.0, 1.0, 1.0),
                glam::Quat::IDENTITY,
                position,
            ),
            crate::Color128::new(redness, 0.1, 0.9, 1.0),
            crate::RenderLayer::default(),
        );
    }, |_| {});
}
```