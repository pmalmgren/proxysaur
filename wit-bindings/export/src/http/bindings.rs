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
        pub body: BodyParam<'a>,
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
                .field("body", &self.body)
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
        pub body: BodyResult,
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
                .field("body", &self.body)
                .finish()
        }
    }
    pub trait Request: Sized {
        fn http_request_get(&mut self) -> Result<HttpRequestResult, Error>;

        fn http_request_set(&mut self, request: HttpRequestParam<'_>);

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
                        let HttpRequestResult {
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
                  arg14: i32,
                  arg15: i32| {
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
                let ptr13 = arg14;
                let len13 = arg15;
                let param0 = HttpRequestParam {
                    path: _bc.slice_str(ptr0, len0)?,
                    authority: _bc.slice_str(ptr1, len1)?,
                    host: _bc.slice_str(ptr2, len2)?,
                    scheme: _bc.slice_str(ptr3, len3)?,
                    version: _bc.slice_str(ptr4, len4)?,
                    headers: result11,
                    method: _bc.slice_str(ptr12, len12)?,
                    body: _bc.slice(ptr13, len13)?,
                };
                host.http_request_set(param0);
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
    use wit_bindgen_wasmtime::rt::invalid_variant;
    use wit_bindgen_wasmtime::rt::RawMem;
}
pub mod response {
    #[allow(unused_imports)]
    use wit_bindgen_wasmtime::{anyhow, wasmtime};
    pub type BodyParam<'a> = &'a [u8];
    pub type BodyResult = Vec<u8>;
    pub type Error = String;
    pub type HttpHeadersParam<'a> = Vec<(&'a str, &'a str)>;
    pub type HttpHeadersResult = Vec<(String, String)>;
    pub type HttpMethod = String;
    #[derive(Clone)]
    pub struct HttpResponse {
        pub headers: HttpHeadersResult,
        pub status: u16,
        pub body: BodyResult,
        pub request_path: String,
        pub request_authority: String,
        pub request_host: String,
        pub request_scheme: String,
        pub request_version: String,
        pub request_headers: HttpHeadersResult,
        pub request_method: HttpMethod,
    }
    impl std::fmt::Debug for HttpResponse {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("HttpResponse")
                .field("headers", &self.headers)
                .field("status", &self.status)
                .field("body", &self.body)
                .field("request-path", &self.request_path)
                .field("request-authority", &self.request_authority)
                .field("request-host", &self.request_host)
                .field("request-scheme", &self.request_scheme)
                .field("request-version", &self.request_version)
                .field("request-headers", &self.request_headers)
                .field("request-method", &self.request_method)
                .finish()
        }
    }
    pub trait Response: Sized {
        fn http_response_get(&mut self) -> Result<HttpResponse, Error>;

        fn http_response_set_status(&mut self, status: u16) -> Result<(), Error>;

        fn http_response_set_body(&mut self, body: BodyParam<'_>) -> Result<(), Error>;

        fn http_response_set_headers(&mut self, headers: HttpHeadersParam<'_>)
            -> Result<(), Error>;
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
                let (
                    result18_0,
                    result18_1,
                    result18_2,
                    result18_3,
                    result18_4,
                    result18_5,
                    result18_6,
                    result18_7,
                    result18_8,
                    result18_9,
                    result18_10,
                    result18_11,
                    result18_12,
                    result18_13,
                    result18_14,
                    result18_15,
                    result18_16,
                    result18_17,
                    result18_18,
                    result18_19,
                ) = match result0 {
                    Ok(e) => {
                        let HttpResponse {
                            headers: headers1,
                            status: status1,
                            body: body1,
                            request_path: request_path1,
                            request_authority: request_authority1,
                            request_host: request_host1,
                            request_scheme: request_scheme1,
                            request_version: request_version1,
                            request_headers: request_headers1,
                            request_method: request_method1,
                        } = e;
                        let vec5 = headers1;
                        let len5 = vec5.len() as i32;
                        let result5 =
                            func_canonical_abi_realloc.call(&mut caller, (0, 0, 4, len5 * 16))?;
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
                        let vec7 = request_path1;
                        let ptr7 = func_canonical_abi_realloc
                            .call(&mut caller, (0, 0, 1, (vec7.len() as i32) * 1))?;
                        let caller_memory = memory.data_mut(&mut caller);
                        caller_memory.store_many(ptr7, vec7.as_ref())?;
                        let vec8 = request_authority1;
                        let ptr8 = func_canonical_abi_realloc
                            .call(&mut caller, (0, 0, 1, (vec8.len() as i32) * 1))?;
                        let caller_memory = memory.data_mut(&mut caller);
                        caller_memory.store_many(ptr8, vec8.as_ref())?;
                        let vec9 = request_host1;
                        let ptr9 = func_canonical_abi_realloc
                            .call(&mut caller, (0, 0, 1, (vec9.len() as i32) * 1))?;
                        let caller_memory = memory.data_mut(&mut caller);
                        caller_memory.store_many(ptr9, vec9.as_ref())?;
                        let vec10 = request_scheme1;
                        let ptr10 = func_canonical_abi_realloc
                            .call(&mut caller, (0, 0, 1, (vec10.len() as i32) * 1))?;
                        let caller_memory = memory.data_mut(&mut caller);
                        caller_memory.store_many(ptr10, vec10.as_ref())?;
                        let vec11 = request_version1;
                        let ptr11 = func_canonical_abi_realloc
                            .call(&mut caller, (0, 0, 1, (vec11.len() as i32) * 1))?;
                        let caller_memory = memory.data_mut(&mut caller);
                        caller_memory.store_many(ptr11, vec11.as_ref())?;
                        let vec15 = request_headers1;
                        let len15 = vec15.len() as i32;
                        let result15 =
                            func_canonical_abi_realloc.call(&mut caller, (0, 0, 4, len15 * 16))?;
                        for (i, e) in vec15.into_iter().enumerate() {
                            let base = result15 + (i as i32) * 16;
                            {
                                let (t12_0, t12_1) = e;
                                let vec13 = t12_0;
                                let ptr13 = func_canonical_abi_realloc
                                    .call(&mut caller, (0, 0, 1, (vec13.len() as i32) * 1))?;
                                let caller_memory = memory.data_mut(&mut caller);
                                caller_memory.store_many(ptr13, vec13.as_ref())?;
                                caller_memory.store(
                                    base + 4,
                                    wit_bindgen_wasmtime::rt::as_i32(vec13.len() as i32),
                                )?;
                                caller_memory
                                    .store(base + 0, wit_bindgen_wasmtime::rt::as_i32(ptr13))?;
                                let vec14 = t12_1;
                                let ptr14 = func_canonical_abi_realloc
                                    .call(&mut caller, (0, 0, 1, (vec14.len() as i32) * 1))?;
                                let caller_memory = memory.data_mut(&mut caller);
                                caller_memory.store_many(ptr14, vec14.as_ref())?;
                                caller_memory.store(
                                    base + 12,
                                    wit_bindgen_wasmtime::rt::as_i32(vec14.len() as i32),
                                )?;
                                caller_memory
                                    .store(base + 8, wit_bindgen_wasmtime::rt::as_i32(ptr14))?;
                            }
                        }
                        let vec16 = request_method1;
                        let ptr16 = func_canonical_abi_realloc
                            .call(&mut caller, (0, 0, 1, (vec16.len() as i32) * 1))?;
                        let caller_memory = memory.data_mut(&mut caller);
                        caller_memory.store_many(ptr16, vec16.as_ref())?;
                        (
                            0i32,
                            result5,
                            len5,
                            wit_bindgen_wasmtime::rt::as_i32(status1),
                            ptr6,
                            vec6.len() as i32,
                            ptr7,
                            vec7.len() as i32,
                            ptr8,
                            vec8.len() as i32,
                            ptr9,
                            vec9.len() as i32,
                            ptr10,
                            vec10.len() as i32,
                            ptr11,
                            vec11.len() as i32,
                            result15,
                            len15,
                            ptr16,
                            vec16.len() as i32,
                        )
                    }
                    Err(e) => {
                        let vec17 = e;
                        let ptr17 = func_canonical_abi_realloc
                            .call(&mut caller, (0, 0, 1, (vec17.len() as i32) * 1))?;
                        let caller_memory = memory.data_mut(&mut caller);
                        caller_memory.store_many(ptr17, vec17.as_ref())?;
                        (
                            1i32,
                            ptr17,
                            vec17.len() as i32,
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
                            0i32,
                            0i32,
                            0i32,
                        )
                    }
                };
                let caller_memory = memory.data_mut(&mut caller);
                caller_memory.store(arg0 + 152, wit_bindgen_wasmtime::rt::as_i32(result18_19))?;
                caller_memory.store(arg0 + 144, wit_bindgen_wasmtime::rt::as_i32(result18_18))?;
                caller_memory.store(arg0 + 136, wit_bindgen_wasmtime::rt::as_i32(result18_17))?;
                caller_memory.store(arg0 + 128, wit_bindgen_wasmtime::rt::as_i32(result18_16))?;
                caller_memory.store(arg0 + 120, wit_bindgen_wasmtime::rt::as_i32(result18_15))?;
                caller_memory.store(arg0 + 112, wit_bindgen_wasmtime::rt::as_i32(result18_14))?;
                caller_memory.store(arg0 + 104, wit_bindgen_wasmtime::rt::as_i32(result18_13))?;
                caller_memory.store(arg0 + 96, wit_bindgen_wasmtime::rt::as_i32(result18_12))?;
                caller_memory.store(arg0 + 88, wit_bindgen_wasmtime::rt::as_i32(result18_11))?;
                caller_memory.store(arg0 + 80, wit_bindgen_wasmtime::rt::as_i32(result18_10))?;
                caller_memory.store(arg0 + 72, wit_bindgen_wasmtime::rt::as_i32(result18_9))?;
                caller_memory.store(arg0 + 64, wit_bindgen_wasmtime::rt::as_i32(result18_8))?;
                caller_memory.store(arg0 + 56, wit_bindgen_wasmtime::rt::as_i32(result18_7))?;
                caller_memory.store(arg0 + 48, wit_bindgen_wasmtime::rt::as_i32(result18_6))?;
                caller_memory.store(arg0 + 40, wit_bindgen_wasmtime::rt::as_i32(result18_5))?;
                caller_memory.store(arg0 + 32, wit_bindgen_wasmtime::rt::as_i32(result18_4))?;
                caller_memory.store(arg0 + 24, wit_bindgen_wasmtime::rt::as_i32(result18_3))?;
                caller_memory.store(arg0 + 16, wit_bindgen_wasmtime::rt::as_i32(result18_2))?;
                caller_memory.store(arg0 + 8, wit_bindgen_wasmtime::rt::as_i32(result18_1))?;
                caller_memory.store(arg0 + 0, wit_bindgen_wasmtime::rt::as_i32(result18_0))?;
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
            "http-response-set-headers",
            move |mut caller: wasmtime::Caller<'_, T>, arg0: i32, arg1: i32, arg2: i32| {
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
                let param0 = result6;
                let result7 = host.http_response_set_headers(param0);
                let (result9_0, result9_1, result9_2) = match result7 {
                    Ok(()) => (0i32, 0i32, 0i32),
                    Err(e) => {
                        let vec8 = e;
                        let ptr8 = func_canonical_abi_realloc
                            .call(&mut caller, (0, 0, 1, (vec8.len() as i32) * 1))?;
                        let caller_memory = memory.data_mut(&mut caller);
                        caller_memory.store_many(ptr8, vec8.as_ref())?;
                        (1i32, ptr8, vec8.len() as i32)
                    }
                };
                let caller_memory = memory.data_mut(&mut caller);
                caller_memory.store(arg2 + 16, wit_bindgen_wasmtime::rt::as_i32(result9_2))?;
                caller_memory.store(arg2 + 8, wit_bindgen_wasmtime::rt::as_i32(result9_1))?;
                caller_memory.store(arg2 + 0, wit_bindgen_wasmtime::rt::as_i32(result9_0))?;
                Ok(())
            },
        )?;
        Ok(())
    }
    use core::convert::TryFrom;
    use wit_bindgen_wasmtime::rt::bad_int;
    use wit_bindgen_wasmtime::rt::invalid_variant;
    use wit_bindgen_wasmtime::rt::RawMem;
}
