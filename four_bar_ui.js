let wasm_bindgen;
(function() {
    const __exports = {};
    let wasm;

    const heap = new Array(32).fill(undefined);

    heap.push(undefined, null, true, false);

function getObject(idx) { return heap[idx]; }

let heap_next = heap.length;

function dropObject(idx) {
    if (idx < 36) return;
    heap[idx] = heap_next;
    heap_next = idx;
}

function takeObject(idx) {
    const ret = getObject(idx);
    dropObject(idx);
    return ret;
}

let cachedTextDecoder = new TextDecoder('utf-8', { ignoreBOM: true, fatal: true });

cachedTextDecoder.decode();

let cachegetUint8Memory0 = null;
function getUint8Memory0() {
    if (cachegetUint8Memory0 === null || cachegetUint8Memory0.buffer !== wasm.memory.buffer) {
        cachegetUint8Memory0 = new Uint8Array(wasm.memory.buffer);
    }
    return cachegetUint8Memory0;
}

function getStringFromWasm0(ptr, len) {
    return cachedTextDecoder.decode(getUint8Memory0().subarray(ptr, ptr + len));
}

function addHeapObject(obj) {
    if (heap_next === heap.length) heap.push(heap.length + 1);
    const idx = heap_next;
    heap_next = heap[idx];

    heap[idx] = obj;
    return idx;
}

function isLikeNone(x) {
    return x === undefined || x === null;
}

let cachegetFloat64Memory0 = null;
function getFloat64Memory0() {
    if (cachegetFloat64Memory0 === null || cachegetFloat64Memory0.buffer !== wasm.memory.buffer) {
        cachegetFloat64Memory0 = new Float64Array(wasm.memory.buffer);
    }
    return cachegetFloat64Memory0;
}

let cachegetInt32Memory0 = null;
function getInt32Memory0() {
    if (cachegetInt32Memory0 === null || cachegetInt32Memory0.buffer !== wasm.memory.buffer) {
        cachegetInt32Memory0 = new Int32Array(wasm.memory.buffer);
    }
    return cachegetInt32Memory0;
}

function debugString(val) {
    // primitive types
    const type = typeof val;
    if (type == 'number' || type == 'boolean' || val == null) {
        return  `${val}`;
    }
    if (type == 'string') {
        return `"${val}"`;
    }
    if (type == 'symbol') {
        const description = val.description;
        if (description == null) {
            return 'Symbol';
        } else {
            return `Symbol(${description})`;
        }
    }
    if (type == 'function') {
        const name = val.name;
        if (typeof name == 'string' && name.length > 0) {
            return `Function(${name})`;
        } else {
            return 'Function';
        }
    }
    // objects
    if (Array.isArray(val)) {
        const length = val.length;
        let debug = '[';
        if (length > 0) {
            debug += debugString(val[0]);
        }
        for(let i = 1; i < length; i++) {
            debug += ', ' + debugString(val[i]);
        }
        debug += ']';
        return debug;
    }
    // Test for built-in
    const builtInMatches = /\[object ([^\]]+)\]/.exec(toString.call(val));
    let className;
    if (builtInMatches.length > 1) {
        className = builtInMatches[1];
    } else {
        // Failed to match the standard '[object ClassName]'
        return toString.call(val);
    }
    if (className == 'Object') {
        // we're a user defined class or Object
        // JSON.stringify avoids problems with cycles, and is generally much
        // easier than looping through ownProperties of `val`.
        try {
            return 'Object(' + JSON.stringify(val) + ')';
        } catch (_) {
            return 'Object';
        }
    }
    // errors
    if (val instanceof Error) {
        return `${val.name}: ${val.message}\n${val.stack}`;
    }
    // TODO we could test for more things here, like `Set`s and `Map`s.
    return className;
}

let WASM_VECTOR_LEN = 0;

let cachedTextEncoder = new TextEncoder('utf-8');

const encodeString = (typeof cachedTextEncoder.encodeInto === 'function'
    ? function (arg, view) {
    return cachedTextEncoder.encodeInto(arg, view);
}
    : function (arg, view) {
    const buf = cachedTextEncoder.encode(arg);
    view.set(buf);
    return {
        read: arg.length,
        written: buf.length
    };
});

function passStringToWasm0(arg, malloc, realloc) {

    if (realloc === undefined) {
        const buf = cachedTextEncoder.encode(arg);
        const ptr = malloc(buf.length);
        getUint8Memory0().subarray(ptr, ptr + buf.length).set(buf);
        WASM_VECTOR_LEN = buf.length;
        return ptr;
    }

    let len = arg.length;
    let ptr = malloc(len);

    const mem = getUint8Memory0();

    let offset = 0;

    for (; offset < len; offset++) {
        const code = arg.charCodeAt(offset);
        if (code > 0x7F) break;
        mem[ptr + offset] = code;
    }

    if (offset !== len) {
        if (offset !== 0) {
            arg = arg.slice(offset);
        }
        ptr = realloc(ptr, len, len = offset + arg.length * 3);
        const view = getUint8Memory0().subarray(ptr + offset, ptr + len);
        const ret = encodeString(arg, view);

        offset += ret.written;
    }

    WASM_VECTOR_LEN = offset;
    return ptr;
}

function makeMutClosure(arg0, arg1, dtor, f) {
    const state = { a: arg0, b: arg1, cnt: 1, dtor };
    const real = (...args) => {
        // First up with a closure we increment the internal reference
        // count. This ensures that the Rust closure environment won't
        // be deallocated while we're invoking it.
        state.cnt++;
        const a = state.a;
        state.a = 0;
        try {
            return f(a, state.b, ...args);
        } finally {
            if (--state.cnt === 0) {
                wasm.__wbindgen_export_2.get(state.dtor)(a, state.b);

            } else {
                state.a = a;
            }
        }
    };
    real.original = state;

    return real;
}
function __wbg_adapter_24(arg0, arg1) {
    wasm._dyn_core__ops__function__FnMut_____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h4c48d7a80b98b6f3(arg0, arg1);
}

function __wbg_adapter_27(arg0, arg1, arg2) {
    wasm._dyn_core__ops__function__FnMut__A____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h13ea291a7d7270b5(arg0, arg1, addHeapObject(arg2));
}

function __wbg_adapter_30(arg0, arg1, arg2) {
    wasm._dyn_core__ops__function__FnMut__A____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h13ea291a7d7270b5(arg0, arg1, addHeapObject(arg2));
}

function __wbg_adapter_33(arg0, arg1, arg2) {
    wasm._dyn_core__ops__function__FnMut__A____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h13ea291a7d7270b5(arg0, arg1, addHeapObject(arg2));
}

function __wbg_adapter_36(arg0, arg1, arg2) {
    wasm._dyn_core__ops__function__FnMut__A____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h13ea291a7d7270b5(arg0, arg1, addHeapObject(arg2));
}

function __wbg_adapter_39(arg0, arg1) {
    wasm._dyn_core__ops__function__FnMut_____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h5336f2771695d54e(arg0, arg1);
}

function __wbg_adapter_42(arg0, arg1, arg2) {
    wasm._dyn_core__ops__function__FnMut__A____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h13ea291a7d7270b5(arg0, arg1, addHeapObject(arg2));
}

function __wbg_adapter_45(arg0, arg1, arg2) {
    wasm._dyn_core__ops__function__FnMut__A____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h13ea291a7d7270b5(arg0, arg1, addHeapObject(arg2));
}

function __wbg_adapter_48(arg0, arg1, arg2) {
    wasm._dyn_core__ops__function__FnMut__A____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h13ea291a7d7270b5(arg0, arg1, addHeapObject(arg2));
}

function __wbg_adapter_51(arg0, arg1, arg2) {
    wasm._dyn_core__ops__function__FnMut__A____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h13ea291a7d7270b5(arg0, arg1, addHeapObject(arg2));
}

function __wbg_adapter_54(arg0, arg1, arg2) {
    wasm._dyn_core__ops__function__FnMut__A____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h78156398f8c5f57f(arg0, arg1, addHeapObject(arg2));
}

/**
* This is the entry-point for all the web-assembly.
* This is called once from the HTML.
* It loads the app, installs some callbacks, then returns.
* You can add more callbacks like this if you want to call in to your code.
* @param {string} canvas_id
*/
__exports.start = function(canvas_id) {
    var ptr0 = passStringToWasm0(canvas_id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    var len0 = WASM_VECTOR_LEN;
    wasm.start(ptr0, len0);
};

function getArrayU8FromWasm0(ptr, len) {
    return getUint8Memory0().subarray(ptr / 1, ptr / 1 + len);
}

function handleError(f, args) {
    try {
        return f.apply(this, args);
    } catch (e) {
        wasm.__wbindgen_exn_store(addHeapObject(e));
    }
}

async function load(module, imports) {
    if (typeof Response === 'function' && module instanceof Response) {
        if (typeof WebAssembly.instantiateStreaming === 'function') {
            try {
                return await WebAssembly.instantiateStreaming(module, imports);

            } catch (e) {
                if (module.headers.get('Content-Type') != 'application/wasm') {
                    console.warn("`WebAssembly.instantiateStreaming` failed because your server does not serve wasm with `application/wasm` MIME type. Falling back to `WebAssembly.instantiate` which is slower. Original error:\n", e);

                } else {
                    throw e;
                }
            }
        }

        const bytes = await module.arrayBuffer();
        return await WebAssembly.instantiate(bytes, imports);

    } else {
        const instance = await WebAssembly.instantiate(module, imports);

        if (instance instanceof WebAssembly.Instance) {
            return { instance, module };

        } else {
            return instance;
        }
    }
}

async function init(input) {
    if (typeof input === 'undefined') {
        let src;
        if (typeof document === 'undefined') {
            src = location.href;
        } else {
            src = document.currentScript.src;
        }
        input = src.replace(/\.js$/, '_bg.wasm');
    }
    const imports = {};
    imports.wbg = {};
    imports.wbg.__wbindgen_object_drop_ref = function(arg0) {
        takeObject(arg0);
    };
    imports.wbg.__wbindgen_string_new = function(arg0, arg1) {
        var ret = getStringFromWasm0(arg0, arg1);
        return addHeapObject(ret);
    };
    imports.wbg.__wbindgen_cb_drop = function(arg0) {
        const obj = takeObject(arg0).original;
        if (obj.cnt-- == 1) {
            obj.a = 0;
            return true;
        }
        var ret = false;
        return ret;
    };
    imports.wbg.__wbindgen_number_get = function(arg0, arg1) {
        const obj = getObject(arg1);
        var ret = typeof(obj) === 'number' ? obj : undefined;
        getFloat64Memory0()[arg0 / 8 + 1] = isLikeNone(ret) ? 0 : ret;
        getInt32Memory0()[arg0 / 4 + 0] = !isLikeNone(ret);
    };
    imports.wbg.__wbindgen_object_clone_ref = function(arg0) {
        var ret = getObject(arg0);
        return addHeapObject(ret);
    };
    imports.wbg.__wbindgen_boolean_get = function(arg0) {
        const v = getObject(arg0);
        var ret = typeof(v) === 'boolean' ? (v ? 1 : 0) : 2;
        return ret;
    };
    imports.wbg.__wbg_instanceof_WebGl2RenderingContext_4bca42e6272623fb = function(arg0) {
        var ret = getObject(arg0) instanceof WebGL2RenderingContext;
        return ret;
    };
    imports.wbg.__wbg_drawingBufferWidth_fd7e9a03dcdea7e2 = function(arg0) {
        var ret = getObject(arg0).drawingBufferWidth;
        return ret;
    };
    imports.wbg.__wbg_drawingBufferHeight_2232ff03ed5c4499 = function(arg0) {
        var ret = getObject(arg0).drawingBufferHeight;
        return ret;
    };
    imports.wbg.__wbg_bindVertexArray_d4e9c27876b2db68 = function(arg0, arg1) {
        getObject(arg0).bindVertexArray(getObject(arg1));
    };
    imports.wbg.__wbg_bufferData_2a7070d86f2279b9 = function(arg0, arg1, arg2, arg3) {
        getObject(arg0).bufferData(arg1 >>> 0, getObject(arg2), arg3 >>> 0);
    };
    imports.wbg.__wbg_bufferData_03730841dc66bf7f = function(arg0, arg1, arg2, arg3, arg4) {
        getObject(arg0).bufferData(arg1 >>> 0, getArrayU8FromWasm0(arg2, arg3), arg4 >>> 0);
    };
    imports.wbg.__wbg_createVertexArray_b4635ed4ee325b2c = function(arg0) {
        var ret = getObject(arg0).createVertexArray();
        return isLikeNone(ret) ? 0 : addHeapObject(ret);
    };
    imports.wbg.__wbg_deleteVertexArray_78e15c0d00e758ce = function(arg0, arg1) {
        getObject(arg0).deleteVertexArray(getObject(arg1));
    };
    imports.wbg.__wbg_texImage2D_a8634891bbe1cc4c = function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9, arg10) {
        getObject(arg0).texImage2D(arg1 >>> 0, arg2, arg3, arg4, arg5, arg6, arg7 >>> 0, arg8 >>> 0, arg9 === 0 ? undefined : getArrayU8FromWasm0(arg9, arg10));
    }, arguments) };
    imports.wbg.__wbg_activeTexture_01fc5074f685836a = function(arg0, arg1) {
        getObject(arg0).activeTexture(arg1 >>> 0);
    };
    imports.wbg.__wbg_attachShader_f542bc1ffd19b410 = function(arg0, arg1, arg2) {
        getObject(arg0).attachShader(getObject(arg1), getObject(arg2));
    };
    imports.wbg.__wbg_bindBuffer_7d0fa67b9824b872 = function(arg0, arg1, arg2) {
        getObject(arg0).bindBuffer(arg1 >>> 0, getObject(arg2));
    };
    imports.wbg.__wbg_bindFramebuffer_550800f08ff6afb4 = function(arg0, arg1, arg2) {
        getObject(arg0).bindFramebuffer(arg1 >>> 0, getObject(arg2));
    };
    imports.wbg.__wbg_bindTexture_032c59f593fe72e4 = function(arg0, arg1, arg2) {
        getObject(arg0).bindTexture(arg1 >>> 0, getObject(arg2));
    };
    imports.wbg.__wbg_blendFunc_39ebb55119589901 = function(arg0, arg1, arg2) {
        getObject(arg0).blendFunc(arg1 >>> 0, arg2 >>> 0);
    };
    imports.wbg.__wbg_clear_7f7cbe327101511a = function(arg0, arg1) {
        getObject(arg0).clear(arg1 >>> 0);
    };
    imports.wbg.__wbg_clearColor_997c3f5e6f9f08d6 = function(arg0, arg1, arg2, arg3, arg4) {
        getObject(arg0).clearColor(arg1, arg2, arg3, arg4);
    };
    imports.wbg.__wbg_compileShader_dc2d5d344e104ba7 = function(arg0, arg1) {
        getObject(arg0).compileShader(getObject(arg1));
    };
    imports.wbg.__wbg_createBuffer_70609557beb6328a = function(arg0) {
        var ret = getObject(arg0).createBuffer();
        return isLikeNone(ret) ? 0 : addHeapObject(ret);
    };
    imports.wbg.__wbg_createFramebuffer_a0f2a25360a9174d = function(arg0) {
        var ret = getObject(arg0).createFramebuffer();
        return isLikeNone(ret) ? 0 : addHeapObject(ret);
    };
    imports.wbg.__wbg_createProgram_423e552f15789a1f = function(arg0) {
        var ret = getObject(arg0).createProgram();
        return isLikeNone(ret) ? 0 : addHeapObject(ret);
    };
    imports.wbg.__wbg_createShader_51d8b138b7a9a555 = function(arg0, arg1) {
        var ret = getObject(arg0).createShader(arg1 >>> 0);
        return isLikeNone(ret) ? 0 : addHeapObject(ret);
    };
    imports.wbg.__wbg_createTexture_8a51782037c34283 = function(arg0) {
        var ret = getObject(arg0).createTexture();
        return isLikeNone(ret) ? 0 : addHeapObject(ret);
    };
    imports.wbg.__wbg_deleteBuffer_8c5ad4beb33c9227 = function(arg0, arg1) {
        getObject(arg0).deleteBuffer(getObject(arg1));
    };
    imports.wbg.__wbg_deleteFramebuffer_854a708728101f3c = function(arg0, arg1) {
        getObject(arg0).deleteFramebuffer(getObject(arg1));
    };
    imports.wbg.__wbg_deleteProgram_916cc3ff29d3366b = function(arg0, arg1) {
        getObject(arg0).deleteProgram(getObject(arg1));
    };
    imports.wbg.__wbg_deleteTexture_06b9f5bf8e1e9488 = function(arg0, arg1) {
        getObject(arg0).deleteTexture(getObject(arg1));
    };
    imports.wbg.__wbg_disable_ab3246f27a493a7b = function(arg0, arg1) {
        getObject(arg0).disable(arg1 >>> 0);
    };
    imports.wbg.__wbg_drawElements_6c421d9a40d2b2d3 = function(arg0, arg1, arg2, arg3, arg4) {
        getObject(arg0).drawElements(arg1 >>> 0, arg2, arg3 >>> 0, arg4);
    };
    imports.wbg.__wbg_enable_76253bb6a6d1c2fd = function(arg0, arg1) {
        getObject(arg0).enable(arg1 >>> 0);
    };
    imports.wbg.__wbg_enableVertexAttribArray_3c8b80f0852aad0d = function(arg0, arg1) {
        getObject(arg0).enableVertexAttribArray(arg1 >>> 0);
    };
    imports.wbg.__wbg_framebufferTexture2D_c85086243d9b5c37 = function(arg0, arg1, arg2, arg3, arg4, arg5) {
        getObject(arg0).framebufferTexture2D(arg1 >>> 0, arg2 >>> 0, arg3 >>> 0, getObject(arg4), arg5);
    };
    imports.wbg.__wbg_getAttribLocation_13c06bdd827ac0aa = function(arg0, arg1, arg2, arg3) {
        var ret = getObject(arg0).getAttribLocation(getObject(arg1), getStringFromWasm0(arg2, arg3));
        return ret;
    };
    imports.wbg.__wbg_getProgramInfoLog_b403ac06b96d8f26 = function(arg0, arg1, arg2) {
        var ret = getObject(arg1).getProgramInfoLog(getObject(arg2));
        var ptr0 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        var len0 = WASM_VECTOR_LEN;
        getInt32Memory0()[arg0 / 4 + 1] = len0;
        getInt32Memory0()[arg0 / 4 + 0] = ptr0;
    };
    imports.wbg.__wbg_getProgramParameter_752b126ee61add0d = function(arg0, arg1, arg2) {
        var ret = getObject(arg0).getProgramParameter(getObject(arg1), arg2 >>> 0);
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_getShaderInfoLog_cc1ddacf5e0ae0c9 = function(arg0, arg1, arg2) {
        var ret = getObject(arg1).getShaderInfoLog(getObject(arg2));
        var ptr0 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        var len0 = WASM_VECTOR_LEN;
        getInt32Memory0()[arg0 / 4 + 1] = len0;
        getInt32Memory0()[arg0 / 4 + 0] = ptr0;
    };
    imports.wbg.__wbg_getShaderParameter_3f1e2ab0eeb1cf60 = function(arg0, arg1, arg2) {
        var ret = getObject(arg0).getShaderParameter(getObject(arg1), arg2 >>> 0);
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_getUniformLocation_07b2d367ed388a0d = function(arg0, arg1, arg2, arg3) {
        var ret = getObject(arg0).getUniformLocation(getObject(arg1), getStringFromWasm0(arg2, arg3));
        return isLikeNone(ret) ? 0 : addHeapObject(ret);
    };
    imports.wbg.__wbg_linkProgram_09bd455e5aa5e84a = function(arg0, arg1) {
        getObject(arg0).linkProgram(getObject(arg1));
    };
    imports.wbg.__wbg_pixelStorei_7cbc86618e815f2f = function(arg0, arg1, arg2) {
        getObject(arg0).pixelStorei(arg1 >>> 0, arg2);
    };
    imports.wbg.__wbg_scissor_25178779edb32faf = function(arg0, arg1, arg2, arg3, arg4) {
        getObject(arg0).scissor(arg1, arg2, arg3, arg4);
    };
    imports.wbg.__wbg_shaderSource_5910f0f849793916 = function(arg0, arg1, arg2, arg3) {
        getObject(arg0).shaderSource(getObject(arg1), getStringFromWasm0(arg2, arg3));
    };
    imports.wbg.__wbg_texParameteri_1d3e4ea04be20d49 = function(arg0, arg1, arg2, arg3) {
        getObject(arg0).texParameteri(arg1 >>> 0, arg2 >>> 0, arg3);
    };
    imports.wbg.__wbg_uniform1i_ff83cd14eeaae333 = function(arg0, arg1, arg2) {
        getObject(arg0).uniform1i(getObject(arg1), arg2);
    };
    imports.wbg.__wbg_uniform2f_10c3c6542fbb58e0 = function(arg0, arg1, arg2, arg3) {
        getObject(arg0).uniform2f(getObject(arg1), arg2, arg3);
    };
    imports.wbg.__wbg_useProgram_ee4a6346a3af060e = function(arg0, arg1) {
        getObject(arg0).useProgram(getObject(arg1));
    };
    imports.wbg.__wbg_vertexAttribPointer_f317dbb314cc891f = function(arg0, arg1, arg2, arg3, arg4, arg5, arg6) {
        getObject(arg0).vertexAttribPointer(arg1 >>> 0, arg2, arg3 >>> 0, arg4 !== 0, arg5, arg6);
    };
    imports.wbg.__wbg_viewport_c0b8e58acb57439a = function(arg0, arg1, arg2, arg3, arg4) {
        getObject(arg0).viewport(arg1, arg2, arg3, arg4);
    };
    imports.wbg.__wbg_instanceof_Window_c4e9146e14ca4a40 = function(arg0) {
        var ret = getObject(arg0) instanceof Window;
        return ret;
    };
    imports.wbg.__wbg_document_40cc17d69aad887e = function(arg0) {
        var ret = getObject(arg0).document;
        return isLikeNone(ret) ? 0 : addHeapObject(ret);
    };
    imports.wbg.__wbg_location_b4f1c5750707af99 = function(arg0) {
        var ret = getObject(arg0).location;
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_navigator_4f6900c2437f3b70 = function(arg0) {
        var ret = getObject(arg0).navigator;
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_innerWidth_1ebaf8a7e1a14dac = function() { return handleError(function (arg0) {
        var ret = getObject(arg0).innerWidth;
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_innerHeight_5036fb5a241360f1 = function() { return handleError(function (arg0) {
        var ret = getObject(arg0).innerHeight;
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_devicePixelRatio_81c875524d455070 = function(arg0) {
        var ret = getObject(arg0).devicePixelRatio;
        return ret;
    };
    imports.wbg.__wbg_performance_f4d483f65b00e19a = function(arg0) {
        var ret = getObject(arg0).performance;
        return isLikeNone(ret) ? 0 : addHeapObject(ret);
    };
    imports.wbg.__wbg_localStorage_6f2f241c03ddd312 = function() { return handleError(function (arg0) {
        var ret = getObject(arg0).localStorage;
        return isLikeNone(ret) ? 0 : addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_matchMedia_570f99cfe245a04f = function() { return handleError(function (arg0, arg1, arg2) {
        var ret = getObject(arg0).matchMedia(getStringFromWasm0(arg1, arg2));
        return isLikeNone(ret) ? 0 : addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_open_9ca4ceed5860be93 = function() { return handleError(function (arg0, arg1, arg2, arg3, arg4) {
        var ret = getObject(arg0).open(getStringFromWasm0(arg1, arg2), getStringFromWasm0(arg3, arg4));
        return isLikeNone(ret) ? 0 : addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_requestAnimationFrame_69e903cfce563326 = function() { return handleError(function (arg0, arg1) {
        var ret = getObject(arg0).requestAnimationFrame(getObject(arg1));
        return ret;
    }, arguments) };
    imports.wbg.__wbg_setInterval_53d06583cf9aceb7 = function() { return handleError(function (arg0, arg1, arg2) {
        var ret = getObject(arg0).setInterval(getObject(arg1), arg2);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_setTimeout_9fae816b718a8753 = function() { return handleError(function (arg0, arg1, arg2) {
        var ret = getObject(arg0).setTimeout(getObject(arg1), arg2);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_items_2468df1546077732 = function(arg0) {
        var ret = getObject(arg0).items;
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_files_483369787ec2495d = function(arg0) {
        var ret = getObject(arg0).files;
        return isLikeNone(ret) ? 0 : addHeapObject(ret);
    };
    imports.wbg.__wbg_length_91e40e8cbbed3a9e = function(arg0) {
        var ret = getObject(arg0).length;
        return ret;
    };
    imports.wbg.__wbg_get_203d2ddcab644f0e = function(arg0, arg1) {
        var ret = getObject(arg0)[arg1 >>> 0];
        return isLikeNone(ret) ? 0 : addHeapObject(ret);
    };
    imports.wbg.__wbg_name_0fadb40f87e3a950 = function(arg0, arg1) {
        var ret = getObject(arg1).name;
        var ptr0 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        var len0 = WASM_VECTOR_LEN;
        getInt32Memory0()[arg0 / 4 + 1] = len0;
        getInt32Memory0()[arg0 / 4 + 0] = ptr0;
    };
    imports.wbg.__wbg_lastModified_41803dbd8e559469 = function(arg0) {
        var ret = getObject(arg0).lastModified;
        return ret;
    };
    imports.wbg.__wbg_length_92e6558c8ff3ada6 = function(arg0) {
        var ret = getObject(arg0).length;
        return ret;
    };
    imports.wbg.__wbg_item_f185893d93a43303 = function(arg0, arg1) {
        var ret = getObject(arg0).item(arg1 >>> 0);
        return isLikeNone(ret) ? 0 : addHeapObject(ret);
    };
    imports.wbg.__wbg_get_317f6fce77e7ed3e = function(arg0, arg1) {
        var ret = getObject(arg0)[arg1 >>> 0];
        return isLikeNone(ret) ? 0 : addHeapObject(ret);
    };
    imports.wbg.__wbg_type_fc6cfff24586a1fe = function(arg0, arg1) {
        var ret = getObject(arg1).type;
        var ptr0 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        var len0 = WASM_VECTOR_LEN;
        getInt32Memory0()[arg0 / 4 + 1] = len0;
        getInt32Memory0()[arg0 / 4 + 0] = ptr0;
    };
    imports.wbg.__wbg_preventDefault_e121cdcccef1e0ee = function(arg0) {
        getObject(arg0).preventDefault();
    };
    imports.wbg.__wbg_stopPropagation_e8771a6d4a5c3c5e = function(arg0) {
        getObject(arg0).stopPropagation();
    };
    imports.wbg.__wbg_instanceof_HtmlCanvasElement_6c2ace70ed374c47 = function(arg0) {
        var ret = getObject(arg0) instanceof HTMLCanvasElement;
        return ret;
    };
    imports.wbg.__wbg_width_37834ac59a35e1e8 = function(arg0) {
        var ret = getObject(arg0).width;
        return ret;
    };
    imports.wbg.__wbg_setwidth_6b276650fdafc277 = function(arg0, arg1) {
        getObject(arg0).width = arg1 >>> 0;
    };
    imports.wbg.__wbg_height_163c7f5f1560dc9e = function(arg0) {
        var ret = getObject(arg0).height;
        return ret;
    };
    imports.wbg.__wbg_setheight_705eb5fd16a41367 = function(arg0, arg1) {
        getObject(arg0).height = arg1 >>> 0;
    };
    imports.wbg.__wbg_getContext_9bb15fd189b2c9f1 = function() { return handleError(function (arg0, arg1, arg2) {
        var ret = getObject(arg0).getContext(getStringFromWasm0(arg1, arg2));
        return isLikeNone(ret) ? 0 : addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_keyCode_d7ae20ee69131a64 = function(arg0) {
        var ret = getObject(arg0).keyCode;
        return ret;
    };
    imports.wbg.__wbg_altKey_7c54a09919382401 = function(arg0) {
        var ret = getObject(arg0).altKey;
        return ret;
    };
    imports.wbg.__wbg_ctrlKey_7508ba8766c257c1 = function(arg0) {
        var ret = getObject(arg0).ctrlKey;
        return ret;
    };
    imports.wbg.__wbg_shiftKey_66fbffa2937afdb2 = function(arg0) {
        var ret = getObject(arg0).shiftKey;
        return ret;
    };
    imports.wbg.__wbg_metaKey_2ca09c032d0be9e1 = function(arg0) {
        var ret = getObject(arg0).metaKey;
        return ret;
    };
    imports.wbg.__wbg_isComposing_91b8f88e4542dd29 = function(arg0) {
        var ret = getObject(arg0).isComposing;
        return ret;
    };
    imports.wbg.__wbg_key_4561659253ce92e8 = function(arg0, arg1) {
        var ret = getObject(arg1).key;
        var ptr0 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        var len0 = WASM_VECTOR_LEN;
        getInt32Memory0()[arg0 / 4 + 1] = len0;
        getInt32Memory0()[arg0 / 4 + 0] = ptr0;
    };
    imports.wbg.__wbg_clipboard_7340acd7970ccd28 = function(arg0) {
        var ret = getObject(arg0).clipboard;
        return isLikeNone(ret) ? 0 : addHeapObject(ret);
    };
    imports.wbg.__wbg_userAgent_2a093ead92bc733c = function() { return handleError(function (arg0, arg1) {
        var ret = getObject(arg1).userAgent;
        var ptr0 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        var len0 = WASM_VECTOR_LEN;
        getInt32Memory0()[arg0 / 4 + 1] = len0;
        getInt32Memory0()[arg0 / 4 + 0] = ptr0;
    }, arguments) };
    imports.wbg.__wbg_touches_03697ecad0c05fff = function(arg0) {
        var ret = getObject(arg0).touches;
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_changedTouches_c324094fba0bde35 = function(arg0) {
        var ret = getObject(arg0).changedTouches;
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_top_cfc010fbb069e4c4 = function(arg0) {
        var ret = getObject(arg0).top;
        return ret;
    };
    imports.wbg.__wbg_left_b68f7377242562f5 = function(arg0) {
        var ret = getObject(arg0).left;
        return ret;
    };
    imports.wbg.__wbg_length_4f566096343afbd4 = function(arg0) {
        var ret = getObject(arg0).length;
        return ret;
    };
    imports.wbg.__wbg_get_3ff0383244a8575b = function(arg0, arg1) {
        var ret = getObject(arg0)[arg1 >>> 0];
        return isLikeNone(ret) ? 0 : addHeapObject(ret);
    };
    imports.wbg.__wbg_appendChild_d1a880b4e5296331 = function() { return handleError(function (arg0, arg1) {
        var ret = getObject(arg0).appendChild(getObject(arg1));
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_getItem_30ec5a1f41cd501c = function() { return handleError(function (arg0, arg1, arg2, arg3) {
        var ret = getObject(arg1).getItem(getStringFromWasm0(arg2, arg3));
        var ptr0 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        var len0 = WASM_VECTOR_LEN;
        getInt32Memory0()[arg0 / 4 + 1] = len0;
        getInt32Memory0()[arg0 / 4 + 0] = ptr0;
    }, arguments) };
    imports.wbg.__wbg_setItem_409efa86019adbd6 = function() { return handleError(function (arg0, arg1, arg2, arg3, arg4) {
        getObject(arg0).setItem(getStringFromWasm0(arg1, arg2), getStringFromWasm0(arg3, arg4));
    }, arguments) };
    imports.wbg.__wbg_size_5f428f6c97d71159 = function(arg0) {
        var ret = getObject(arg0).size;
        return ret;
    };
    imports.wbg.__wbg_arrayBuffer_d77e32da27e23063 = function(arg0) {
        var ret = getObject(arg0).arrayBuffer();
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_hash_2c72f10162e9ec6c = function() { return handleError(function (arg0, arg1) {
        var ret = getObject(arg1).hash;
        var ptr0 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        var len0 = WASM_VECTOR_LEN;
        getInt32Memory0()[arg0 / 4 + 1] = len0;
        getInt32Memory0()[arg0 / 4 + 0] = ptr0;
    }, arguments) };
    imports.wbg.__wbg_clientX_356d72e5d45d20b0 = function(arg0) {
        var ret = getObject(arg0).clientX;
        return ret;
    };
    imports.wbg.__wbg_clientY_2be90f8fd44a2457 = function(arg0) {
        var ret = getObject(arg0).clientY;
        return ret;
    };
    imports.wbg.__wbg_ctrlKey_6524b1ff1ec7c0d9 = function(arg0) {
        var ret = getObject(arg0).ctrlKey;
        return ret;
    };
    imports.wbg.__wbg_metaKey_756c220b0900cfe6 = function(arg0) {
        var ret = getObject(arg0).metaKey;
        return ret;
    };
    imports.wbg.__wbg_button_d26f7934bec41270 = function(arg0) {
        var ret = getObject(arg0).button;
        return ret;
    };
    imports.wbg.__wbg_body_358bdbcdd48b1995 = function(arg0) {
        var ret = getObject(arg0).body;
        return isLikeNone(ret) ? 0 : addHeapObject(ret);
    };
    imports.wbg.__wbg_createElement_2b9dbef12990d2d6 = function() { return handleError(function (arg0, arg1, arg2) {
        var ret = getObject(arg0).createElement(getStringFromWasm0(arg1, arg2));
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_getElementById_97cfcb57d9ef46b3 = function(arg0, arg1, arg2) {
        var ret = getObject(arg0).getElementById(getStringFromWasm0(arg1, arg2));
        return isLikeNone(ret) ? 0 : addHeapObject(ret);
    };
    imports.wbg.__wbg_setid_500e596f870abbe4 = function(arg0, arg1, arg2) {
        getObject(arg0).id = getStringFromWasm0(arg1, arg2);
    };
    imports.wbg.__wbg_scrollLeft_0eab11639a2779e6 = function(arg0) {
        var ret = getObject(arg0).scrollLeft;
        return ret;
    };
    imports.wbg.__wbg_getBoundingClientRect_d3dd218166366f7b = function(arg0) {
        var ret = getObject(arg0).getBoundingClientRect();
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_instanceof_WebGlRenderingContext_c837a7a1630ab279 = function(arg0) {
        var ret = getObject(arg0) instanceof WebGLRenderingContext;
        return ret;
    };
    imports.wbg.__wbg_drawingBufferWidth_4e8473e7fe47b1fc = function(arg0) {
        var ret = getObject(arg0).drawingBufferWidth;
        return ret;
    };
    imports.wbg.__wbg_drawingBufferHeight_1993986ad8a5bf0c = function(arg0) {
        var ret = getObject(arg0).drawingBufferHeight;
        return ret;
    };
    imports.wbg.__wbg_bufferData_8dcff40bb7ea4e13 = function(arg0, arg1, arg2, arg3) {
        getObject(arg0).bufferData(arg1 >>> 0, getObject(arg2), arg3 >>> 0);
    };
    imports.wbg.__wbg_bufferData_c5a69efc65fccda8 = function(arg0, arg1, arg2, arg3, arg4) {
        getObject(arg0).bufferData(arg1 >>> 0, getArrayU8FromWasm0(arg2, arg3), arg4 >>> 0);
    };
    imports.wbg.__wbg_texImage2D_e7c24918556688fd = function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9, arg10) {
        getObject(arg0).texImage2D(arg1 >>> 0, arg2, arg3, arg4, arg5, arg6, arg7 >>> 0, arg8 >>> 0, arg9 === 0 ? undefined : getArrayU8FromWasm0(arg9, arg10));
    }, arguments) };
    imports.wbg.__wbg_activeTexture_b1a48defe102b8b4 = function(arg0, arg1) {
        getObject(arg0).activeTexture(arg1 >>> 0);
    };
    imports.wbg.__wbg_attachShader_b71a941dbde2f06a = function(arg0, arg1, arg2) {
        getObject(arg0).attachShader(getObject(arg1), getObject(arg2));
    };
    imports.wbg.__wbg_bindBuffer_c63c000cae2827a7 = function(arg0, arg1, arg2) {
        getObject(arg0).bindBuffer(arg1 >>> 0, getObject(arg2));
    };
    imports.wbg.__wbg_bindFramebuffer_8ac3fbda184a07cd = function(arg0, arg1, arg2) {
        getObject(arg0).bindFramebuffer(arg1 >>> 0, getObject(arg2));
    };
    imports.wbg.__wbg_bindTexture_d3e14d55f480c87c = function(arg0, arg1, arg2) {
        getObject(arg0).bindTexture(arg1 >>> 0, getObject(arg2));
    };
    imports.wbg.__wbg_blendFunc_db52a67e55f1f589 = function(arg0, arg1, arg2) {
        getObject(arg0).blendFunc(arg1 >>> 0, arg2 >>> 0);
    };
    imports.wbg.__wbg_clear_6035c51172b3fcdb = function(arg0, arg1) {
        getObject(arg0).clear(arg1 >>> 0);
    };
    imports.wbg.__wbg_clearColor_1bea576858a1b6ee = function(arg0, arg1, arg2, arg3, arg4) {
        getObject(arg0).clearColor(arg1, arg2, arg3, arg4);
    };
    imports.wbg.__wbg_compileShader_86034162d2750fa0 = function(arg0, arg1) {
        getObject(arg0).compileShader(getObject(arg1));
    };
    imports.wbg.__wbg_createBuffer_1a7d3ead6934c72b = function(arg0) {
        var ret = getObject(arg0).createBuffer();
        return isLikeNone(ret) ? 0 : addHeapObject(ret);
    };
    imports.wbg.__wbg_createFramebuffer_ea3800404236687e = function(arg0) {
        var ret = getObject(arg0).createFramebuffer();
        return isLikeNone(ret) ? 0 : addHeapObject(ret);
    };
    imports.wbg.__wbg_createProgram_a4b4dcbd12f5074e = function(arg0) {
        var ret = getObject(arg0).createProgram();
        return isLikeNone(ret) ? 0 : addHeapObject(ret);
    };
    imports.wbg.__wbg_createShader_ef171c856fb49871 = function(arg0, arg1) {
        var ret = getObject(arg0).createShader(arg1 >>> 0);
        return isLikeNone(ret) ? 0 : addHeapObject(ret);
    };
    imports.wbg.__wbg_createTexture_122b948b07df3423 = function(arg0) {
        var ret = getObject(arg0).createTexture();
        return isLikeNone(ret) ? 0 : addHeapObject(ret);
    };
    imports.wbg.__wbg_deleteBuffer_7df094eb78419aab = function(arg0, arg1) {
        getObject(arg0).deleteBuffer(getObject(arg1));
    };
    imports.wbg.__wbg_deleteFramebuffer_aa0ef382e4a51307 = function(arg0, arg1) {
        getObject(arg0).deleteFramebuffer(getObject(arg1));
    };
    imports.wbg.__wbg_deleteProgram_8a9cdc935784d06e = function(arg0, arg1) {
        getObject(arg0).deleteProgram(getObject(arg1));
    };
    imports.wbg.__wbg_deleteTexture_e3c76a05372155cc = function(arg0, arg1) {
        getObject(arg0).deleteTexture(getObject(arg1));
    };
    imports.wbg.__wbg_disable_a00ba808192589ac = function(arg0, arg1) {
        getObject(arg0).disable(arg1 >>> 0);
    };
    imports.wbg.__wbg_drawElements_5c538584e40dd38d = function(arg0, arg1, arg2, arg3, arg4) {
        getObject(arg0).drawElements(arg1 >>> 0, arg2, arg3 >>> 0, arg4);
    };
    imports.wbg.__wbg_enable_b68d0096e1cf7257 = function(arg0, arg1) {
        getObject(arg0).enable(arg1 >>> 0);
    };
    imports.wbg.__wbg_enableVertexAttribArray_817eb356245a0a23 = function(arg0, arg1) {
        getObject(arg0).enableVertexAttribArray(arg1 >>> 0);
    };
    imports.wbg.__wbg_framebufferTexture2D_8ba44ac97622f6e4 = function(arg0, arg1, arg2, arg3, arg4, arg5) {
        getObject(arg0).framebufferTexture2D(arg1 >>> 0, arg2 >>> 0, arg3 >>> 0, getObject(arg4), arg5);
    };
    imports.wbg.__wbg_getAttribLocation_41767c873c35a1db = function(arg0, arg1, arg2, arg3) {
        var ret = getObject(arg0).getAttribLocation(getObject(arg1), getStringFromWasm0(arg2, arg3));
        return ret;
    };
    imports.wbg.__wbg_getExtension_c5bac62c2df9b51f = function() { return handleError(function (arg0, arg1, arg2) {
        var ret = getObject(arg0).getExtension(getStringFromWasm0(arg1, arg2));
        return isLikeNone(ret) ? 0 : addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_getProgramInfoLog_1fb1ab4435c625b0 = function(arg0, arg1, arg2) {
        var ret = getObject(arg1).getProgramInfoLog(getObject(arg2));
        var ptr0 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        var len0 = WASM_VECTOR_LEN;
        getInt32Memory0()[arg0 / 4 + 1] = len0;
        getInt32Memory0()[arg0 / 4 + 0] = ptr0;
    };
    imports.wbg.__wbg_getProgramParameter_514228e301d883a1 = function(arg0, arg1, arg2) {
        var ret = getObject(arg0).getProgramParameter(getObject(arg1), arg2 >>> 0);
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_getShaderInfoLog_6c1b8ea071469485 = function(arg0, arg1, arg2) {
        var ret = getObject(arg1).getShaderInfoLog(getObject(arg2));
        var ptr0 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        var len0 = WASM_VECTOR_LEN;
        getInt32Memory0()[arg0 / 4 + 1] = len0;
        getInt32Memory0()[arg0 / 4 + 0] = ptr0;
    };
    imports.wbg.__wbg_getShaderParameter_9cb2e8dca5f6497d = function(arg0, arg1, arg2) {
        var ret = getObject(arg0).getShaderParameter(getObject(arg1), arg2 >>> 0);
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_getUniformLocation_c33ac369426e3f23 = function(arg0, arg1, arg2, arg3) {
        var ret = getObject(arg0).getUniformLocation(getObject(arg1), getStringFromWasm0(arg2, arg3));
        return isLikeNone(ret) ? 0 : addHeapObject(ret);
    };
    imports.wbg.__wbg_linkProgram_10db6bca776dad11 = function(arg0, arg1) {
        getObject(arg0).linkProgram(getObject(arg1));
    };
    imports.wbg.__wbg_pixelStorei_ccfe1ba5b9fcc3c7 = function(arg0, arg1, arg2) {
        getObject(arg0).pixelStorei(arg1 >>> 0, arg2);
    };
    imports.wbg.__wbg_scissor_76167a43e274fced = function(arg0, arg1, arg2, arg3, arg4) {
        getObject(arg0).scissor(arg1, arg2, arg3, arg4);
    };
    imports.wbg.__wbg_shaderSource_8d77f1b58d544eb9 = function(arg0, arg1, arg2, arg3) {
        getObject(arg0).shaderSource(getObject(arg1), getStringFromWasm0(arg2, arg3));
    };
    imports.wbg.__wbg_texParameteri_b2c6bdb098d2d85b = function(arg0, arg1, arg2, arg3) {
        getObject(arg0).texParameteri(arg1 >>> 0, arg2 >>> 0, arg3);
    };
    imports.wbg.__wbg_uniform1i_50cd6b8ea817127c = function(arg0, arg1, arg2) {
        getObject(arg0).uniform1i(getObject(arg1), arg2);
    };
    imports.wbg.__wbg_uniform2f_55d982ae38cc2fce = function(arg0, arg1, arg2, arg3) {
        getObject(arg0).uniform2f(getObject(arg1), arg2, arg3);
    };
    imports.wbg.__wbg_useProgram_39874f6f12a8845b = function(arg0, arg1) {
        getObject(arg0).useProgram(getObject(arg1));
    };
    imports.wbg.__wbg_vertexAttribPointer_7c93677ed5cd7ee2 = function(arg0, arg1, arg2, arg3, arg4, arg5, arg6) {
        getObject(arg0).vertexAttribPointer(arg1 >>> 0, arg2, arg3 >>> 0, arg4 !== 0, arg5, arg6);
    };
    imports.wbg.__wbg_viewport_fb7b9098d880a2e5 = function(arg0, arg1, arg2, arg3, arg4) {
        getObject(arg0).viewport(arg1, arg2, arg3, arg4);
    };
    imports.wbg.__wbg_error_2394084f0db4734f = function(arg0) {
        console.error(getObject(arg0));
    };
    imports.wbg.__wbg_log_9fc393c5050092d4 = function(arg0) {
        console.log(getObject(arg0));
    };
    imports.wbg.__wbg_warn_0293323340169279 = function(arg0) {
        console.warn(getObject(arg0));
    };
    imports.wbg.__wbg_scrollTop_c9303ca1fe14509f = function(arg0) {
        var ret = getObject(arg0).scrollTop;
        return ret;
    };
    imports.wbg.__wbg_hidden_1b6f88557fe67202 = function(arg0) {
        var ret = getObject(arg0).hidden;
        return ret;
    };
    imports.wbg.__wbg_sethidden_e6c617ca3019ed81 = function(arg0, arg1) {
        getObject(arg0).hidden = arg1 !== 0;
    };
    imports.wbg.__wbg_style_f582a03a2fedfede = function(arg0) {
        var ret = getObject(arg0).style;
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_offsetTop_c1da7eb6cebd2daa = function(arg0) {
        var ret = getObject(arg0).offsetTop;
        return ret;
    };
    imports.wbg.__wbg_offsetLeft_892cf630a6f9eee7 = function(arg0) {
        var ret = getObject(arg0).offsetLeft;
        return ret;
    };
    imports.wbg.__wbg_offsetWidth_be23990e54551338 = function(arg0) {
        var ret = getObject(arg0).offsetWidth;
        return ret;
    };
    imports.wbg.__wbg_blur_db758d6546528e8c = function() { return handleError(function (arg0) {
        getObject(arg0).blur();
    }, arguments) };
    imports.wbg.__wbg_focus_20606cbd8af94e2a = function() { return handleError(function (arg0) {
        getObject(arg0).focus();
    }, arguments) };
    imports.wbg.__wbg_writeText_c02542ef3f7286c5 = function(arg0, arg1, arg2) {
        var ret = getObject(arg0).writeText(getStringFromWasm0(arg1, arg2));
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_data_cf82a43e9f38735b = function(arg0, arg1) {
        var ret = getObject(arg1).data;
        var ptr0 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        var len0 = WASM_VECTOR_LEN;
        getInt32Memory0()[arg0 / 4 + 1] = len0;
        getInt32Memory0()[arg0 / 4 + 0] = ptr0;
    };
    imports.wbg.__wbg_getPropertyValue_40dcb14ac2e5dccf = function() { return handleError(function (arg0, arg1, arg2, arg3) {
        var ret = getObject(arg1).getPropertyValue(getStringFromWasm0(arg2, arg3));
        var ptr0 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        var len0 = WASM_VECTOR_LEN;
        getInt32Memory0()[arg0 / 4 + 1] = len0;
        getInt32Memory0()[arg0 / 4 + 0] = ptr0;
    }, arguments) };
    imports.wbg.__wbg_setProperty_bb5ef041aa234119 = function() { return handleError(function (arg0, arg1, arg2, arg3, arg4) {
        getObject(arg0).setProperty(getStringFromWasm0(arg1, arg2), getStringFromWasm0(arg3, arg4));
    }, arguments) };
    imports.wbg.__wbg_type_2a5cf551e38cb11b = function(arg0, arg1) {
        var ret = getObject(arg1).type;
        var ptr0 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        var len0 = WASM_VECTOR_LEN;
        getInt32Memory0()[arg0 / 4 + 1] = len0;
        getInt32Memory0()[arg0 / 4 + 0] = ptr0;
    };
    imports.wbg.__wbg_addEventListener_ba672fd0a86ea7c0 = function() { return handleError(function (arg0, arg1, arg2, arg3) {
        getObject(arg0).addEventListener(getStringFromWasm0(arg1, arg2), getObject(arg3));
    }, arguments) };
    imports.wbg.__wbg_deltaX_4fc71517926b97b2 = function(arg0) {
        var ret = getObject(arg0).deltaX;
        return ret;
    };
    imports.wbg.__wbg_deltaY_f6f00b37939ded4f = function(arg0) {
        var ret = getObject(arg0).deltaY;
        return ret;
    };
    imports.wbg.__wbg_deltaMode_5841e202184877c4 = function(arg0) {
        var ret = getObject(arg0).deltaMode;
        return ret;
    };
    imports.wbg.__wbg_dataTransfer_d1c30982d8dfb155 = function(arg0) {
        var ret = getObject(arg0).dataTransfer;
        return isLikeNone(ret) ? 0 : addHeapObject(ret);
    };
    imports.wbg.__wbg_matches_081caf14c84df8f4 = function(arg0) {
        var ret = getObject(arg0).matches;
        return ret;
    };
    imports.wbg.__wbg_now_e3cde1a07a4d3e37 = function(arg0) {
        var ret = getObject(arg0).now();
        return ret;
    };
    imports.wbg.__wbg_identifier_94bb7db9ff642dbd = function(arg0) {
        var ret = getObject(arg0).identifier;
        return ret;
    };
    imports.wbg.__wbg_pageX_343494be0964e93e = function(arg0) {
        var ret = getObject(arg0).pageX;
        return ret;
    };
    imports.wbg.__wbg_pageY_fffe4590f237651b = function(arg0) {
        var ret = getObject(arg0).pageY;
        return ret;
    };
    imports.wbg.__wbg_force_71ff5e1979817b51 = function(arg0) {
        var ret = getObject(arg0).force;
        return ret;
    };
    imports.wbg.__wbg_instanceof_HtmlInputElement_d103e944c9f64251 = function(arg0) {
        var ret = getObject(arg0) instanceof HTMLInputElement;
        return ret;
    };
    imports.wbg.__wbg_setautofocus_f2429aea3879faea = function(arg0, arg1) {
        getObject(arg0).autofocus = arg1 !== 0;
    };
    imports.wbg.__wbg_setsize_88515f059ab92db2 = function(arg0, arg1) {
        getObject(arg0).size = arg1 >>> 0;
    };
    imports.wbg.__wbg_value_a507ac01ff7d8403 = function(arg0, arg1) {
        var ret = getObject(arg1).value;
        var ptr0 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        var len0 = WASM_VECTOR_LEN;
        getInt32Memory0()[arg0 / 4 + 1] = len0;
        getInt32Memory0()[arg0 / 4 + 0] = ptr0;
    };
    imports.wbg.__wbg_setvalue_24673032b3b48825 = function(arg0, arg1, arg2) {
        getObject(arg0).value = getStringFromWasm0(arg1, arg2);
    };
    imports.wbg.__wbg_newnoargs_ac91a24e57fcaec8 = function(arg0, arg1) {
        var ret = new Function(getStringFromWasm0(arg0, arg1));
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_call_9e1eb05d905a21d9 = function() { return handleError(function (arg0, arg1) {
        var ret = getObject(arg0).call(getObject(arg1));
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_getHours_535b93f4e08fb2cd = function(arg0) {
        var ret = getObject(arg0).getHours();
        return ret;
    };
    imports.wbg.__wbg_getMilliseconds_907462bd204bf4d1 = function(arg0) {
        var ret = getObject(arg0).getMilliseconds();
        return ret;
    };
    imports.wbg.__wbg_getMinutes_3e6f317a384509e2 = function(arg0) {
        var ret = getObject(arg0).getMinutes();
        return ret;
    };
    imports.wbg.__wbg_getSeconds_560927ea60ab6b9d = function(arg0) {
        var ret = getObject(arg0).getSeconds();
        return ret;
    };
    imports.wbg.__wbg_new0_d38255ac176a2e53 = function() {
        var ret = new Date();
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_resolve_89251e936a5e00ac = function(arg0) {
        var ret = Promise.resolve(getObject(arg0));
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_then_fe720822c4da5711 = function(arg0, arg1) {
        var ret = getObject(arg0).then(getObject(arg1));
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_then_f040b93e57f11d67 = function(arg0, arg1, arg2) {
        var ret = getObject(arg0).then(getObject(arg1), getObject(arg2));
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_self_bce917bbd61b0be0 = function() { return handleError(function () {
        var ret = self.self;
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_window_08048ce184ae3496 = function() { return handleError(function () {
        var ret = window.window;
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_globalThis_d6f1ff349571af81 = function() { return handleError(function () {
        var ret = globalThis.globalThis;
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_global_63b22b64d239db75 = function() { return handleError(function () {
        var ret = global.global;
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbindgen_is_undefined = function(arg0) {
        var ret = getObject(arg0) === undefined;
        return ret;
    };
    imports.wbg.__wbg_buffer_fbad716641c158a5 = function(arg0) {
        var ret = getObject(arg0).buffer;
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_new_4d58d9eda3702b20 = function(arg0) {
        var ret = new Int16Array(getObject(arg0));
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_new_c9e78bd69716df92 = function(arg0) {
        var ret = new Uint8Array(getObject(arg0));
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_set_2fd4486048716f38 = function(arg0, arg1, arg2) {
        getObject(arg0).set(getObject(arg1), arg2 >>> 0);
    };
    imports.wbg.__wbg_length_82dd1e63e9c75f09 = function(arg0) {
        var ret = getObject(arg0).length;
        return ret;
    };
    imports.wbg.__wbg_new_30824fe670ec89a1 = function(arg0) {
        var ret = new Float32Array(getObject(arg0));
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_subarray_0e612371989bdbc6 = function(arg0, arg1, arg2) {
        var ret = getObject(arg0).subarray(arg1 >>> 0, arg2 >>> 0);
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_subarray_e80c85d931be89c4 = function(arg0, arg1, arg2) {
        var ret = getObject(arg0).subarray(arg1 >>> 0, arg2 >>> 0);
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_subarray_064fb2d7414f32a1 = function(arg0, arg1, arg2) {
        var ret = getObject(arg0).subarray(arg1 >>> 0, arg2 >>> 0);
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_instanceof_Memory_aaa95e49eedcf179 = function(arg0) {
        var ret = getObject(arg0) instanceof WebAssembly.Memory;
        return ret;
    };
    imports.wbg.__wbindgen_debug_string = function(arg0, arg1) {
        var ret = debugString(getObject(arg1));
        var ptr0 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        var len0 = WASM_VECTOR_LEN;
        getInt32Memory0()[arg0 / 4 + 1] = len0;
        getInt32Memory0()[arg0 / 4 + 0] = ptr0;
    };
    imports.wbg.__wbindgen_throw = function(arg0, arg1) {
        throw new Error(getStringFromWasm0(arg0, arg1));
    };
    imports.wbg.__wbindgen_rethrow = function(arg0) {
        throw takeObject(arg0);
    };
    imports.wbg.__wbindgen_memory = function() {
        var ret = wasm.memory;
        return addHeapObject(ret);
    };
    imports.wbg.__wbindgen_closure_wrapper608 = function(arg0, arg1, arg2) {
        var ret = makeMutClosure(arg0, arg1, 117, __wbg_adapter_24);
        return addHeapObject(ret);
    };
    imports.wbg.__wbindgen_closure_wrapper609 = function(arg0, arg1, arg2) {
        var ret = makeMutClosure(arg0, arg1, 117, __wbg_adapter_27);
        return addHeapObject(ret);
    };
    imports.wbg.__wbindgen_closure_wrapper612 = function(arg0, arg1, arg2) {
        var ret = makeMutClosure(arg0, arg1, 117, __wbg_adapter_30);
        return addHeapObject(ret);
    };
    imports.wbg.__wbindgen_closure_wrapper614 = function(arg0, arg1, arg2) {
        var ret = makeMutClosure(arg0, arg1, 117, __wbg_adapter_33);
        return addHeapObject(ret);
    };
    imports.wbg.__wbindgen_closure_wrapper616 = function(arg0, arg1, arg2) {
        var ret = makeMutClosure(arg0, arg1, 117, __wbg_adapter_36);
        return addHeapObject(ret);
    };
    imports.wbg.__wbindgen_closure_wrapper618 = function(arg0, arg1, arg2) {
        var ret = makeMutClosure(arg0, arg1, 117, __wbg_adapter_39);
        return addHeapObject(ret);
    };
    imports.wbg.__wbindgen_closure_wrapper620 = function(arg0, arg1, arg2) {
        var ret = makeMutClosure(arg0, arg1, 117, __wbg_adapter_42);
        return addHeapObject(ret);
    };
    imports.wbg.__wbindgen_closure_wrapper622 = function(arg0, arg1, arg2) {
        var ret = makeMutClosure(arg0, arg1, 117, __wbg_adapter_45);
        return addHeapObject(ret);
    };
    imports.wbg.__wbindgen_closure_wrapper624 = function(arg0, arg1, arg2) {
        var ret = makeMutClosure(arg0, arg1, 117, __wbg_adapter_48);
        return addHeapObject(ret);
    };
    imports.wbg.__wbindgen_closure_wrapper626 = function(arg0, arg1, arg2) {
        var ret = makeMutClosure(arg0, arg1, 117, __wbg_adapter_51);
        return addHeapObject(ret);
    };
    imports.wbg.__wbindgen_closure_wrapper931 = function(arg0, arg1, arg2) {
        var ret = makeMutClosure(arg0, arg1, 243, __wbg_adapter_54);
        return addHeapObject(ret);
    };

    if (typeof input === 'string' || (typeof Request === 'function' && input instanceof Request) || (typeof URL === 'function' && input instanceof URL)) {
        input = fetch(input);
    }



    const { instance, module } = await load(await input, imports);

    wasm = instance.exports;
    init.__wbindgen_wasm_module = module;

    return wasm;
}

wasm_bindgen = Object.assign(init, __exports);

})();
