use wasi_nn::{Tensor, EXECUTION_TARGET_CPU, GRAPH_ENCODING_OPENVINO};

/// An example model used by tract.
///
/// Downloaded from [their test folder](https://github.com/sonos/tract/blob/38c6728db2469741b95b51c13114b6b68a4cb14c/tensorflow/tests/models/plus3.pb).
const MODEL: &[u8] = include_bytes!("../plus3.pb");

#[no_mangle]
pub extern "C" fn start() {
    unsafe {
        // Load the graph
        let graph_builders = &[MODEL];
        let graph = wasi_nn::load(
            graph_builders,
            GRAPH_ENCODING_OPENVINO,
            EXECUTION_TARGET_CPU,
        )
        .unwrap();

        // Then create an execution context
        let context = wasi_nn::init_execution_context(graph).unwrap();

        let input: f32 = 42.0;

        // we need to set our input
        let input_tensor = Tensor {
            data: &input.to_le_bytes(),
            dimensions: &[1],
            r#type: wasi_nn::TENSOR_TYPE_F32,
        };
        wasi_nn::set_input(context, 0, input_tensor).unwrap();

        // run inference
        wasi_nn::compute(context).unwrap();

        // then read the result back
        let mut buffer = [0; 4];
        wasi_nn::get_output(
            context,
            0,
            buffer.as_mut_ptr(),
            buffer.len() as u32,
        )
        .unwrap();

        // let's see what we got!
        let result = f32::from_le_bytes(buffer);
        println!("{} + 3 = {}", input, result);
    }
}
