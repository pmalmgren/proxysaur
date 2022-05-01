pub mod config {
    #[allow(unused_imports)]
    use wit_bindgen_wasmtime::{anyhow, wasmtime};
    pub trait Config: Sized {
        fn get_config_data(&mut self) -> Vec<u8>;

        fn set_invalid_data(&mut self, error: &str);
    }

    pub fn add_to_linker<T, U>(
        linker: &mut wasmtime::Linker<T>,
        get: impl Fn(&mut T) -> &mut U + Send + Sync + Copy + 'static,
    ) -> anyhow::Result<()>
    where
        U: Config,
    {
        use wit_bindgen_wasmtime::rt::get_func;
        use wit_bindgen_wasmtime::rt::get_memory;
        linker.func_wrap(
            "config",
            "get-config-data",
            move |mut caller: wasmtime::Caller<'_, T>, arg0: i32| {
                let func = get_func(&mut caller, "canonical_abi_realloc")?;
                let func_canonical_abi_realloc =
                    func.typed::<(i32, i32, i32, i32), i32, _>(&caller)?;
                let memory = &get_memory(&mut caller, "memory")?;
                let host = get(caller.data_mut());
                let result0 = host.get_config_data();
                let vec1 = result0;
                let ptr1 = func_canonical_abi_realloc
                    .call(&mut caller, (0, 0, 1, (vec1.len() as i32) * 1))?;
                let caller_memory = memory.data_mut(&mut caller);
                caller_memory.store_many(ptr1, vec1.as_ref())?;
                caller_memory.store(
                    arg0 + 8,
                    wit_bindgen_wasmtime::rt::as_i32(vec1.len() as i32),
                )?;
                caller_memory.store(arg0 + 0, wit_bindgen_wasmtime::rt::as_i32(ptr1))?;
                Ok(())
            },
        )?;
        linker.func_wrap(
            "config",
            "set-invalid-data",
            move |mut caller: wasmtime::Caller<'_, T>, arg0: i32, arg1: i32| {
                let memory = &get_memory(&mut caller, "memory")?;
                let (mem, data) = memory.data_and_store_mut(&mut caller);
                let mut _bc = wit_bindgen_wasmtime::BorrowChecker::new(mem);
                let host = get(data);
                let ptr0 = arg0;
                let len0 = arg1;
                let param0 = _bc.slice_str(ptr0, len0)?;
                host.set_invalid_data(param0);
                Ok(())
            },
        )?;
        Ok(())
    }
    use wit_bindgen_wasmtime::rt::RawMem;
}
