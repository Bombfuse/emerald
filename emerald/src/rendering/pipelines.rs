pub mod texture_pipeline {
    use miniquad::{
        BlendFactor, BlendState, BlendValue, BufferLayout, Context, Equation, Pipeline,
        PipelineParams, Shader, VertexAttribute, VertexFormat,
    };

    use crate::{rendering::shaders, EmeraldError, FRAGMENT, VERTEX};

    pub fn create_texture_pipeline(ctx: &mut Context) -> Result<Pipeline, EmeraldError> {
        let shader = Shader::new(ctx, VERTEX, FRAGMENT, shaders::meta()).unwrap();
        let params = PipelineParams {
            depth_write: true,
            color_blend: Some(BlendState::new(
                Equation::Add,
                BlendFactor::Value(BlendValue::SourceAlpha),
                BlendFactor::OneMinusValue(BlendValue::SourceAlpha),
            )),
            alpha_blend: Some(BlendState::new(
                Equation::Add,
                BlendFactor::Zero,
                BlendFactor::One,
            )),
            ..Default::default()
        };

        let texture_pipeline = Pipeline::with_params(
            ctx,
            &[BufferLayout::default()],
            &[VertexAttribute::new("position", VertexFormat::Float2)],
            shader,
            params,
        );

        Ok(texture_pipeline)
    }
}
