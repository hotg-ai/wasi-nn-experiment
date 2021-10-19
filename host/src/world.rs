#![allow(unused_variables)]
use wiggle::GuestPtr;

use crate::{
    types::{
        BufferSize, ExecutionTarget, Graph, GraphBuilderArray, GraphEncoding,
        GraphExecutionContext, NnErrno, Tensor,
    },
    wasi_ephemeral_nn::WasiEphemeralNn,
};

#[derive(Debug, Default)]
pub struct World {
    pub(crate) wasi_nn_ctx: WasiNnCtx,
}

#[derive(Debug, Default)]
pub struct WasiNnCtx {}

impl WasiEphemeralNn for WasiNnCtx {
    fn load<'a>(
        &mut self,
        builder: &GraphBuilderArray<'a>,
        encoding: GraphEncoding,
        target: ExecutionTarget,
    ) -> Result<Graph, NnErrno> {
        todo!()
    }

    fn init_execution_context(
        &mut self,
        graph: Graph,
    ) -> Result<GraphExecutionContext, NnErrno> {
        todo!()
    }

    fn set_input<'a>(
        &mut self,
        context: GraphExecutionContext,
        index: u32,
        tensor: &Tensor<'a>,
    ) -> Result<(), NnErrno> {
        todo!()
    }

    fn get_output<'a>(
        &mut self,
        context: GraphExecutionContext,
        index: u32,
        out_buffer: &GuestPtr<'a, u8>,
        out_buffer_max_size: BufferSize,
    ) -> Result<BufferSize, NnErrno> {
        todo!()
    }

    fn compute(
        &mut self,
        context: GraphExecutionContext,
    ) -> Result<(), NnErrno> {
        todo!()
    }
}
