let W=0,R=null,T=`undefined`,$=`boolean`,a5=1230,a0=`string`,S=1,a1=`Object`,U=`utf-8`,_=`number`,a3=4,Y=`function`,a2=16,a4=344,P=Array,V=Error,Z=Int32Array,X=Uint8Array,Q=undefined;var u=(a=>{const b=typeof a;if(b==_||b==$||a==R){return `${a}`};if(b==a0){return `"${a}"`};if(b==`symbol`){const b=a.description;if(b==R){return `Symbol`}else{return `Symbol(${b})`}};if(b==Y){const b=a.name;if(typeof b==a0&&b.length>W){return `Function(${b})`}else{return `Function`}};if(P.isArray(a)){const b=a.length;let c=`[`;if(b>W){c+=u(a[W])};for(let d=S;d<b;d++){c+=`, `+ u(a[d])};c+=`]`;return c};const c=/\[object ([^\]]+)\]/.exec(toString.call(a));let d;if(c.length>S){d=c[S]}else{return toString.call(a)};if(d==a1){try{return `Object(`+ JSON.stringify(a)+ `)`}catch(a){return a1}};if(a instanceof V){return `${a.name}: ${a.message}\n${a.stack}`};return d});var C=((b,c)=>{try{const g=a.__wbindgen_add_to_stack_pointer(-a2);a.wasm_bindgen__convert__closures__invoke0_mut__he8f9452136334ee6(g,b,c);var d=r()[g/a3+ W];var e=r()[g/a3+ S];if(e){throw f(d)}}finally{a.__wbindgen_add_to_stack_pointer(a2)}});var L=((a,b)=>{});var y=((b,c,d,e)=>{const f=o(d,a.__wbindgen_malloc,a.__wbindgen_realloc);const g=l;const h=o(e,a.__wbindgen_malloc,a.__wbindgen_realloc);const i=l;a.wasm_bindgen__convert__closures__invoke2__h6608f0a675891a56(b,c,f,g,h,i)});var H=(a=>()=>{throw new V(`${a} is not defined`)});function I(b,c){try{return b.apply(this,c)}catch(b){a.__wbindgen_exn_store(g(b))}}var B=((b,c,d,e)=>{const f=o(d,a.__wbindgen_malloc,a.__wbindgen_realloc);const g=l;const h=z(e,a.__wbindgen_malloc);const i=l;a.wasm_bindgen__convert__closures__invoke2_mut__hb9c42cb2828ab2de(b,c,f,g,h,i)});var A=((b,c,d)=>{const e=z(d,a.__wbindgen_malloc);const f=l;a.wasm_bindgen__convert__closures__invoke1__hc14e6b2725146a6a(b,c,e,f)});var p=(a=>a===Q||a===R);var c=(a=>b[a]);var z=((a,b)=>{const c=b(a.length*S,S)>>>W;j().set(a,c/S);l=a.length;return c});var K=(()=>{const b={};b.wbg={};b.wbg.__wbindgen_object_drop_ref=(a=>{f(a)});b.wbg.__wbindgen_cb_drop=(a=>{const b=f(a).original;if(b.cnt--==S){b.a=W;return !0};const c=!1;return c});b.wbg.__wbg_openfile_9f5fd7fe0055f877=((a,b,c,d,e)=>{open_file(k(a,b),f(c),d!==W,e!==W)});b.wbg.__wbindgen_object_clone_ref=(a=>{const b=c(a);return g(b)});b.wbg.__wbg_savefile_9e7433c1bb68e28c=((a,b,c,d)=>{save_file(G(a,b),k(c,d))});b.wbg.__wbg_loadurl_4d3a8cb4b589bf7e=(b=>{const c=load_url();const d=o(c,a.__wbindgen_malloc,a.__wbindgen_realloc);const e=l;r()[b/a3+ S]=e;r()[b/a3+ W]=d});b.wbg.__wbg_loadingfinished_f212398285aa1896=typeof loading_finished==Y?loading_finished:H(`loading_finished`);b.wbg.__wbindgen_string_new=((a,b)=>{const c=k(a,b);return g(c)});b.wbg.__wbg_log_c9486ca5d8e2cbe8=((b,c)=>{let d;let e;try{d=b;e=c;console.log(k(b,c))}finally{a.__wbindgen_free(d,e,S)}});b.wbg.__wbg_log_aba5996d9bde071f=((b,c,d,e,f,g,h,i)=>{let j;let l;try{j=b;l=c;console.log(k(b,c),k(d,e),k(f,g),k(h,i))}finally{a.__wbindgen_free(j,l,S)}});b.wbg.__wbg_mark_40e050a77cc39fea=((a,b)=>{performance.mark(k(a,b))});b.wbg.__wbg_measure_aa7a73f17813f708=function(){return I(((b,c,d,e)=>{let f;let g;let h;let i;try{f=b;g=c;h=d;i=e;performance.measure(k(b,c),k(d,e))}finally{a.__wbindgen_free(f,g,S);a.__wbindgen_free(h,i,S)}}),arguments)};b.wbg.__wbg_new_abda76e883ba8a5f=(()=>{const a=new V();return g(a)});b.wbg.__wbg_stack_658279fe44541cf6=((b,d)=>{const e=c(d).stack;const f=o(e,a.__wbindgen_malloc,a.__wbindgen_realloc);const g=l;r()[b/a3+ S]=g;r()[b/a3+ W]=f});b.wbg.__wbg_error_f851667af71bcfc6=((b,c)=>{let d;let e;try{d=b;e=c;console.error(k(b,c))}finally{a.__wbindgen_free(d,e,S)}});b.wbg.__wbg_alert_fb3418a0398674a2=((a,b)=>{alert(k(a,b))});b.wbg.__wbg_confirm_1047833407807376=((a,b)=>{const c=confirm(k(a,b));return c});b.wbg.__wbg_crypto_d05b68a3572bb8ca=(a=>{const b=c(a).crypto;return g(b)});b.wbg.__wbindgen_is_object=(a=>{const b=c(a);const d=typeof b===`object`&&b!==R;return d});b.wbg.__wbg_process_b02b3570280d0366=(a=>{const b=c(a).process;return g(b)});b.wbg.__wbg_versions_c1cb42213cedf0f5=(a=>{const b=c(a).versions;return g(b)});b.wbg.__wbg_node_43b1089f407e4ec2=(a=>{const b=c(a).node;return g(b)});b.wbg.__wbindgen_is_string=(a=>{const b=typeof c(a)===a0;return b});b.wbg.__wbg_msCrypto_10fc94afee92bd76=(a=>{const b=c(a).msCrypto;return g(b)});b.wbg.__wbg_require_9a7e0f667ead4995=function(){return I((()=>{const a=module.require;return g(a)}),arguments)};b.wbg.__wbindgen_is_function=(a=>{const b=typeof c(a)===Y;return b});b.wbg.__wbg_randomFillSync_b70ccbdf4926a99d=function(){return I(((a,b)=>{c(a).randomFillSync(f(b))}),arguments)};b.wbg.__wbg_getRandomValues_7e42b4fb8779dc6d=function(){return I(((a,b)=>{c(a).getRandomValues(c(b))}),arguments)};b.wbg.__wbindgen_string_get=((b,d)=>{const e=c(d);const f=typeof e===a0?e:Q;var g=p(f)?W:o(f,a.__wbindgen_malloc,a.__wbindgen_realloc);var h=l;r()[b/a3+ S]=h;r()[b/a3+ W]=g});b.wbg.__wbg_error_14d05ac44618f166=((b,c)=>{let d;let e;try{d=b;e=c;console.error(k(b,c))}finally{a.__wbindgen_free(d,e,S)}});b.wbg.__wbg_new_5558faf900c28862=(()=>{const a=new V();return g(a)});b.wbg.__wbg_stack_0ad674156f31e4a6=((b,d)=>{const e=c(d).stack;const f=o(e,a.__wbindgen_malloc,a.__wbindgen_realloc);const g=l;r()[b/a3+ S]=g;r()[b/a3+ W]=f});b.wbg.__wbindgen_number_get=((a,b)=>{const d=c(b);const e=typeof d===_?d:Q;t()[a/8+ S]=p(e)?W:e;r()[a/a3+ W]=!p(e)});b.wbg.__wbg_queueMicrotask_118eeb525d584d9a=(a=>{queueMicrotask(c(a))});b.wbg.__wbg_queueMicrotask_26a89c14c53809c0=(a=>{const b=c(a).queueMicrotask;return g(b)});b.wbg.__wbindgen_boolean_get=(a=>{const b=c(a);const d=typeof b===$?(b?S:W):2;return d});b.wbg.__wbg_instanceof_WebGl2RenderingContext_92adf5bbd2568b71=(a=>{let b;try{b=c(a) instanceof WebGL2RenderingContext}catch(a){b=!1}const d=b;return d});b.wbg.__wbg_bindVertexArray_2a70aed123353597=((a,b)=>{c(a).bindVertexArray(c(b))});b.wbg.__wbg_bufferData_eab63186e3e72d98=((a,b,d,e)=>{c(a).bufferData(b>>>W,c(d),e>>>W)});b.wbg.__wbg_createVertexArray_761ba22fc5da3ad7=(a=>{const b=c(a).createVertexArray();return p(b)?W:g(b)});b.wbg.__wbg_texImage2D_1159b898accc2807=function(){return I(((a,b,d,e,f,g,h,i,j,k)=>{c(a).texImage2D(b>>>W,d,e,f,g,h,i>>>W,j>>>W,c(k))}),arguments)};b.wbg.__wbg_texSubImage2D_33018bcf2de70890=function(){return I(((a,b,d,e,f,g,h,i,j,k)=>{c(a).texSubImage2D(b>>>W,d,e,f,g,h,i>>>W,j>>>W,c(k))}),arguments)};b.wbg.__wbg_texSubImage2D_b97aa5ddc0162314=function(){return I(((a,b,d,e,f,g,h,i,j,k)=>{c(a).texSubImage2D(b>>>W,d,e,f,g,h,i>>>W,j>>>W,k)}),arguments)};b.wbg.__wbg_activeTexture_02d56293bce2f613=((a,b)=>{c(a).activeTexture(b>>>W)});b.wbg.__wbg_attachShader_70c3f88b777a0a54=((a,b,d)=>{c(a).attachShader(c(b),c(d))});b.wbg.__wbg_bindBuffer_ac939bcab5249160=((a,b,d)=>{c(a).bindBuffer(b>>>W,c(d))});b.wbg.__wbg_bindTexture_e28115f3ea3da6d2=((a,b,d)=>{c(a).bindTexture(b>>>W,c(d))});b.wbg.__wbg_blendEquationSeparate_457e81472270e23c=((a,b,d)=>{c(a).blendEquationSeparate(b>>>W,d>>>W)});b.wbg.__wbg_blendFuncSeparate_b6a96b8e26e75171=((a,b,d,e,f)=>{c(a).blendFuncSeparate(b>>>W,d>>>W,e>>>W,f>>>W)});b.wbg.__wbg_clear_40335e7899ec7759=((a,b)=>{c(a).clear(b>>>W)});b.wbg.__wbg_clearColor_b48ee3ca810de959=((a,b,d,e,f)=>{c(a).clearColor(b,d,e,f)});b.wbg.__wbg_colorMask_743f2bbb6e3fb4e5=((a,b,d,e,f)=>{c(a).colorMask(b!==W,d!==W,e!==W,f!==W)});b.wbg.__wbg_compileShader_bdfb3d5a3ad59498=((a,b)=>{c(a).compileShader(c(b))});b.wbg.__wbg_createBuffer_a95c59cc2c1750e7=(a=>{const b=c(a).createBuffer();return p(b)?W:g(b)});b.wbg.__wbg_createProgram_0a7670ed33f06d97=(a=>{const b=c(a).createProgram();return p(b)?W:g(b)});b.wbg.__wbg_createShader_119ffcdb1667f405=((a,b)=>{const d=c(a).createShader(b>>>W);return p(d)?W:g(d)});b.wbg.__wbg_createTexture_4f0c3c77df4bde11=(a=>{const b=c(a).createTexture();return p(b)?W:g(b)});b.wbg.__wbg_deleteBuffer_b8aaa61f9bb13617=((a,b)=>{c(a).deleteBuffer(c(b))});b.wbg.__wbg_deleteProgram_d90e44574acb8018=((a,b)=>{c(a).deleteProgram(c(b))});b.wbg.__wbg_deleteShader_5ec1e25476df2da0=((a,b)=>{c(a).deleteShader(c(b))});b.wbg.__wbg_deleteTexture_554c30847d340929=((a,b)=>{c(a).deleteTexture(c(b))});b.wbg.__wbg_detachShader_5fe9df16c9007fca=((a,b,d)=>{c(a).detachShader(c(b),c(d))});b.wbg.__wbg_disable_f68719f70ddfb5b8=((a,b)=>{c(a).disable(b>>>W)});b.wbg.__wbg_disableVertexAttribArray_557393d91e187e24=((a,b)=>{c(a).disableVertexAttribArray(b>>>W)});b.wbg.__wbg_drawElements_a3781a76e2ccb054=((a,b,d,e,f)=>{c(a).drawElements(b>>>W,d,e>>>W,f)});b.wbg.__wbg_enable_6dab9d5278ba15e2=((a,b)=>{c(a).enable(b>>>W)});b.wbg.__wbg_enableVertexAttribArray_c2bfb733e87824c8=((a,b)=>{c(a).enableVertexAttribArray(b>>>W)});b.wbg.__wbg_getAttribLocation_cab9273a8063f57a=((a,b,d,e)=>{const f=c(a).getAttribLocation(c(b),k(d,e));return f});b.wbg.__wbg_getError_b3d74583648031ab=(a=>{const b=c(a).getError();return b});b.wbg.__wbg_getExtension_25430e0ed157fcf8=function(){return I(((a,b,d)=>{const e=c(a).getExtension(k(b,d));return p(e)?W:g(e)}),arguments)};b.wbg.__wbg_getParameter_b282105ca8420119=function(){return I(((a,b)=>{const d=c(a).getParameter(b>>>W);return g(d)}),arguments)};b.wbg.__wbg_getProgramInfoLog_110f43b4125782e9=((b,d,e)=>{const f=c(d).getProgramInfoLog(c(e));var g=p(f)?W:o(f,a.__wbindgen_malloc,a.__wbindgen_realloc);var h=l;r()[b/a3+ S]=h;r()[b/a3+ W]=g});b.wbg.__wbg_getProgramParameter_22b3f1c8d913cd2c=((a,b,d)=>{const e=c(a).getProgramParameter(c(b),d>>>W);return g(e)});b.wbg.__wbg_getShaderInfoLog_562b1447e7c24866=((b,d,e)=>{const f=c(d).getShaderInfoLog(c(e));var g=p(f)?W:o(f,a.__wbindgen_malloc,a.__wbindgen_realloc);var h=l;r()[b/a3+ S]=h;r()[b/a3+ W]=g});b.wbg.__wbg_getShaderParameter_58d3b34afa9db13b=((a,b,d)=>{const e=c(a).getShaderParameter(c(b),d>>>W);return g(e)});b.wbg.__wbg_getSupportedExtensions_1a007030d26efba5=(a=>{const b=c(a).getSupportedExtensions();return p(b)?W:g(b)});b.wbg.__wbg_getUniformLocation_7b435a76db4f3128=((a,b,d,e)=>{const f=c(a).getUniformLocation(c(b),k(d,e));return p(f)?W:g(f)});b.wbg.__wbg_linkProgram_e170ffe0b8242136=((a,b)=>{c(a).linkProgram(c(b))});b.wbg.__wbg_pixelStorei_6be3fc7114b690b8=((a,b,d)=>{c(a).pixelStorei(b>>>W,d)});b.wbg.__wbg_scissor_27cb154cc9864444=((a,b,d,e,f)=>{c(a).scissor(b,d,e,f)});b.wbg.__wbg_shaderSource_e12efd3a2bf3413d=((a,b,d,e)=>{c(a).shaderSource(c(b),k(d,e))});b.wbg.__wbg_texParameteri_f5c0d085b77931dd=((a,b,d,e)=>{c(a).texParameteri(b>>>W,d>>>W,e)});b.wbg.__wbg_uniform1i_1fd90743f7b78faa=((a,b,d)=>{c(a).uniform1i(c(b),d)});b.wbg.__wbg_uniform2f_e5d4fed81577da9b=((a,b,d,e)=>{c(a).uniform2f(c(b),d,e)});b.wbg.__wbg_useProgram_53de6b084c4780ce=((a,b)=>{c(a).useProgram(c(b))});b.wbg.__wbg_vertexAttribPointer_3133080603a92d4c=((a,b,d,e,f,g,h)=>{c(a).vertexAttribPointer(b>>>W,d,e>>>W,f!==W,g,h)});b.wbg.__wbg_viewport_afd5166081d009b2=((a,b,d,e,f)=>{c(a).viewport(b,d,e,f)});b.wbg.__wbg_instanceof_Window_99dc9805eaa2614b=(a=>{let b;try{b=c(a) instanceof Window}catch(a){b=!1}const d=b;return d});b.wbg.__wbg_document_5257b70811e953c0=(a=>{const b=c(a).document;return p(b)?W:g(b)});b.wbg.__wbg_location_0f233324e8e8c699=(a=>{const b=c(a).location;return g(b)});b.wbg.__wbg_navigator_910cca0226b70083=(a=>{const b=c(a).navigator;return g(b)});b.wbg.__wbg_innerHeight_dc4c81e04e8bc294=function(){return I((a=>{const b=c(a).innerHeight;return g(b)}),arguments)};b.wbg.__wbg_devicePixelRatio_93bac98af723c7ba=(a=>{const b=c(a).devicePixelRatio;return b});b.wbg.__wbg_localStorage_318b1c4f106a46f9=function(){return I((a=>{const b=c(a).localStorage;return p(b)?W:g(b)}),arguments)};b.wbg.__wbg_performance_698febdfb8f1f470=(a=>{const b=c(a).performance;return p(b)?W:g(b)});b.wbg.__wbg_matchMedia_fed5c8e73cf148cf=function(){return I(((a,b,d)=>{const e=c(a).matchMedia(k(b,d));return p(e)?W:g(e)}),arguments)};b.wbg.__wbg_open_0aa18467f0bb625e=function(){return I(((a,b,d,e,f)=>{const h=c(a).open(k(b,d),k(e,f));return p(h)?W:g(h)}),arguments)};b.wbg.__wbg_requestAnimationFrame_1820a8e6b645ec5a=function(){return I(((a,b)=>{const d=c(a).requestAnimationFrame(c(b));return d}),arguments)};b.wbg.__wbg_clearInterval_9886eebcc6575e58=((a,b)=>{c(a).clearInterval(b)});b.wbg.__wbg_setTimeout_bd20251bb242e262=function(){return I(((a,b,d)=>{const e=c(a).setTimeout(c(b),d);return e}),arguments)};b.wbg.__wbg_setid_4a30be2ea97a37dd=((a,b,d)=>{c(a).id=k(b,d)});b.wbg.__wbg_scrollLeft_d6eb4c6a0a6417b2=(a=>{const b=c(a).scrollLeft;return b});b.wbg.__wbg_clientWidth_63a18f3f1c0d50b9=(a=>{const b=c(a).clientWidth;return b});b.wbg.__wbg_clientHeight_12bebacfbf7ddf82=(a=>{const b=c(a).clientHeight;return b});b.wbg.__wbg_getBoundingClientRect_f3f6eb39f24c1bb0=(a=>{const b=c(a).getBoundingClientRect();return g(b)});b.wbg.__wbg_setAttribute_0918ea45d5a1c663=function(){return I(((a,b,d,e,f)=>{c(a).setAttribute(k(b,d),k(e,f))}),arguments)};b.wbg.__wbg_remove_ed2f62f1a8be044b=(a=>{c(a).remove()});b.wbg.__wbg_append_459bddb5f3a5b5fa=function(){return I(((a,b)=>{c(a).append(c(b))}),arguments)};b.wbg.__wbg_body_3eb73da919b867a1=(a=>{const b=c(a).body;return p(b)?W:g(b)});b.wbg.__wbg_createElement_1a136faad4101f43=function(){return I(((a,b,d)=>{const e=c(a).createElement(k(b,d));return g(e)}),arguments)};b.wbg.__wbg_getElementById_00904c7c4a32c23b=((a,b,d)=>{const e=c(a).getElementById(k(b,d));return p(e)?W:g(e)});b.wbg.__wbg_instanceof_HtmlElement_430cfa09315574cc=(a=>{let b;try{b=c(a) instanceof HTMLElement}catch(a){b=!1}const d=b;return d});b.wbg.__wbg_scrollTop_b8364983aece464a=(a=>{const b=c(a).scrollTop;return b});b.wbg.__wbg_hidden_445daefa35729d27=(a=>{const b=c(a).hidden;return b});b.wbg.__wbg_sethidden_a1bed94b94610e67=((a,b)=>{c(a).hidden=b!==W});b.wbg.__wbg_style_b32d5cb9a6bd4720=(a=>{const b=c(a).style;return g(b)});b.wbg.__wbg_offsetTop_f17e37517e25eb43=(a=>{const b=c(a).offsetTop;return b});b.wbg.__wbg_offsetLeft_0d0f84745a0af686=(a=>{const b=c(a).offsetLeft;return b});b.wbg.__wbg_offsetWidth_d131cad586641a97=(a=>{const b=c(a).offsetWidth;return b});b.wbg.__wbg_offsetHeight_1441e9cf0a410559=(a=>{const b=c(a).offsetHeight;return b});b.wbg.__wbg_blur_3de7a3848d6d481c=function(){return I((a=>{c(a).blur()}),arguments)};b.wbg.__wbg_focus_623326ec4eefd224=function(){return I((a=>{c(a).focus()}),arguments)};b.wbg.__wbg_instanceof_WebGlRenderingContext_7515fd5b9abf4249=(a=>{let b;try{b=c(a) instanceof WebGLRenderingContext}catch(a){b=!1}const d=b;return d});b.wbg.__wbg_bufferData_b2e68fdc1fd1e94b=((a,b,d,e)=>{c(a).bufferData(b>>>W,c(d),e>>>W)});b.wbg.__wbg_texImage2D_9cd1931c442b03ad=function(){return I(((a,b,d,e,f,g,h,i,j,k)=>{c(a).texImage2D(b>>>W,d,e,f,g,h,i>>>W,j>>>W,c(k))}),arguments)};b.wbg.__wbg_texSubImage2D_d23a3ec1fa60bdaf=function(){return I(((a,b,d,e,f,g,h,i,j,k)=>{c(a).texSubImage2D(b>>>W,d,e,f,g,h,i>>>W,j>>>W,c(k))}),arguments)};b.wbg.__wbg_activeTexture_3748123e1becf07d=((a,b)=>{c(a).activeTexture(b>>>W)});b.wbg.__wbg_attachShader_cfbbdefc08a0422f=((a,b,d)=>{c(a).attachShader(c(b),c(d))});b.wbg.__wbg_bindBuffer_3f166cc2f502fc09=((a,b,d)=>{c(a).bindBuffer(b>>>W,c(d))});b.wbg.__wbg_bindTexture_be92cdd3f162b4f9=((a,b,d)=>{c(a).bindTexture(b>>>W,c(d))});b.wbg.__wbg_blendEquationSeparate_33f23a57d77e8079=((a,b,d)=>{c(a).blendEquationSeparate(b>>>W,d>>>W)});b.wbg.__wbg_blendFuncSeparate_52fdb0f1fbf57928=((a,b,d,e,f)=>{c(a).blendFuncSeparate(b>>>W,d>>>W,e>>>W,f>>>W)});b.wbg.__wbg_clear_af4278a00382d3ce=((a,b)=>{c(a).clear(b>>>W)});b.wbg.__wbg_clearColor_9a45e2200c61a8f2=((a,b,d,e,f)=>{c(a).clearColor(b,d,e,f)});b.wbg.__wbg_colorMask_57603facaeb6e2e3=((a,b,d,e,f)=>{c(a).colorMask(b!==W,d!==W,e!==W,f!==W)});b.wbg.__wbg_compileShader_be824cfad43331b8=((a,b)=>{c(a).compileShader(c(b))});b.wbg.__wbg_createBuffer_90bf79c414ad4956=(a=>{const b=c(a).createBuffer();return p(b)?W:g(b)});b.wbg.__wbg_createProgram_983b87cad6d06768=(a=>{const b=c(a).createProgram();return p(b)?W:g(b)});b.wbg.__wbg_createShader_896229165c5a11d4=((a,b)=>{const d=c(a).createShader(b>>>W);return p(d)?W:g(d)});b.wbg.__wbg_createTexture_b77eefdce0bb2c55=(a=>{const b=c(a).createTexture();return p(b)?W:g(b)});b.wbg.__wbg_deleteBuffer_d70596808095dac2=((a,b)=>{c(a).deleteBuffer(c(b))});b.wbg.__wbg_deleteProgram_8447c337271aa934=((a,b)=>{c(a).deleteProgram(c(b))});b.wbg.__wbg_deleteShader_322b059ad560664a=((a,b)=>{c(a).deleteShader(c(b))});b.wbg.__wbg_deleteTexture_bbda7cb554bc12b9=((a,b)=>{c(a).deleteTexture(c(b))});b.wbg.__wbg_detachShader_1faf06c8a1000e58=((a,b,d)=>{c(a).detachShader(c(b),c(d))});b.wbg.__wbg_disable_57e8624c865bd654=((a,b)=>{c(a).disable(b>>>W)});b.wbg.__wbg_disableVertexAttribArray_fb822948cb54eec9=((a,b)=>{c(a).disableVertexAttribArray(b>>>W)});b.wbg.__wbg_drawElements_5cade7fb4236c93b=((a,b,d,e,f)=>{c(a).drawElements(b>>>W,d,e>>>W,f)});b.wbg.__wbg_enable_54d01bacc240df3e=((a,b)=>{c(a).enable(b>>>W)});b.wbg.__wbg_enableVertexAttribArray_c971ef03599058ec=((a,b)=>{c(a).enableVertexAttribArray(b>>>W)});b.wbg.__wbg_getAttribLocation_3ec473fee682bd2a=((a,b,d,e)=>{const f=c(a).getAttribLocation(c(b),k(d,e));return f});b.wbg.__wbg_getError_0a6390188216606e=(a=>{const b=c(a).getError();return b});b.wbg.__wbg_getExtension_5dfa3b5f570d8fe1=function(){return I(((a,b,d)=>{const e=c(a).getExtension(k(b,d));return p(e)?W:g(e)}),arguments)};b.wbg.__wbg_getParameter_798cbb8ff20c7af0=function(){return I(((a,b)=>{const d=c(a).getParameter(b>>>W);return g(d)}),arguments)};b.wbg.__wbg_getProgramInfoLog_3ff10ea818ab6ce4=((b,d,e)=>{const f=c(d).getProgramInfoLog(c(e));var g=p(f)?W:o(f,a.__wbindgen_malloc,a.__wbindgen_realloc);var h=l;r()[b/a3+ S]=h;r()[b/a3+ W]=g});b.wbg.__wbg_getProgramParameter_35800b92324ff726=((a,b,d)=>{const e=c(a).getProgramParameter(c(b),d>>>W);return g(e)});b.wbg.__wbg_getShaderInfoLog_3e435d2b50e0ecf0=((b,d,e)=>{const f=c(d).getShaderInfoLog(c(e));var g=p(f)?W:o(f,a.__wbindgen_malloc,a.__wbindgen_realloc);var h=l;r()[b/a3+ S]=h;r()[b/a3+ W]=g});b.wbg.__wbg_getShaderParameter_a9315ba73ab18731=((a,b,d)=>{const e=c(a).getShaderParameter(c(b),d>>>W);return g(e)});b.wbg.__wbg_getSupportedExtensions_eebc361c389e2ab3=(a=>{const b=c(a).getSupportedExtensions();return p(b)?W:g(b)});b.wbg.__wbg_getUniformLocation_f161344f25983444=((a,b,d,e)=>{const f=c(a).getUniformLocation(c(b),k(d,e));return p(f)?W:g(f)});b.wbg.__wbg_linkProgram_caeab1eb0c0246be=((a,b)=>{c(a).linkProgram(c(b))});b.wbg.__wbg_pixelStorei_ac98844c2d6d1937=((a,b,d)=>{c(a).pixelStorei(b>>>W,d)});b.wbg.__wbg_scissor_7206bcd2a5540aa3=((a,b,d,e,f)=>{c(a).scissor(b,d,e,f)});b.wbg.__wbg_shaderSource_04af20ecb1962b3b=((a,b,d,e)=>{c(a).shaderSource(c(b),k(d,e))});b.wbg.__wbg_texParameteri_dd08984388e62491=((a,b,d,e)=>{c(a).texParameteri(b>>>W,d>>>W,e)});b.wbg.__wbg_uniform1i_5a5f1f9d5828e6c6=((a,b,d)=>{c(a).uniform1i(c(b),d)});b.wbg.__wbg_uniform2f_d1df633e1cda7ce0=((a,b,d,e)=>{c(a).uniform2f(c(b),d,e)});b.wbg.__wbg_useProgram_229c8fa8394b4c26=((a,b)=>{c(a).useProgram(c(b))});b.wbg.__wbg_vertexAttribPointer_e9c4ff85658b9ad2=((a,b,d,e,f,g,h)=>{c(a).vertexAttribPointer(b>>>W,d,e>>>W,f!==W,g,h)});b.wbg.__wbg_viewport_0ca27d1d6ac8424c=((a,b,d,e,f)=>{c(a).viewport(b,d,e,f)});b.wbg.__wbg_items_5ca9bad002b2890c=(a=>{const b=c(a).items;return g(b)});b.wbg.__wbg_files_0aa81397021d2faa=(a=>{const b=c(a).files;return p(b)?W:g(b)});b.wbg.__wbg_preventDefault_d2c7416966cb0632=(a=>{c(a).preventDefault()});b.wbg.__wbg_stopPropagation_786ab850031995e5=(a=>{c(a).stopPropagation()});b.wbg.__wbg_clientX_4d37584813a1790a=(a=>{const b=c(a).clientX;return b});b.wbg.__wbg_clientY_ea543e0b8dc1490d=(a=>{const b=c(a).clientY;return b});b.wbg.__wbg_ctrlKey_0d75e0e9028bd999=(a=>{const b=c(a).ctrlKey;return b});b.wbg.__wbg_shiftKey_12353f0e19b21d6a=(a=>{const b=c(a).shiftKey;return b});b.wbg.__wbg_metaKey_4e3f6e986f2802b1=(a=>{const b=c(a).metaKey;return b});b.wbg.__wbg_button_8a97c55db17c7314=(a=>{const b=c(a).button;return b});b.wbg.__wbg_identifier_87f10c1b114973b1=(a=>{const b=c(a).identifier;return b});b.wbg.__wbg_pageX_6bdd2e573704efc2=(a=>{const b=c(a).pageX;return b});b.wbg.__wbg_pageY_74fbace64ec902b5=(a=>{const b=c(a).pageY;return b});b.wbg.__wbg_force_a248870a06b19f84=(a=>{const b=c(a).force;return b});b.wbg.__wbg_length_568297424aea6468=(a=>{const b=c(a).length;return b});b.wbg.__wbg_item_b77b7c1ae96bba19=((a,b)=>{const d=c(a).item(b>>>W);return p(d)?W:g(d)});b.wbg.__wbg_get_2f7d53cc08af8d1a=((a,b)=>{const d=c(a)[b>>>W];return p(d)?W:g(d)});b.wbg.__wbg_setProperty_a763529f4ef8ac76=function(){return I(((a,b,d,e,f)=>{c(a).setProperty(k(b,d),k(e,f))}),arguments)};b.wbg.__wbg_type_b820b38587c684cd=((b,d)=>{const e=c(d).type;const f=o(e,a.__wbindgen_malloc,a.__wbindgen_realloc);const g=l;r()[b/a3+ S]=g;r()[b/a3+ W]=f});b.wbg.__wbg_name_6c808ccae465f9e1=((b,d)=>{const e=c(d).name;const f=o(e,a.__wbindgen_malloc,a.__wbindgen_realloc);const g=l;r()[b/a3+ S]=g;r()[b/a3+ W]=f});b.wbg.__wbg_lastModified_5b92d1f516d58609=(a=>{const b=c(a).lastModified;return b});b.wbg.__wbg_length_5f3530f0f1af8661=(a=>{const b=c(a).length;return b});b.wbg.__wbg_get_f2ba4265e9e1e12b=((a,b)=>{const d=c(a)[b>>>W];return p(d)?W:g(d)});b.wbg.__wbg_matches_68b7ad47c1091323=(a=>{const b=c(a).matches;return b});b.wbg.__wbg_bindVertexArrayOES_e95cf32f50e47240=((a,b)=>{c(a).bindVertexArrayOES(c(b))});b.wbg.__wbg_createVertexArrayOES_96ccfea00081dcf3=(a=>{const b=c(a).createVertexArrayOES();return p(b)?W:g(b)});b.wbg.__wbg_instanceof_HtmlCanvasElement_a6076360513b6876=(a=>{let b;try{b=c(a) instanceof HTMLCanvasElement}catch(a){b=!1}const d=b;return d});b.wbg.__wbg_width_9d9d26b087c6ad54=(a=>{const b=c(a).width;return b});b.wbg.__wbg_setwidth_05075fb6b4cc720e=((a,b)=>{c(a).width=b>>>W});b.wbg.__wbg_height_770da314320603d8=(a=>{const b=c(a).height;return b});b.wbg.__wbg_setheight_7e0e88a922100d8c=((a,b)=>{c(a).height=b>>>W});b.wbg.__wbg_getContext_39cdfeffd658feb7=function(){return I(((a,b,d)=>{const e=c(a).getContext(k(b,d));return p(e)?W:g(e)}),arguments)};b.wbg.__wbg_userAgent_4106f80b9924b065=function(){return I(((b,d)=>{const e=c(d).userAgent;const f=o(e,a.__wbindgen_malloc,a.__wbindgen_realloc);const g=l;r()[b/a3+ S]=g;r()[b/a3+ W]=f}),arguments)};b.wbg.__wbg_now_65ff8ec2b863300c=(a=>{const b=c(a).now();return b});b.wbg.__wbg_getItem_f7e7a061bbdabefe=function(){return I(((b,d,e,f)=>{const g=c(d).getItem(k(e,f));var h=p(g)?W:o(g,a.__wbindgen_malloc,a.__wbindgen_realloc);var i=l;r()[b/a3+ S]=i;r()[b/a3+ W]=h}),arguments)};b.wbg.__wbg_setItem_2b72ddf192083111=function(){return I(((a,b,d,e,f)=>{c(a).setItem(k(b,d),k(e,f))}),arguments)};b.wbg.__wbg_length_c610906ecf0a8f99=(a=>{const b=c(a).length;return b});b.wbg.__wbg_get_428f35579210a950=((a,b)=>{const d=c(a)[b>>>W];return p(d)?W:g(d)});b.wbg.__wbg_top_d39cc7e325e1f687=(a=>{const b=c(a).top;return b});b.wbg.__wbg_left_064e5e69a7d7c925=(a=>{const b=c(a).left;return b});b.wbg.__wbg_dataTransfer_114daff2829a408c=(a=>{const b=c(a).dataTransfer;return p(b)?W:g(b)});b.wbg.__wbg_width_164c11c1f72aa632=(a=>{const b=c(a).width;return b});b.wbg.__wbg_height_ac60120008caa50b=(a=>{const b=c(a).height;return b});b.wbg.__wbg_instanceof_HtmlInputElement_d53941bc0aaa6ae9=(a=>{let b;try{b=c(a) instanceof HTMLInputElement}catch(a){b=!1}const d=b;return d});b.wbg.__wbg_setautofocus_4389a83ce5fce4de=((a,b)=>{c(a).autofocus=b!==W});b.wbg.__wbg_setsize_16b7c38ee657b247=((a,b)=>{c(a).size=b>>>W});b.wbg.__wbg_value_c93cb4b4d352228e=((b,d)=>{const e=c(d).value;const f=o(e,a.__wbindgen_malloc,a.__wbindgen_realloc);const g=l;r()[b/a3+ S]=g;r()[b/a3+ W]=f});b.wbg.__wbg_setvalue_9bd3f93b3864ddbf=((a,b,d)=>{c(a).value=k(b,d)});b.wbg.__wbg_addEventListener_2f891d22985fd3c8=function(){return I(((a,b,d,e)=>{c(a).addEventListener(k(b,d),c(e))}),arguments)};b.wbg.__wbg_removeEventListener_07715e6f464823fc=function(){return I(((a,b,d,e)=>{c(a).removeEventListener(k(b,d),c(e))}),arguments)};b.wbg.__wbg_matches_2a7b0d97653c323f=(a=>{const b=c(a).matches;return b});b.wbg.__wbg_parentElement_86a7612dde875ba9=(a=>{const b=c(a).parentElement;return p(b)?W:g(b)});b.wbg.__wbg_settextContent_1fec240f77aa3dc4=((a,b,d)=>{c(a).textContent=b===W?Q:k(b,d)});b.wbg.__wbg_appendChild_bd383ec5356c0bdb=function(){return I(((a,b)=>{const d=c(a).appendChild(c(b));return g(d)}),arguments)};b.wbg.__wbg_deltaX_de18e6f358ab88cf=(a=>{const b=c(a).deltaX;return b});b.wbg.__wbg_deltaY_50a026b7421f883d=(a=>{const b=c(a).deltaY;return b});b.wbg.__wbg_deltaMode_b8290e36698673d0=(a=>{const b=c(a).deltaMode;return b});b.wbg.__wbg_data_03b517344e75fca6=((b,d)=>{const e=c(d).data;var f=p(e)?W:o(e,a.__wbindgen_malloc,a.__wbindgen_realloc);var g=l;r()[b/a3+ S]=g;r()[b/a3+ W]=f});b.wbg.__wbg_keyCode_6acbcd0e4e062504=(a=>{const b=c(a).keyCode;return b});b.wbg.__wbg_altKey_c3c61dc3af936846=(a=>{const b=c(a).altKey;return b});b.wbg.__wbg_ctrlKey_e7fc1575581bc431=(a=>{const b=c(a).ctrlKey;return b});b.wbg.__wbg_shiftKey_0a061aeba25dbd63=(a=>{const b=c(a).shiftKey;return b});b.wbg.__wbg_metaKey_b879a69fa9f3f7af=(a=>{const b=c(a).metaKey;return b});b.wbg.__wbg_isComposing_aa6fdae3e5d50cdb=(a=>{const b=c(a).isComposing;return b});b.wbg.__wbg_key_9a2550983fbad1d0=((b,d)=>{const e=c(d).key;const f=o(e,a.__wbindgen_malloc,a.__wbindgen_realloc);const g=l;r()[b/a3+ S]=g;r()[b/a3+ W]=f});b.wbg.__wbg_size_be41bf26ab113208=(a=>{const b=c(a).size;return b});b.wbg.__wbg_type_b596e92b4e34956a=((b,d)=>{const e=c(d).type;const f=o(e,a.__wbindgen_malloc,a.__wbindgen_realloc);const g=l;r()[b/a3+ S]=g;r()[b/a3+ W]=f});b.wbg.__wbg_arrayBuffer_fb7b7f60c42268ca=(a=>{const b=c(a).arrayBuffer();return g(b)});b.wbg.__wbg_href_1ab7f03b8a745310=function(){return I(((b,d)=>{const e=c(d).href;const f=o(e,a.__wbindgen_malloc,a.__wbindgen_realloc);const g=l;r()[b/a3+ S]=g;r()[b/a3+ W]=f}),arguments)};b.wbg.__wbg_origin_a66ff95a994d7e40=function(){return I(((b,d)=>{const e=c(d).origin;const f=o(e,a.__wbindgen_malloc,a.__wbindgen_realloc);const g=l;r()[b/a3+ S]=g;r()[b/a3+ W]=f}),arguments)};b.wbg.__wbg_protocol_14f54c8356e78bea=function(){return I(((b,d)=>{const e=c(d).protocol;const f=o(e,a.__wbindgen_malloc,a.__wbindgen_realloc);const g=l;r()[b/a3+ S]=g;r()[b/a3+ W]=f}),arguments)};b.wbg.__wbg_host_0c29a6ff8ae1ff8c=function(){return I(((b,d)=>{const e=c(d).host;const f=o(e,a.__wbindgen_malloc,a.__wbindgen_realloc);const g=l;r()[b/a3+ S]=g;r()[b/a3+ W]=f}),arguments)};b.wbg.__wbg_hostname_26a3a1944f8c045c=function(){return I(((b,d)=>{const e=c(d).hostname;const f=o(e,a.__wbindgen_malloc,a.__wbindgen_realloc);const g=l;r()[b/a3+ S]=g;r()[b/a3+ W]=f}),arguments)};b.wbg.__wbg_port_a56212936bd85dac=function(){return I(((b,d)=>{const e=c(d).port;const f=o(e,a.__wbindgen_malloc,a.__wbindgen_realloc);const g=l;r()[b/a3+ S]=g;r()[b/a3+ W]=f}),arguments)};b.wbg.__wbg_search_eb68df82d26f8761=function(){return I(((b,d)=>{const e=c(d).search;const f=o(e,a.__wbindgen_malloc,a.__wbindgen_realloc);const g=l;r()[b/a3+ S]=g;r()[b/a3+ W]=f}),arguments)};b.wbg.__wbg_hash_9bd16c0f666cdf27=function(){return I(((b,d)=>{const e=c(d).hash;const f=o(e,a.__wbindgen_malloc,a.__wbindgen_realloc);const g=l;r()[b/a3+ S]=g;r()[b/a3+ W]=f}),arguments)};b.wbg.__wbg_touches_95bba57784560e75=(a=>{const b=c(a).touches;return g(b)});b.wbg.__wbg_changedTouches_9667f17739458e92=(a=>{const b=c(a).changedTouches;return g(b)});b.wbg.__wbg_get_c43534c00f382c8a=((a,b)=>{const d=c(a)[b>>>W];return g(d)});b.wbg.__wbg_length_d99b680fd68bf71b=(a=>{const b=c(a).length;return b});b.wbg.__wbg_newnoargs_5859b6d41c6fe9f7=((a,b)=>{const c=new Function(k(a,b));return g(c)});b.wbg.__wbg_get_5027b32da70f39b1=function(){return I(((a,b)=>{const d=Reflect.get(c(a),c(b));return g(d)}),arguments)};b.wbg.__wbg_call_a79f1973a4f07d5e=function(){return I(((a,b)=>{const d=c(a).call(c(b));return g(d)}),arguments)};b.wbg.__wbg_self_086b5302bcafb962=function(){return I((()=>{const a=self.self;return g(a)}),arguments)};b.wbg.__wbg_window_132fa5d7546f1de5=function(){return I((()=>{const a=window.window;return g(a)}),arguments)};b.wbg.__wbg_globalThis_e5f801a37ad7d07b=function(){return I((()=>{const a=globalThis.globalThis;return g(a)}),arguments)};b.wbg.__wbg_global_f9a61fce4af6b7c1=function(){return I((()=>{const a=global.global;return g(a)}),arguments)};b.wbg.__wbindgen_is_undefined=(a=>{const b=c(a)===Q;return b});b.wbg.__wbg_call_f6a2bc58c19c53c6=function(){return I(((a,b,d)=>{const e=c(a).call(c(b),c(d));return g(e)}),arguments)};b.wbg.__wbg_resolve_97ecd55ee839391b=(a=>{const b=Promise.resolve(c(a));return g(b)});b.wbg.__wbg_then_7aeb7c5f1536640f=((a,b)=>{const d=c(a).then(c(b));return g(d)});b.wbg.__wbg_then_5842e4e97f7beace=((a,b,d)=>{const e=c(a).then(c(b),c(d));return g(e)});b.wbg.__wbg_buffer_5d1b598a01b41a42=(a=>{const b=c(a).buffer;return g(b)});b.wbg.__wbg_newwithbyteoffsetandlength_54c7b98977affdec=((a,b,d)=>{const e=new Int8Array(c(a),b>>>W,d>>>W);return g(e)});b.wbg.__wbg_newwithbyteoffsetandlength_16ba6d10861ea013=((a,b,d)=>{const e=new Int16Array(c(a),b>>>W,d>>>W);return g(e)});b.wbg.__wbg_newwithbyteoffsetandlength_821c7736f0d22b04=((a,b,d)=>{const e=new Z(c(a),b>>>W,d>>>W);return g(e)});b.wbg.__wbg_newwithbyteoffsetandlength_d695c7957788f922=((a,b,d)=>{const e=new X(c(a),b>>>W,d>>>W);return g(e)});b.wbg.__wbg_new_ace717933ad7117f=(a=>{const b=new X(c(a));return g(b)});b.wbg.__wbg_set_74906aa30864df5a=((a,b,d)=>{c(a).set(c(b),d>>>W)});b.wbg.__wbg_length_f0764416ba5bb237=(a=>{const b=c(a).length;return b});b.wbg.__wbg_newwithbyteoffsetandlength_2412e38a0385bbe2=((a,b,d)=>{const e=new Uint16Array(c(a),b>>>W,d>>>W);return g(e)});b.wbg.__wbg_newwithbyteoffsetandlength_aeed38cac7555df7=((a,b,d)=>{const e=new Uint32Array(c(a),b>>>W,d>>>W);return g(e)});b.wbg.__wbg_newwithbyteoffsetandlength_21163b4dfcbc673c=((a,b,d)=>{const e=new Float32Array(c(a),b>>>W,d>>>W);return g(e)});b.wbg.__wbg_newwithlength_728575f3bba9959b=(a=>{const b=new X(a>>>W);return g(b)});b.wbg.__wbg_subarray_7f7a652672800851=((a,b,d)=>{const e=c(a).subarray(b>>>W,d>>>W);return g(e)});b.wbg.__wbindgen_debug_string=((b,d)=>{const e=u(c(d));const f=o(e,a.__wbindgen_malloc,a.__wbindgen_realloc);const g=l;r()[b/a3+ S]=g;r()[b/a3+ W]=f});b.wbg.__wbindgen_throw=((a,b)=>{throw new V(k(a,b))});b.wbg.__wbindgen_memory=(()=>{const b=a.memory;return g(b)});b.wbg.__wbindgen_closure_wrapper1611=((a,b,c)=>{const d=v(a,b,a4,w);return g(d)});b.wbg.__wbindgen_closure_wrapper1613=((a,b,c)=>{const d=x(a,b,a4,y);return g(d)});b.wbg.__wbindgen_closure_wrapper1615=((a,b,c)=>{const d=x(a,b,a4,A);return g(d)});b.wbg.__wbindgen_closure_wrapper1617=((a,b,c)=>{const d=v(a,b,a4,B);return g(d)});b.wbg.__wbindgen_closure_wrapper4019=((a,b,c)=>{const d=v(a,b,a5,C);return g(d)});b.wbg.__wbindgen_closure_wrapper4021=((a,b,c)=>{const d=v(a,b,a5,D);return g(d)});b.wbg.__wbindgen_closure_wrapper4023=((a,b,c)=>{const d=v(a,b,a5,E);return g(d)});b.wbg.__wbindgen_closure_wrapper4216=((a,b,c)=>{const d=v(a,b,1279,F);return g(d)});return b});var t=(()=>{if(s===R||s.byteLength===W){s=new Float64Array(a.memory.buffer)};return s});var G=((a,b)=>{a=a>>>W;return j().subarray(a/S,a/S+ b)});var o=((a,b,c)=>{if(c===Q){const c=m.encode(a);const d=b(c.length,S)>>>W;j().subarray(d,d+ c.length).set(c);l=c.length;return d};let d=a.length;let e=b(d,S)>>>W;const f=j();let g=W;for(;g<d;g++){const b=a.charCodeAt(g);if(b>127)break;f[e+ g]=b};if(g!==d){if(g!==W){a=a.slice(g)};e=c(e,d,d=g+ a.length*3,S)>>>W;const b=j().subarray(e+ g,e+ d);const f=n(a,b);g+=f.written};l=g;return e});var j=(()=>{if(i===R||i.byteLength===W){i=new X(a.memory.buffer)};return i});var F=((b,c,d)=>{a._dyn_core__ops__function__FnMut__A____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__hd93f4746c8dbe6c8(b,c,g(d))});var J=(async(a,b)=>{if(typeof Response===Y&&a instanceof Response){if(typeof WebAssembly.instantiateStreaming===Y){try{return await WebAssembly.instantiateStreaming(a,b)}catch(b){if(a.headers.get(`Content-Type`)!=`application/wasm`){console.warn(`\`WebAssembly.instantiateStreaming\` failed because your server does not serve wasm with \`application/wasm\` MIME type. Falling back to \`WebAssembly.instantiate\` which is slower. Original error:\\n`,b)}else{throw b}}};const c=await a.arrayBuffer();return await WebAssembly.instantiate(c,b)}else{const c=await WebAssembly.instantiate(a,b);if(c instanceof WebAssembly.Instance){return {instance:c,module:a}}else{return c}}});var w=((b,c,d,e)=>{const f=o(d,a.__wbindgen_malloc,a.__wbindgen_realloc);const g=l;const h=o(e,a.__wbindgen_malloc,a.__wbindgen_realloc);const i=l;a.wasm_bindgen__convert__closures__invoke2_mut__h429375f2b7e74153(b,c,f,g,h,i)});var g=(a=>{if(d===b.length)b.push(b.length+ S);const c=d;d=b[c];b[c]=a;return c});var f=(a=>{const b=c(a);e(a);return b});var r=(()=>{if(q===R||q.byteLength===W){q=new Z(a.memory.buffer)};return q});var O=(async(b)=>{if(a!==Q)return a;if(typeof b===T){b=new URL(`four-bar-ui_bg.wasm`,import.meta.url)};const c=K();if(typeof b===a0||typeof Request===Y&&b instanceof Request||typeof URL===Y&&b instanceof URL){b=fetch(b)};L(c);const {instance:d,module:e}=await J(await b,c);return M(d,e)});var N=(b=>{if(a!==Q)return a;const c=K();L(c);if(!(b instanceof WebAssembly.Module)){b=new WebAssembly.Module(b)};const d=new WebAssembly.Instance(b,c);return M(d,b)});var x=((b,c,d,e)=>{const f={a:b,b:c,cnt:S,dtor:d};const g=(...b)=>{f.cnt++;try{return e(f.a,f.b,...b)}finally{if(--f.cnt===W){a.__wbindgen_export_2.get(f.dtor)(f.a,f.b);f.a=W}}};g.original=f;return g});var e=(a=>{if(a<132)return;b[a]=d;d=a});var E=((b,c,d)=>{a.wasm_bindgen__convert__closures__invoke1_mut__hbd88fb4f40dec91d(b,c,g(d))});var D=((b,c)=>{a.wasm_bindgen__convert__closures__invoke0_mut__hece5f828b38cb117(b,c)});var v=((b,c,d,e)=>{const f={a:b,b:c,cnt:S,dtor:d};const g=(...b)=>{f.cnt++;const c=f.a;f.a=W;try{return e(c,f.b,...b)}finally{if(--f.cnt===W){a.__wbindgen_export_2.get(f.dtor)(c,f.b)}else{f.a=c}}};g.original=f;return g});var k=((a,b)=>{a=a>>>W;return h.decode(j().subarray(a,a+ b))});var M=((b,c)=>{a=b.exports;O.__wbindgen_wasm_module=c;s=R;q=R;i=R;a.__wbindgen_start();return a});let a;const b=new P(128).fill(Q);b.push(Q,R,!0,!1);let d=b.length;const h=typeof TextDecoder!==T?new TextDecoder(U,{ignoreBOM:!0,fatal:!0}):{decode:()=>{throw V(`TextDecoder not available`)}};if(typeof TextDecoder!==T){h.decode()};let i=R;let l=W;const m=typeof TextEncoder!==T?new TextEncoder(U):{encode:()=>{throw V(`TextEncoder not available`)}};const n=typeof m.encodeInto===Y?((a,b)=>m.encodeInto(a,b)):((a,b)=>{const c=m.encode(a);b.set(c);return {read:a.length,written:c.length}});let q=R;let s=R;export default O;export{N as initSync}