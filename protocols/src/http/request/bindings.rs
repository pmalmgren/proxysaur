#[allow(clippy::all)]
#[allow(dead_code)]
pub mod request {
    #[allow(unused_imports)]
    use wit_bindgen_wasmtime::{anyhow, wasmtime};
    pub type BodyParam<'a> = &'a [u8];
    pub type BodyResult = Vec<u8>;
    pub type Error = String;
    pub type HttpHeaders = Vec<(String, String)>;
    pub type HttpMethod = String;
    #[derive(Clone)]
    pub struct HttpRequest {
        pub path: String,
        pub authority: String,
        pub host: String,
        pub scheme: String,
        pub version: String,
        pub headers: HttpHeaders,
        pub method: HttpMethod,
        pub body: BodyResult,
    }
    impl std::fmt::Debug for HttpRequest {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("HttpRequest")
                .field("path", &self.path)
                .field("authority", &self.authority)
                .field("host", &self.host)
                .field("scheme", &self.scheme)
                .field("version", &self.version)
                .field("headers", &self.headers)
                .field("method", &self.method)
                .field("body", &self.body)
                .finish()
        }
    }
    pub trait Request: Sized {
        fn http_request_get(&mut self) -> Result<HttpRequest, Error>;

        fn http_request_set_method(&mut self, method: &str) -> Result<(), Error>;

        fn http_request_set_header(&mut self, header: &str, value: &str) -> Result<(), Error>;

        fn http_request_set_uri(&mut self, uri: &str) -> Result<(), Error>;

        fn http_request_set_version(&mut self, version: &str) -> Result<(), Error>;

        fn http_request_set_body(&mut self, body: BodyParam<'_>) -> Result<(), Error>;

        fn http_request_rm_header(&mut self, header: &str) -> Result<(), Error>;
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
                    result14_0,
                    result14_1,
                    result14_2,
                    result14_3,
                    result14_4,
                    result14_5,
                    result14_6,
                    result14_7,
                    result14_8,
                    result14_9,
                    result14_10,
                    result14_11,
                    result14_12,
                    result14_13,
                    result14_14,
                    result14_15,
                    result14_16,
                ) = match result0 {
                    Ok(e) => {
                        let HttpRequest {
                            path: path1,
                            authority: authority1,
                            host: host1,
                            scheme: scheme1,
                            version: version1,
                            headers: headers1,
                            method: method1,
                            body: body1,
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
                        let vec12 = body1;
                        let ptr12 = func_canonical_abi_realloc
                            .call(&mut caller, (0, 0, 1, (vec12.len() as i32) * 1))?;
                        let caller_memory = memory.data_mut(&mut caller);
                        caller_memory.store_many(ptr12, vec12.as_ref())?;
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
                            ptr12,
                            vec12.len() as i32,
                        )
                    }
                    Err(e) => {
                        let vec13 = e;
                        let ptr13 = func_canonical_abi_realloc
                            .call(&mut caller, (0, 0, 1, (vec13.len() as i32) * 1))?;
                        let caller_memory = memory.data_mut(&mut caller);
                        caller_memory.store_many(ptr13, vec13.as_ref())?;
                        (
                            1i32,
                            ptr13,
                            vec13.len() as i32,
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
                            0i32,
                            0i32,
                        )
                    }
                };
                let caller_memory = memory.data_mut(&mut caller);
                caller_memory.store(arg0 + 128, wit_bindgen_wasmtime::rt::as_i32(result14_16))?;
                caller_memory.store(arg0 + 120, wit_bindgen_wasmtime::rt::as_i32(result14_15))?;
                caller_memory.store(arg0 + 112, wit_bindgen_wasmtime::rt::as_i32(result14_14))?;
                caller_memory.store(arg0 + 104, wit_bindgen_wasmtime::rt::as_i32(result14_13))?;
                caller_memory.store(arg0 + 96, wit_bindgen_wasmtime::rt::as_i32(result14_12))?;
                caller_memory.store(arg0 + 88, wit_bindgen_wasmtime::rt::as_i32(result14_11))?;
                caller_memory.store(arg0 + 80, wit_bindgen_wasmtime::rt::as_i32(result14_10))?;
                caller_memory.store(arg0 + 72, wit_bindgen_wasmtime::rt::as_i32(result14_9))?;
                caller_memory.store(arg0 + 64, wit_bindgen_wasmtime::rt::as_i32(result14_8))?;
                caller_memory.store(arg0 + 56, wit_bindgen_wasmtime::rt::as_i32(result14_7))?;
                caller_memory.store(arg0 + 48, wit_bindgen_wasmtime::rt::as_i32(result14_6))?;
                caller_memory.store(arg0 + 40, wit_bindgen_wasmtime::rt::as_i32(result14_5))?;
                caller_memory.store(arg0 + 32, wit_bindgen_wasmtime::rt::as_i32(result14_4))?;
                caller_memory.store(arg0 + 24, wit_bindgen_wasmtime::rt::as_i32(result14_3))?;
                caller_memory.store(arg0 + 16, wit_bindgen_wasmtime::rt::as_i32(result14_2))?;
                caller_memory.store(arg0 + 8, wit_bindgen_wasmtime::rt::as_i32(result14_1))?;
                caller_memory.store(arg0 + 0, wit_bindgen_wasmtime::rt::as_i32(result14_0))?;
                Ok(())
            },
        )?;
        linker.func_wrap(
            "request",
            "http-request-set-method",
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
                let result1 = host.http_request_set_method(param0);
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
            "request",
            "http-request-set-header",
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
                let result2 = host.http_request_set_header(param0, param1);
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
            "request",
            "http-request-set-uri",
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
                let result1 = host.http_request_set_uri(param0);
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
            "request",
            "http-request-set-version",
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
                let result1 = host.http_request_set_version(param0);
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
            "request",
            "http-request-set-body",
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
                let result1 = host.http_request_set_body(param0);
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
            "request",
            "http-request-rm-header",
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
                let result1 = host.http_request_rm_header(param0);
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
