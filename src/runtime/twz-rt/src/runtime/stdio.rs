use twizzler_runtime_api::RustStdioRuntime;

use super::ReferenceRuntime;

impl RustStdioRuntime for ReferenceRuntime {
    fn with_panic_output(&self, _cb: twizzler_runtime_api::IoWriteDynCallback<'_, ()>) {
        todo!()
    }

    fn with_stdin(
        &self,
        _cb: twizzler_runtime_api::IoReadDynCallback<
            '_,
            Result<usize, twizzler_runtime_api::ReadError>,
        >,
    ) -> Result<usize, twizzler_runtime_api::ReadError> {
        todo!()
    }

    fn with_stdout(
        &self,
        _cb: twizzler_runtime_api::IoWriteDynCallback<
            '_,
            Result<usize, twizzler_runtime_api::WriteError>,
        >,
    ) -> Result<usize, twizzler_runtime_api::WriteError> {
        todo!()
    }

    fn with_stderr(
        &self,
        _cb: twizzler_runtime_api::IoWriteDynCallback<
            '_,
            Result<usize, twizzler_runtime_api::WriteError>,
        >,
    ) -> Result<usize, twizzler_runtime_api::WriteError> {
        todo!()
    }
}
