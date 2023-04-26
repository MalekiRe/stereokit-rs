use crate::{CStereoKitDraw, CStereoKitMultiThread};

#[test]
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