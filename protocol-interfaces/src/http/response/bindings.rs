#![allow(warnings)]
#![allow(clippy)]
#![allow(unknown_lints)]

#[allow(dead_code)]
pub mod response {
    #[allow(unused_imports)]
    use wit_bindgen_wasmtime::{anyhow, wasmtime};
    pub type BodyParam<'a> = &'a [u8];
    pub type BodyResult = Vec<u8>;
    pub type Error = String;
    pub type HttpHeadersParam<'a> = Vec<(&'a str, &'a str)>;
    pub type HttpHeadersResult = Vec<(String, String)>;
    #[derive(Clone)]
    pub struct HttpResponseParam<'a> {
        pub headers: HttpHeadersParam<'a>,
        pub status: u16,
    }
    impl<'a> std::fmt::Debug for HttpResponseParam<'a> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("HttpResponseParam")
                .field("headers", &self.headers)
                .field("status", &self.status)
                .finish()
        }
    }
    #[derive(Clone)]
    pub struct HttpResponseResult {
        pub headers: HttpHeadersResult,
        pub status: u16,
    }
    impl std::fmt::Debug for HttpResponseResult {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("HttpResponseResult")
                .field("headers", &self.headers)
                .field("status", &self.status)
                .finish()
        }
    }
    #[wit_bindgen_wasmtime::async_trait]
    pub trait Response: Sized + Send {
        async fn http_response_get(&mut self) -> Result<HttpResponseResult, Error>;

        async fn http_response_set(&mut self, resp: HttpResponseParam<'_>) -> Result<(), Error>;

        async fn http_response_body_get(&mut self) -> Result<BodyResult, Error>;

        async fn http_response_body_set(&mut self, body: BodyParam<'_>) -> Result<(), Error>;
    }

    pub fn add_to_linker<T, U>(
        linker: &mut wasmtime::Linker<T>,
        get: impl Fn(&mut T) -> &mut U + Send + Sync + Copy + 'static,
    ) -> anyhow::Result<()>
    where
        U: Response,
        T: Send,
    {
        use wit_bindgen_wasmtime::rt::get_func;
        use wit_bindgen_wasmtime::rt::get_memory;
        linker.func_wrap1_async(
            "response",
            "http-response-get",
            move |mut caller: wasmtime::Caller<'_, T>, arg0: i32| {
                Box::new(async move {
                    let func = get_func(&mut caller, "canonical_abi_realloc")?;
                    let func_canonical_abi_realloc =
                        func.typed::<(i32, i32, i32, i32), i32, _>(&caller)?;
                    let memory = &get_memory(&mut caller, "memory")?;
                    let host = get(caller.data_mut());
                    let result0 = host.http_response_get().await;
                    let (result7_0, result7_1, result7_2, result7_3) = match result0 {
                        Ok(e) => {
                            let HttpResponseResult {
                                headers: headers1,
                                status: status1,
                            } = e;
                            let vec5 = headers1;
                            let len5 = vec5.len() as i32;
                            let result5 = func_canonical_abi_realloc
                                .call_async(&mut caller, (0, 0, 4, len5 * 16))
                                .await?;
                            for (i, e) in vec5.into_iter().enumerate() {
                                let base = result5 + (i as i32) * 16;
                                {
                                    let (t2_0, t2_1) = e;
                                    let vec3 = t2_0;
                                    let ptr3 = func_canonical_abi_realloc
                                        .call_async(&mut caller, (0, 0, 1, (vec3.len() as i32) * 1))
                                        .await?;
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
                                        .call_async(&mut caller, (0, 0, 1, (vec4.len() as i32) * 1))
                                        .await?;
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
                            (
                                0i32,
                                result5,
                                len5,
                                wit_bindgen_wasmtime::rt::as_i32(status1),
                            )
                        }
                        Err(e) => {
                            let vec6 = e;
                            let ptr6 = func_canonical_abi_realloc
                                .call_async(&mut caller, (0, 0, 1, (vec6.len() as i32) * 1))
                                .await?;
                            let caller_memory = memory.data_mut(&mut caller);
                            caller_memory.store_many(ptr6, vec6.as_ref())?;
                            (1i32, ptr6, vec6.len() as i32, 0i32)
                        }
                    };
                    let caller_memory = memory.data_mut(&mut caller);
                    caller_memory.store(arg0 + 24, wit_bindgen_wasmtime::rt::as_i32(result7_3))?;
                    caller_memory.store(arg0 + 16, wit_bindgen_wasmtime::rt::as_i32(result7_2))?;
                    caller_memory.store(arg0 + 8, wit_bindgen_wasmtime::rt::as_i32(result7_1))?;
                    caller_memory.store(arg0 + 0, wit_bindgen_wasmtime::rt::as_i32(result7_0))?;
                    Ok(())
                })
            },
        )?;
        linker.func_wrap4_async(
            "response",
            "http-response-set",
            move |mut caller: wasmtime::Caller<'_, T>,
                  arg0: i32,
                  arg1: i32,
                  arg2: i32,
                  arg3: i32| {
                Box::new(async move {
                    let func = get_func(&mut caller, "canonical_abi_realloc")?;
                    let func_canonical_abi_realloc =
                        func.typed::<(i32, i32, i32, i32), i32, _>(&caller)?;
                    let memory = &get_memory(&mut caller, "memory")?;
                    let (mem, data) = memory.data_and_store_mut(&mut caller);
                    let mut _bc = wit_bindgen_wasmtime::BorrowChecker::new(mem);
                    let host = get(data);
                    let len6 = arg1;
                    let base6 = arg0;
                    let mut result6 = Vec::with_capacity(len6 as usize);
                    for i in 0..len6 {
                        let base = base6 + i * 16;
                        result6.push({
                            let load0 = _bc.load::<i32>(base + 0)?;
                            let load1 = _bc.load::<i32>(base + 4)?;
                            let ptr2 = load0;
                            let len2 = load1;
                            let load3 = _bc.load::<i32>(base + 8)?;
                            let load4 = _bc.load::<i32>(base + 12)?;
                            let ptr5 = load3;
                            let len5 = load4;
                            (_bc.slice_str(ptr2, len2)?, _bc.slice_str(ptr5, len5)?)
                        });
                    }
                    let param0 = HttpResponseParam {
                        headers: result6,
                        status: u16::try_from(arg2).map_err(bad_int)?,
                    };
                    let result7 = host.http_response_set(param0).await;
                    let (result9_0, result9_1, result9_2) = match result7 {
                        Ok(()) => (0i32, 0i32, 0i32),
                        Err(e) => {
                            let vec8 = e;
                            let ptr8 = func_canonical_abi_realloc
                                .call_async(&mut caller, (0, 0, 1, (vec8.len() as i32) * 1))
                                .await?;
                            let caller_memory = memory.data_mut(&mut caller);
                            caller_memory.store_many(ptr8, vec8.as_ref())?;
                            (1i32, ptr8, vec8.len() as i32)
                        }
                    };
                    let caller_memory = memory.data_mut(&mut caller);
                    caller_memory.store(arg3 + 16, wit_bindgen_wasmtime::rt::as_i32(result9_2))?;
                    caller_memory.store(arg3 + 8, wit_bindgen_wasmtime::rt::as_i32(result9_1))?;
                    caller_memory.store(arg3 + 0, wit_bindgen_wasmtime::rt::as_i32(result9_0))?;
                    Ok(())
                })
            },
        )?;
        linker.func_wrap1_async(
            "response",
            "http-response-body-get",
            move |mut caller: wasmtime::Caller<'_, T>, arg0: i32| {
                Box::new(async move {
                    let func = get_func(&mut caller, "canonical_abi_realloc")?;
                    let func_canonical_abi_realloc =
                        func.typed::<(i32, i32, i32, i32), i32, _>(&caller)?;
                    let memory = &get_memory(&mut caller, "memory")?;
                    let host = get(caller.data_mut());
                    let result0 = host.http_response_body_get().await;
                    let (result3_0, result3_1, result3_2) = match result0 {
                        Ok(e) => {
                            let vec1 = e;
                            let ptr1 = func_canonical_abi_realloc
                                .call_async(&mut caller, (0, 0, 1, (vec1.len() as i32) * 1))
                                .await?;
                            let caller_memory = memory.data_mut(&mut caller);
                            caller_memory.store_many(ptr1, vec1.as_ref())?;
                            (0i32, ptr1, vec1.len() as i32)
                        }
                        Err(e) => {
                            let vec2 = e;
                            let ptr2 = func_canonical_abi_realloc
                                .call_async(&mut caller, (0, 0, 1, (vec2.len() as i32) * 1))
                                .await?;
                            let caller_memory = memory.data_mut(&mut caller);
                            caller_memory.store_many(ptr2, vec2.as_ref())?;
                            (1i32, ptr2, vec2.len() as i32)
                        }
                    };
                    let caller_memory = memory.data_mut(&mut caller);
                    caller_memory.store(arg0 + 16, wit_bindgen_wasmtime::rt::as_i32(result3_2))?;
                    caller_memory.store(arg0 + 8, wit_bindgen_wasmtime::rt::as_i32(result3_1))?;
                    caller_memory.store(arg0 + 0, wit_bindgen_wasmtime::rt::as_i32(result3_0))?;
                    Ok(())
                })
            },
        )?;
        linker.func_wrap3_async(
            "response",
            "http-response-body-set",
            move |mut caller: wasmtime::Caller<'_, T>, arg0: i32, arg1: i32, arg2: i32| {
                Box::new(async move {
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
                    let result1 = host.http_response_body_set(param0).await;
                    let (result3_0, result3_1, result3_2) = match result1 {
                        Ok(()) => (0i32, 0i32, 0i32),
                        Err(e) => {
                            let vec2 = e;
                            let ptr2 = func_canonical_abi_realloc
                                .call_async(&mut caller, (0, 0, 1, (vec2.len() as i32) * 1))
                                .await?;
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
                })
            },
        )?;
        Ok(())
    }
    use core::convert::TryFrom;
    use wit_bindgen_wasmtime::rt::bad_int;
    use wit_bindgen_wasmtime::rt::RawMem;
}
