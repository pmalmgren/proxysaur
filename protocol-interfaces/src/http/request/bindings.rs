#![allow(warnings)]
#![allow(clippy)]
#![allow(unknown_lints)]

pub mod request {
    #[allow(unused_imports)]
    use wit_bindgen_wasmtime::{anyhow, wasmtime};
    pub type BodyParam<'a> = &'a [u8];
    pub type BodyResult = Vec<u8>;
    pub type Error = String;
    pub type HttpHeadersParam<'a> = Vec<(&'a str, &'a str)>;
    pub type HttpHeadersResult = Vec<(String, String)>;
    pub type HttpMethodParam<'a> = &'a str;
    pub type HttpMethodResult = String;
    #[derive(Clone)]
    pub struct HttpRequestParam<'a> {
        pub path: &'a str,
        pub authority: &'a str,
        pub host: &'a str,
        pub scheme: &'a str,
        pub version: &'a str,
        pub headers: HttpHeadersParam<'a>,
        pub method: HttpMethodParam<'a>,
    }
    impl<'a> std::fmt::Debug for HttpRequestParam<'a> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("HttpRequestParam")
                .field("path", &self.path)
                .field("authority", &self.authority)
                .field("host", &self.host)
                .field("scheme", &self.scheme)
                .field("version", &self.version)
                .field("headers", &self.headers)
                .field("method", &self.method)
                .finish()
        }
    }
    #[derive(Clone)]
    pub struct HttpRequestResult {
        pub path: String,
        pub authority: String,
        pub host: String,
        pub scheme: String,
        pub version: String,
        pub headers: HttpHeadersResult,
        pub method: HttpMethodResult,
    }
    impl std::fmt::Debug for HttpRequestResult {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("HttpRequestResult")
                .field("path", &self.path)
                .field("authority", &self.authority)
                .field("host", &self.host)
                .field("scheme", &self.scheme)
                .field("version", &self.version)
                .field("headers", &self.headers)
                .field("method", &self.method)
                .finish()
        }
    }
    pub trait Request: Sized {
        fn http_request_get(&mut self) -> Result<HttpRequestResult, Error>;

        fn http_request_set(&mut self, req: HttpRequestParam<'_>) -> Result<(), Error>;

        fn http_request_body_get(&mut self) -> Result<BodyResult, Error>;

        fn http_request_body_set(&mut self, body: BodyParam<'_>) -> Result<(), Error>;
    }

    pub fn add_to_linker<T, U>(
        linker: &mut wasmtime::Linker<T>,
        get: impl Fn(&mut T) -> &mut U + Send + Sync + Copy + 'static,
    ) -> anyhow::Result<()>
    where
        U: Request,
    {
        use wit_bindgen_wasmtime::rt::get_func;
        use wit_bindgen_wasmtime::rt::get_memory;
        linker.func_wrap(
            "request",
            "http-request-get",
            move |mut caller: wasmtime::Caller<'_, T>, arg0: i32| {
                let func = get_func(&mut caller, "canonical_abi_realloc")?;
                let func_canonical_abi_realloc =
                    func.typed::<(i32, i32, i32, i32), i32, _>(&caller)?;
                let memory = &get_memory(&mut caller, "memory")?;
                let host = get(caller.data_mut());
                let result0 = host.http_request_get();
                let (
                    result13_0,
                    result13_1,
                    result13_2,
                    result13_3,
                    result13_4,
                    result13_5,
                    result13_6,
                    result13_7,
                    result13_8,
                    result13_9,
                    result13_10,
                    result13_11,
                    result13_12,
                    result13_13,
                    result13_14,
                ) = match result0 {
                    Ok(e) => {
                        let HttpRequestResult {
                            path: path1,
                            authority: authority1,
                            host: host1,
                            scheme: scheme1,
                            version: version1,
                            headers: headers1,
                            method: method1,
                        } = e;
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
                        let vec6 = version1;
                        let ptr6 = func_canonical_abi_realloc
                            .call(&mut caller, (0, 0, 1, (vec6.len() as i32) * 1))?;
                        let caller_memory = memory.data_mut(&mut caller);
                        caller_memory.store_many(ptr6, vec6.as_ref())?;
                        let vec10 = headers1;
                        let len10 = vec10.len() as i32;
                        let result10 =
                            func_canonical_abi_realloc.call(&mut caller, (0, 0, 4, len10 * 16))?;
                        for (i, e) in vec10.into_iter().enumerate() {
                            let base = result10 + (i as i32) * 16;
                            {
                                let (t7_0, t7_1) = e;
                                let vec8 = t7_0;
                                let ptr8 = func_canonical_abi_realloc
                                    .call(&mut caller, (0, 0, 1, (vec8.len() as i32) * 1))?;
                                let caller_memory = memory.data_mut(&mut caller);
                                caller_memory.store_many(ptr8, vec8.as_ref())?;
                                caller_memory.store(
                                    base + 4,
                                    wit_bindgen_wasmtime::rt::as_i32(vec8.len() as i32),
                                )?;
                                caller_memory
                                    .store(base + 0, wit_bindgen_wasmtime::rt::as_i32(ptr8))?;
                                let vec9 = t7_1;
                                let ptr9 = func_canonical_abi_realloc
                                    .call(&mut caller, (0, 0, 1, (vec9.len() as i32) * 1))?;
                                let caller_memory = memory.data_mut(&mut caller);
                                caller_memory.store_many(ptr9, vec9.as_ref())?;
                                caller_memory.store(
                                    base + 12,
                                    wit_bindgen_wasmtime::rt::as_i32(vec9.len() as i32),
                                )?;
                                caller_memory
                                    .store(base + 8, wit_bindgen_wasmtime::rt::as_i32(ptr9))?;
                            }
                        }
                        let vec11 = method1;
                        let ptr11 = func_canonical_abi_realloc
                            .call(&mut caller, (0, 0, 1, (vec11.len() as i32) * 1))?;
                        let caller_memory = memory.data_mut(&mut caller);
                        caller_memory.store_many(ptr11, vec11.as_ref())?;
                        (
                            0i32,
                            ptr2,
                            vec2.len() as i32,
                            ptr3,
                            vec3.len() as i32,
                            ptr4,
                            vec4.len() as i32,
                            ptr5,
                            vec5.len() as i32,
                            ptr6,
                            vec6.len() as i32,
                            result10,
                            len10,
                            ptr11,
                            vec11.len() as i32,
                        )
                    }
                    Err(e) => {
                        let vec12 = e;
                        let ptr12 = func_canonical_abi_realloc
                            .call(&mut caller, (0, 0, 1, (vec12.len() as i32) * 1))?;
                        let caller_memory = memory.data_mut(&mut caller);
                        caller_memory.store_many(ptr12, vec12.as_ref())?;
                        (
                            1i32,
                            ptr12,
                            vec12.len() as i32,
                            0i32,
                            0i32,
                            0i32,
                            0i32,
                            0i32,
                            0i32,
                            0i32,
                            0i32,
                            0i32,
                            0i32,
                            0i32,
                            0i32,
                        )
                    }
                };
                let caller_memory = memory.data_mut(&mut caller);
                caller_memory.store(arg0 + 112, wit_bindgen_wasmtime::rt::as_i32(result13_14))?;
                caller_memory.store(arg0 + 104, wit_bindgen_wasmtime::rt::as_i32(result13_13))?;
                caller_memory.store(arg0 + 96, wit_bindgen_wasmtime::rt::as_i32(result13_12))?;
                caller_memory.store(arg0 + 88, wit_bindgen_wasmtime::rt::as_i32(result13_11))?;
                caller_memory.store(arg0 + 80, wit_bindgen_wasmtime::rt::as_i32(result13_10))?;
                caller_memory.store(arg0 + 72, wit_bindgen_wasmtime::rt::as_i32(result13_9))?;
                caller_memory.store(arg0 + 64, wit_bindgen_wasmtime::rt::as_i32(result13_8))?;
                caller_memory.store(arg0 + 56, wit_bindgen_wasmtime::rt::as_i32(result13_7))?;
                caller_memory.store(arg0 + 48, wit_bindgen_wasmtime::rt::as_i32(result13_6))?;
                caller_memory.store(arg0 + 40, wit_bindgen_wasmtime::rt::as_i32(result13_5))?;
                caller_memory.store(arg0 + 32, wit_bindgen_wasmtime::rt::as_i32(result13_4))?;
                caller_memory.store(arg0 + 24, wit_bindgen_wasmtime::rt::as_i32(result13_3))?;
                caller_memory.store(arg0 + 16, wit_bindgen_wasmtime::rt::as_i32(result13_2))?;
                caller_memory.store(arg0 + 8, wit_bindgen_wasmtime::rt::as_i32(result13_1))?;
                caller_memory.store(arg0 + 0, wit_bindgen_wasmtime::rt::as_i32(result13_0))?;
                Ok(())
            },
        )?;
        linker.func_wrap(
            "request",
            "http-request-set",
            move |mut caller: wasmtime::Caller<'_, T>,
                  arg0: i32,
                  arg1: i32,
                  arg2: i32,
                  arg3: i32,
                  arg4: i32,
                  arg5: i32,
                  arg6: i32,
                  arg7: i32,
                  arg8: i32,
                  arg9: i32,
                  arg10: i32,
                  arg11: i32,
                  arg12: i32,
                  arg13: i32,
                  arg14: i32| {
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
                let ptr2 = arg4;
                let len2 = arg5;
                let ptr3 = arg6;
                let len3 = arg7;
                let ptr4 = arg8;
                let len4 = arg9;
                let len11 = arg11;
                let base11 = arg10;
                let mut result11 = Vec::with_capacity(len11 as usize);
                for i in 0..len11 {
                    let base = base11 + i * 16;
                    result11.push({
                        let load5 = _bc.load::<i32>(base + 0)?;
                        let load6 = _bc.load::<i32>(base + 4)?;
                        let ptr7 = load5;
                        let len7 = load6;
                        let load8 = _bc.load::<i32>(base + 8)?;
                        let load9 = _bc.load::<i32>(base + 12)?;
                        let ptr10 = load8;
                        let len10 = load9;
                        (_bc.slice_str(ptr7, len7)?, _bc.slice_str(ptr10, len10)?)
                    });
                }
                let ptr12 = arg12;
                let len12 = arg13;
                let param0 = HttpRequestParam {
                    path: _bc.slice_str(ptr0, len0)?,
                    authority: _bc.slice_str(ptr1, len1)?,
                    host: _bc.slice_str(ptr2, len2)?,
                    scheme: _bc.slice_str(ptr3, len3)?,
                    version: _bc.slice_str(ptr4, len4)?,
                    headers: result11,
                    method: _bc.slice_str(ptr12, len12)?,
                };
                let result13 = host.http_request_set(param0);
                let (result15_0, result15_1, result15_2) = match result13 {
                    Ok(()) => (0i32, 0i32, 0i32),
                    Err(e) => {
                        let vec14 = e;
                        let ptr14 = func_canonical_abi_realloc
                            .call(&mut caller, (0, 0, 1, (vec14.len() as i32) * 1))?;
                        let caller_memory = memory.data_mut(&mut caller);
                        caller_memory.store_many(ptr14, vec14.as_ref())?;
                        (1i32, ptr14, vec14.len() as i32)
                    }
                };
                let caller_memory = memory.data_mut(&mut caller);
                caller_memory.store(arg14 + 16, wit_bindgen_wasmtime::rt::as_i32(result15_2))?;
                caller_memory.store(arg14 + 8, wit_bindgen_wasmtime::rt::as_i32(result15_1))?;
                caller_memory.store(arg14 + 0, wit_bindgen_wasmtime::rt::as_i32(result15_0))?;
                Ok(())
            },
        )?;
        linker.func_wrap(
            "request",
            "http-request-body-get",
            move |mut caller: wasmtime::Caller<'_, T>, arg0: i32| {
                let func = get_func(&mut caller, "canonical_abi_realloc")?;
                let func_canonical_abi_realloc =
                    func.typed::<(i32, i32, i32, i32), i32, _>(&caller)?;
                let memory = &get_memory(&mut caller, "memory")?;
                let host = get(caller.data_mut());
                let result0 = host.http_request_body_get();
                let (result3_0, result3_1, result3_2) = match result0 {
                    Ok(e) => {
                        let vec1 = e;
                        let ptr1 = func_canonical_abi_realloc
                            .call(&mut caller, (0, 0, 1, (vec1.len() as i32) * 1))?;
                        let caller_memory = memory.data_mut(&mut caller);
                        caller_memory.store_many(ptr1, vec1.as_ref())?;
                        (0i32, ptr1, vec1.len() as i32)
                    }
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
                caller_memory.store(arg0 + 16, wit_bindgen_wasmtime::rt::as_i32(result3_2))?;
                caller_memory.store(arg0 + 8, wit_bindgen_wasmtime::rt::as_i32(result3_1))?;
                caller_memory.store(arg0 + 0, wit_bindgen_wasmtime::rt::as_i32(result3_0))?;
                Ok(())
            },
        )?;
        linker.func_wrap(
            "request",
            "http-request-body-set",
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
                let result1 = host.http_request_body_set(param0);
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
    use wit_bindgen_wasmtime::rt::RawMem;
}
