#![allow(unused_variables)]

use crate::{
    types::{
        BufferSize, ExecutionTarget, Graph, GraphBuilderArray, GraphEncoding,
        GraphExecutionContext, NnErrno, Tensor, TensorType,
    },
    wasi_ephemeral_nn::WasiEphemeralNn,
};
use std::{collections::HashMap, io::Cursor, sync::Arc};
use tract_core::{
    framework::Framework,
    prelude::{DatumType, TVec, Tensor as TractTensor},
};
use tract_tensorflow::prelude::{InferenceModel, InferenceSimplePlan};
use wiggle::{GuestPtr, GuestSlice};

#[derive(Debug, Default)]
pub struct World {
    pub(crate) wasi_nn_ctx: WasiNnCtx,
}

#[derive(Debug, Default)]
pub struct WasiNnCtx {
    models: HashMap<GraphExecutionContext, ExecutionContext>,
    graphs: HashMap<Graph, InferenceModel>,
    last_id: u32,
}

impl WasiNnCtx {
    fn next_id<T>(&mut self) -> T
    where
        T: From<u32>,
    {
        self.last_id += 1;
        self.last_id.into()
    }
}

impl WasiEphemeralNn for WasiNnCtx {
    fn load<'a>(
        &mut self,
        builders: &GraphBuilderArray<'a>,
        encoding: GraphEncoding,
        target: ExecutionTarget,
    ) -> Result<Graph, NnErrno> {
        assert_eq!(encoding, GraphEncoding::Openvino);
        assert_eq!(target, ExecutionTarget::Cpu);

        let model: GuestSlice<u8> =
            builders.as_ptr().read().unwrap().as_slice().unwrap();
        let mut reader = Cursor::new(model.as_ref());

        let graph = tract_tensorflow::tensorflow()
            .model_for_read(&mut reader)
            .unwrap();

        log::info!("Loaded the model:\n{}", graph);

        let id = self.next_id();
        self.graphs.insert(id, graph);

        Ok(id)
    }

    fn init_execution_context(
        &mut self,
        graph: Graph,
    ) -> Result<GraphExecutionContext, NnErrno> {
        let runnable_graph =
            self.graphs.remove(&graph).unwrap().into_runnable().unwrap();

        let id = self.next_id();
        log::info!("Converted {:?} to {:?}", graph, id);

        self.models.insert(
            id,
            ExecutionContext {
                runnable_graph,
                inputs: TVec::default(),
                outputs: TVec::default(),
            },
        );

        Ok(id)
    }

    fn set_input<'a>(
        &mut self,
        context: GraphExecutionContext,
        index: u32,
        tensor: &Tensor<'a>,
    ) -> Result<(), NnErrno> {
        let ctx = self.models.get_mut(&context).unwrap();

        let Tensor {
            dimensions,
            type_,
            data,
        } = tensor;

        let shape = dimensions.as_slice().unwrap();
        let shape: Vec<usize> = shape.iter().map(|&s| s as usize).collect();

        let content = data.as_slice().unwrap();

        let dt = match type_ {
            TensorType::F16 => DatumType::F16,
            TensorType::F32 => DatumType::F32,
            TensorType::U8 => DatumType::U8,
            TensorType::I32 => DatumType::I32,
        };

        let index = index as usize;

        if ctx.inputs.len() < index + 1 {
            ctx.inputs.resize_with(index + 1, Default::default);
        }

        // This function doesn't actually specify why it is "unsafe", but I'm
        // assuming it's just about ensuring our bytes are valid for the
        // underlying data type.
        let tensor =
            unsafe { TractTensor::from_raw_dt(dt, &shape, &content).unwrap() };
        log::info!("Setting input {} on {:?} to {:?}", index, context, tensor);
        ctx.inputs[index] = tensor;

        Ok(())
    }

    fn get_output<'a>(
        &mut self,
        context: GraphExecutionContext,
        index: u32,
        out_buffer: &GuestPtr<'a, u8>,
        out_buffer_max_size: BufferSize,
    ) -> Result<BufferSize, NnErrno> {
        let ctx = self.models.get(&context).unwrap();
        let output_tensor = ctx.outputs.get(index as usize).unwrap();

        let output = unsafe { output_tensor.as_bytes() };

        let output_length = output.len() as u32;
        assert!(out_buffer_max_size >= output_length);

        out_buffer
            .as_array(output_length)
            .as_slice_mut()
            .unwrap()
            .copy_from_slice(output);

        log::info!(
            "Read output {} from {:?}: {:?}",
            index,
            context,
            output_tensor
        );

        Ok(output_length)
    }

    fn compute(
        &mut self,
        context: GraphExecutionContext,
    ) -> Result<(), NnErrno> {
        let ExecutionContext {
            runnable_graph,
            inputs,
            outputs,
        } = self.models.get_mut(&context).unwrap();

        let inputs = std::mem::take(inputs);
        log::info!("Running {:?} with {:?} ", context, inputs,);

        *outputs = runnable_graph.run(inputs).unwrap();

        log::info!("The result is {:?}", outputs);

        Ok(())
    }
}

#[derive(Debug)]
struct ExecutionContext {
    runnable_graph: InferenceSimplePlan<InferenceModel>,
    inputs: TVec<TractTensor>,
    outputs: TVec<Arc<TractTensor>>,
}
