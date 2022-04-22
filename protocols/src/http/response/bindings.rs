#[allow(clippy::all)]
#[allow(dead_code)]
pub mod response {
    #[allow(unused_imports)]
    use wit_bindgen_wasmtime::{anyhow, wasmtime};
    pub type BodyParam<'a> = &'a [u8];
    pub type BodyResult = Vec<u8>;
    pub type Error = String;
    pub type HttpHeaders = Vec<(String, String)>;
    #[derive(Clone)]
    pub struct HttpResponse {
        pub headers: HttpHeaders,
        pub status: u16,
        pub body: BodyResult,
    }
    impl std::fmt::Debug for HttpResponse {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("HttpResponse")
                .field("headers", &self.headers)
                .field("status", &self.status)
                .field("body", &self.body)
                .finish()
        }
    }
    pub trait Response: Sized {
        fn http_response_get(&mut self) -> Result<HttpResponse, Error>;

        fn http_response_set_status(&mut self, status: u16) -> Result<(), Error>;

        fn http_response_set_body(&mut self, body: BodyParam<'_>) -> Result<(), Error>;

        fn http_response_set_header(&mut self, header: &str, value: &str) -> Result<(), Error>;

        fn http_response_rm_header(&mut self, header: &str) -> Result<(), Error>;
    }

    pub fn add_to_linker<T, U>(
        linker: &mut wasmtime::Linker<T>,
        get: impl Fn(&mut T) -> &mut U + Send + Sync + Copy + 'static,
    ) -> anyhow::Result<()>
    where
        U: Response,
    {
        use wit_bindgen_wasmtime::rt::get_func;
        use wit_bindgen_wasmtime::rt::get_memory;
        linker.func_wrap(
            "response",
            "http-response-get",
            move |mut caller: wasmtime::Caller<'_, T>, arg0: i32| {
                let func = get_func(&mut caller, "canonical_abi_realloc")?;
                let func_canonical_abi_realloc =
                    func.typed::<(i32, i32, i32, i32), i32, _>(&caller)?;
                let memory = &get_memory(&mut caller, "memory")?;
                let host = get(caller.data_mut());
                let result0 = host.http_response_get();
                let (result8_0, result8_1, result8_2, result8_3, result8_4, result8_5) =
                    match result0 {
                        Ok(e) => {
                            let HttpResponse {
                                headers: headers1,
                                status: status1,
                                body: body1,
                            } = e;
                            let vec5 = headers1;
                            let len5 = vec5.len() as i32;
                            let result5 = func_canonical_abi_realloc
                                .call(&mut caller, (0, 0, 4, len5 * 16))?;
                            for (i, e) in vec5.into_iter().enumerate() {
                                let base = result5 + (i as i32) * 16;
                                {
                                    let (t2_0, t2_1) = e;
                                    let vec3 = t2_0;
                                    let ptr3 = func_canonical_abi_realloc
                                        .call(&mut caller, (0, 0, 1, (vec3.len() as i32) * 1))?;
                                    let caller_memory = memory.data_mut(&mut caller);
                                    caller_memory.store_many(ptr3, vec3.as_ref())?;
                                    caller_memory.store(
                                        base + 4,
                                        wit_bindgen_wasmtime::rt::as_i32(vec3.len() as i32),
                                    )?;
                                    caller_memory
                                        .store(base + 0, wit_bindgen_wasmtime::rt::as_i32(ptr3))?;
                                    let vec4 = t2_1;
                                    let ptr4 = func_canonical_abi_realloc
                                        .call(&mut caller, (0, 0, 1, (vec4.len() as i32) * 1))?;
                                    let caller_memory = memory.data_mut(&mut caller);
                                    caller_memory.store_many(ptr4, vec4.as_ref())?;
                                    caller_memory.store(
                                        base + 12,
                                        wit_bindgen_wasmtime::rt::as_i32(vec4.len() as i32),
                                    )?;
                                    caller_memory
                                        .store(base + 8, wit_bindgen_wasmtime::rt::as_i32(ptr4))?;
                                }
                            }
                            let vec6 = body1;
                            let ptr6 = func_canonical_abi_realloc
                                .call(&mut caller, (0, 0, 1, (vec6.len() as i32) * 1))?;
                            let caller_memory = memory.data_mut(&mut caller);
                            caller_memory.store_many(ptr6, vec6.as_ref())?;
                            (
                                0i32,
                                result5,
                                len5,
                                wit_bindgen_wasmtime::rt::as_i32(status1),
                                ptr6,
                                vec6.len() as i32,
                            )
                        }
                        Err(e) => {
                            let vec7 = e;
                            let ptr7 = func_canonical_abi_realloc
                                .call(&mut caller, (0, 0, 1, (vec7.len() as i32) * 1))?;
                            let caller_memory = memory.data_mut(&mut caller);
                            caller_memory.store_many(ptr7, vec7.as_ref())?;
                            (1i32, ptr7, vec7.len() as i32, 0i32, 0i32, 0i32)
                        }
                    };
                let caller_memory = memory.data_mut(&mut caller);
                caller_memory.store(arg0 + 40, wit_bindgen_wasmtime::rt::as_i32(result8_5))?;
                caller_memory.store(arg0 + 32, wit_bindgen_wasmtime::rt::as_i32(result8_4))?;
                caller_memory.store(arg0 + 24, wit_bindgen_wasmtime::rt::as_i32(result8_3))?;
                caller_memory.store(arg0 + 16, wit_bindgen_wasmtime::rt::as_i32(result8_2))?;
                caller_memory.store(arg0 + 8, wit_bindgen_wasmtime::rt::as_i32(result8_1))?;
                caller_memory.store(arg0 + 0, wit_bindgen_wasmtime::rt::as_i32(result8_0))?;
                Ok(())
            },
        )?;
        linker.func_wrap(
            "response",
            "http-response-set-status",
            move |mut caller: wasmtime::Caller<'_, T>, arg0: i32, arg1: i32| {
                let func = get_func(&mut caller, "canonical_abi_realloc")?;
                let func_canonical_abi_realloc =
                    func.typed::<(i32, i32, i32, i32), i32, _>(&caller)?;
                let memory = &get_memory(&mut caller, "memory")?;
                let host = get(caller.data_mut());
                let param0 = u16::try_from(arg0).map_err(bad_int)?;
                let result0 = host.http_response_set_status(param0);
                let (result2_0, result2_1, result2_2) = match result0 {
                    Ok(()) => (0i32, 0i32, 0i32),
                    Err(e) => {
                        let vec1 = e;
                        let ptr1 = func_canonical_abi_realloc
                            .call(&mut caller, (0, 0, 1, (vec1.len() as i32) * 1))?;
                        let caller_memory = memory.data_mut(&mut caller);
                        caller_memory.store_many(ptr1, vec1.as_ref())?;
                        (1i32, ptr1, vec1.len() as i32)
                    }
                };
                let caller_memory = memory.data_mut(&mut caller);
                caller_memory.store(arg1 + 16, wit_bindgen_wasmtime::rt::as_i32(result2_2))?;
                caller_memory.store(arg1 + 8, wit_bindgen_wasmtime::rt::as_i32(result2_1))?;
                caller_memory.store(arg1 + 0, wit_bindgen_wasmtime::rt::as_i32(result2_0))?;
                Ok(())
            },
        )?;
        linker.func_wrap(
            "response",
            "http-response-set-body",
            move |mut caller: wasmtime::Caller<'_, T>, arg0: i32, arg1: i32, arg2: i32| {
                let func = get_func(&mut caller, "canonical_abi_realloc")?;
                let func_canonical_abi_realloc =
                    func.typed::<(i32, i32, i32, i32), i32, _>(&caller)?;
                let memory = &get_memory(&mut caller, "memory")?;
                let (mem, data) = memory.data_and_store_mut(&mut caller);
                let mut _bc = wit_bindgen_wasmtime::BorrowChecker::new(mem);
                let host = get(data);
                let ptr0 = arg0;
                let len0 = arg1;
                let param0 = _bc.slice(ptr0, len0)?;
                let result1 = host.http_response_set_body(param0);
                let (result3_0, result3_1, result3_2) = match result1 {
                    Ok(()) => (0i32, 0i32, 0i32),
                    Err(e) => {
                        let vec2 = e;
                        let ptr2 = func_canonical_abi_realloc
                            .call(&mut caller, (0, 0, 1, (vec2.len() as i32) * 1))?;
                        let caller_memory = memory.data_mut(&mut caller);
                        caller_memory.store_many(ptr2, vec2.as_ref())?;
                        (1i32, ptr2, vec2.len() as i32)
                    }
                };
                let caller_memory = memory.data_mut(&mut caller);
                caller_memory.store(arg2 + 16, wit_bindgen_wasmtime::rt::as_i32(result3_2))?;
                caller_memory.store(arg2 + 8, wit_bindgen_wasmtime::rt::as_i32(result3_1))?;
                caller_memory.store(arg2 + 0, wit_bindgen_wasmtime::rt::as_i32(result3_0))?;
                Ok(())
            },
        )?;
        linker.func_wrap(
            "response",
            "http-response-set-header",
            move |mut caller: wasmtime::Caller<'_, T>,
                  arg0: i32,
                  arg1: i32,
                  arg2: i32,
                  arg3: i32,
                  arg4: i32| {
                let func = get_func(&mut caller, "canonical_abi_realloc")?;
                let func_canonical_abi_realloc =
                    func.typed::<(i32, i32, i32, i32), i32, _>(&caller)?;
                let memory = &get_memory(&mut caller, "memory")?;
                let (mem, data) = memory.data_and_store_mut(&mut caller);
                let mut _bc = wit_bindgen_wasmtime::BorrowChecker::new(mem);
                let host = get(data);
                let ptr0 = arg0;
                let len0 = arg1;
                let ptr1 = arg2;
                let len1 = arg3;
                let param0 = _bc.slice_str(ptr0, len0)?;
                let param1 = _bc.slice_str(ptr1, len1)?;
                let result2 = host.http_response_set_header(param0, param1);
                let (result4_0, result4_1, result4_2) = match result2 {
                    Ok(()) => (0i32, 0i32, 0i32),
                    Err(e) => {
                        let vec3 = e;
                        let ptr3 = func_canonical_abi_realloc
                            .call(&mut caller, (0, 0, 1, (vec3.len() as i32) * 1))?;
                        let caller_memory = memory.data_mut(&mut caller);
                        caller_memory.store_many(ptr3, vec3.as_ref())?;
                        (1i32, ptr3, vec3.len() as i32)
                    }
                };
                let caller_memory = memory.data_mut(&mut caller);
                caller_memory.store(arg4 + 16, wit_bindgen_wasmtime::rt::as_i32(result4_2))?;
                caller_memory.store(arg4 + 8, wit_bindgen_wasmtime::rt::as_i32(result4_1))?;
                caller_memory.store(arg4 + 0, wit_bindgen_wasmtime::rt::as_i32(result4_0))?;
                Ok(())
            },
        )?;
        linker.func_wrap(
            "response",
            "http-response-rm-header",
            move |mut caller: wasmtime::Caller<'_, T>, arg0: i32, arg1: i32, arg2: i32| {
                let func = get_func(&mut caller, "canonical_abi_realloc")?;
                let func_canonical_abi_realloc =
                    func.typed::<(i32, i32, i32, i32), i32, _>(&caller)?;
                let memory = &get_memory(&mut caller, "memory")?;
                let (mem, data) = memory.data_and_store_mut(&mut caller);
                let mut _bc = wit_bindgen_wasmtime::BorrowChecker::new(mem);
                let host = get(data);
                let ptr0 = arg0;
                let len0 = arg1;
                let param0 = _bc.slice_str(ptr0, len0)?;
                let result1 = host.http_response_rm_header(param0);
                let (result3_0, result3_1, result3_2) = match result1 {
                    Ok(()) => (0i32, 0i32, 0i32),
                    Err(e) => {
                        let vec2 = e;
                        let ptr2 = func_canonical_abi_realloc
                            .call(&mut caller, (0, 0, 1, (vec2.len() as i32) * 1))?;
                        let caller_memory = memory.data_mut(&mut caller);
                        caller_memory.store_many(ptr2, vec2.as_ref())?;
                        (1i32, ptr2, vec2.len() as i32)
                    }
                };
                let caller_memory = memory.data_mut(&mut caller);
                caller_memory.store(arg2 + 16, wit_bindgen_wasmtime::rt::as_i32(result3_2))?;
                caller_memory.store(arg2 + 8, wit_bindgen_wasmtime::rt::as_i32(result3_1))?;
                caller_memory.store(arg2 + 0, wit_bindgen_wasmtime::rt::as_i32(result3_0))?;
                Ok(())
            },
        )?;
        Ok(())
    }
    use core::convert::TryFrom;
    use wit_bindgen_wasmtime::rt::bad_int;
    use wit_bindgen_wasmtime::rt::RawMem;
}
