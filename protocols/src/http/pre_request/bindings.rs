#[allow(clippy::all)]
#[allow(dead_code)]
pub mod pre_request {
    #[allow(unused_imports)]
    use wit_bindgen_wasmtime::{anyhow, wasmtime};
    #[derive(Clone)]
    pub struct HttpPreRequest {
        pub path: String,
        pub authority: String,
        pub host: String,
        pub scheme: String,
    }
    impl std::fmt::Debug for HttpPreRequest {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("HttpPreRequest")
                .field("path", &self.path)
                .field("authority", &self.authority)
                .field("host", &self.host)
                .field("scheme", &self.scheme)
                .finish()
        }
    }
    #[repr(u8)]
    #[derive(Clone, Copy, PartialEq, Eq)]
    pub enum ProxyMode {
        Intercept,
        Pass,
    }
    impl std::fmt::Debug for ProxyMode {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                ProxyMode::Intercept => f.debug_tuple("ProxyMode::Intercept").finish(),
                ProxyMode::Pass => f.debug_tuple("ProxyMode::Pass").finish(),
            }
        }
    }
    pub trait PreRequest: Sized {
        fn http_request_get(&mut self) -> HttpPreRequest;

        fn http_set_proxy_mode(&mut self, mode: ProxyMode);
    }

    pub fn add_to_linker<T, U>(
        linker: &mut wasmtime::Linker<T>,
        get: impl Fn(&mut T) -> &mut U + Send + Sync + Copy + 'static,
    ) -> anyhow::Result<()>
    where
        U: PreRequest,
    {
        use wit_bindgen_wasmtime::rt::get_func;
        use wit_bindgen_wasmtime::rt::get_memory;
        linker.func_wrap(
            "pre-request",
            "http-request-get",
            move |mut caller: wasmtime::Caller<'_, T>, arg0: i32| {
                let func = get_func(&mut caller, "canonical_abi_realloc")?;
                let func_canonical_abi_realloc =
                    func.typed::<(i32, i32, i32, i32), i32, _>(&caller)?;
                let memory = &get_memory(&mut caller, "memory")?;
                let host = get(caller.data_mut());
                let result0 = host.http_request_get();
                let HttpPreRequest {
                    path: path1,
                    authority: authority1,
                    host: host1,
                    scheme: scheme1,
                } = result0;
                let vec2 = path1;
                let ptr2 = func_canonical_abi_realloc
                    .call(&mut caller, (0, 0, 1, (vec2.len() as i32) * 1))?;
                let caller_memory = memory.data_mut(&mut caller);
                caller_memory.store_many(ptr2, vec2.as_ref())?;
                let vec3 = authority1;
                let ptr3 = func_canonical_abi_realloc
                    .call(&mut caller, (0, 0, 1, (vec3.len() as i32) * 1))?;
                let caller_memory = memory.data_mut(&mut caller);
                caller_memory.store_many(ptr3, vec3.as_ref())?;
                let vec4 = host1;
                let ptr4 = func_canonical_abi_realloc
                    .call(&mut caller, (0, 0, 1, (vec4.len() as i32) * 1))?;
                let caller_memory = memory.data_mut(&mut caller);
                caller_memory.store_many(ptr4, vec4.as_ref())?;
                let vec5 = scheme1;
                let ptr5 = func_canonical_abi_realloc
                    .call(&mut caller, (0, 0, 1, (vec5.len() as i32) * 1))?;
                let caller_memory = memory.data_mut(&mut caller);
                caller_memory.store_many(ptr5, vec5.as_ref())?;
                caller_memory.store(
                    arg0 + 56,
                    wit_bindgen_wasmtime::rt::as_i32(vec5.len() as i32),
                )?;
                caller_memory.store(arg0 + 48, wit_bindgen_wasmtime::rt::as_i32(ptr5))?;
                caller_memory.store(
                    arg0 + 40,
                    wit_bindgen_wasmtime::rt::as_i32(vec4.len() as i32),
                )?;
                caller_memory.store(arg0 + 32, wit_bindgen_wasmtime::rt::as_i32(ptr4))?;
                caller_memory.store(
                    arg0 + 24,
                    wit_bindgen_wasmtime::rt::as_i32(vec3.len() as i32),
                )?;
                caller_memory.store(arg0 + 16, wit_bindgen_wasmtime::rt::as_i32(ptr3))?;
                caller_memory.store(
                    arg0 + 8,
                    wit_bindgen_wasmtime::rt::as_i32(vec2.len() as i32),
                )?;
                caller_memory.store(arg0 + 0, wit_bindgen_wasmtime::rt::as_i32(ptr2))?;
                Ok(())
            },
        )?;
        linker.func_wrap(
            "pre-request",
            "http-set-proxy-mode",
            move |mut caller: wasmtime::Caller<'_, T>, arg0: i32| {
                let host = get(caller.data_mut());
                let param0 = match arg0 {
                    0 => ProxyMode::Intercept,
                    1 => ProxyMode::Pass,
                    _ => return Err(invalid_variant("ProxyMode")),
                };
                host.http_set_proxy_mode(param0);
                Ok(())
            },
        )?;
        Ok(())
    }
    use wit_bindgen_wasmtime::rt::invalid_variant;
    use wit_bindgen_wasmtime::rt::RawMem;
}
