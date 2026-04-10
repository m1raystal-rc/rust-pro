@vertex
fn main(@location(0) vertices: vec3<f32>) -> @builtin(position) vec4<f32> {
    return vec4(vertices/3.0,1.0);
}