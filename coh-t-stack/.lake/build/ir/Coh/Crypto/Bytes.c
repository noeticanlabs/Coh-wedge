// Lean compiler output
// Module: Coh.Crypto.Bytes
// Imports: Init Coh.Prelude
#include <lean/lean.h>
#if defined(__clang__)
#pragma clang diagnostic ignored "-Wunused-parameter"
#pragma clang diagnostic ignored "-Wunused-label"
#elif defined(__GNUC__) && !defined(__CLANG__)
#pragma GCC diagnostic ignored "-Wunused-parameter"
#pragma GCC diagnostic ignored "-Wunused-label"
#pragma GCC diagnostic ignored "-Wunused-but-set-variable"
#endif
#ifdef __cplusplus
extern "C" {
#endif
LEAN_EXPORT lean_object* l_Coh_Crypto_utf8Encode___boxed(lean_object*);
static lean_object* l_Coh_Crypto_pipeDelimiter___closed__1;
LEAN_EXPORT lean_object* l_Coh_Crypto_bytesConcat___boxed(lean_object*, lean_object*);
LEAN_EXPORT lean_object* l_Coh_Crypto_utf8Encode(lean_object*);
LEAN_EXPORT lean_object* l_Coh_Crypto_bytesConcat(lean_object*, lean_object*);
LEAN_EXPORT lean_object* l_Coh_Crypto_pipeDelimiter;
lean_object* lean_string_append(lean_object*, lean_object*);
LEAN_EXPORT lean_object* l_Coh_Crypto_utf8Encode(lean_object* x_1) {
_start:
{
lean_inc(x_1);
return x_1;
}
}
LEAN_EXPORT lean_object* l_Coh_Crypto_utf8Encode___boxed(lean_object* x_1) {
_start:
{
lean_object* x_2; 
x_2 = l_Coh_Crypto_utf8Encode(x_1);
lean_dec(x_1);
return x_2;
}
}
LEAN_EXPORT lean_object* l_Coh_Crypto_bytesConcat(lean_object* x_1, lean_object* x_2) {
_start:
{
lean_object* x_3; 
x_3 = lean_string_append(x_1, x_2);
return x_3;
}
}
LEAN_EXPORT lean_object* l_Coh_Crypto_bytesConcat___boxed(lean_object* x_1, lean_object* x_2) {
_start:
{
lean_object* x_3; 
x_3 = l_Coh_Crypto_bytesConcat(x_1, x_2);
lean_dec(x_2);
return x_3;
}
}
static lean_object* _init_l_Coh_Crypto_pipeDelimiter___closed__1() {
_start:
{
lean_object* x_1; 
x_1 = lean_mk_string_unchecked("|", 1, 1);
return x_1;
}
}
static lean_object* _init_l_Coh_Crypto_pipeDelimiter() {
_start:
{
lean_object* x_1; 
x_1 = l_Coh_Crypto_pipeDelimiter___closed__1;
return x_1;
}
}
lean_object* initialize_Init(uint8_t builtin, lean_object*);
lean_object* initialize_Coh_Prelude(uint8_t builtin, lean_object*);
static bool _G_initialized = false;
LEAN_EXPORT lean_object* initialize_Coh_Crypto_Bytes(uint8_t builtin, lean_object* w) {
lean_object * res;
if (_G_initialized) return lean_io_result_mk_ok(lean_box(0));
_G_initialized = true;
res = initialize_Init(builtin, lean_io_mk_world());
if (lean_io_result_is_error(res)) return res;
lean_dec_ref(res);
res = initialize_Coh_Prelude(builtin, lean_io_mk_world());
if (lean_io_result_is_error(res)) return res;
lean_dec_ref(res);
l_Coh_Crypto_pipeDelimiter___closed__1 = _init_l_Coh_Crypto_pipeDelimiter___closed__1();
lean_mark_persistent(l_Coh_Crypto_pipeDelimiter___closed__1);
l_Coh_Crypto_pipeDelimiter = _init_l_Coh_Crypto_pipeDelimiter();
lean_mark_persistent(l_Coh_Crypto_pipeDelimiter);
return lean_io_result_mk_ok(lean_box(0));
}
#ifdef __cplusplus
}
#endif
