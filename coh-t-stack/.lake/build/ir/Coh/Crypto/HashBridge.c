// Lean compiler output
// Module: Coh.Crypto.HashBridge
// Imports: Init Coh.Crypto.SHA256Spec Coh.Crypto.JCS
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
LEAN_EXPORT lean_object* l_Coh_Crypto_rustChainDigestInputBytes(lean_object*, lean_object*);
static lean_object* l_Coh_Crypto_rustChainDigestInputBytes___closed__2;
LEAN_EXPORT lean_object* l_Coh_Crypto_canonicalDigestInputBytes(lean_object*);
LEAN_EXPORT lean_object* l_Coh_Crypto_rustChainDigestInputBytes___boxed(lean_object*, lean_object*);
uint8_t lean_string_dec_eq(lean_object*, lean_object*);
static lean_object* l_Coh_Crypto_rustChainDigestInputBytes___closed__3;
static lean_object* l_Coh_Crypto_rustChainDigestInputBytes___closed__4;
LEAN_EXPORT lean_object* l_Coh_Crypto_instDecidablePayloadMatchesCanonicalJson___boxed(lean_object*);
LEAN_EXPORT uint8_t l_Coh_Crypto_instDecidablePayloadMatchesCanonicalJson(lean_object*);
lean_object* l_Coh_Crypto_canonicalMicroJson(lean_object*);
static lean_object* l_Coh_Crypto_rustChainDigestInputBytes___closed__1;
LEAN_EXPORT lean_object* l_Coh_Crypto_canonicalDigestInputBytes___boxed(lean_object*);
extern lean_object* l_Coh_Core_digestDomainTag;
lean_object* lean_string_append(lean_object*, lean_object*);
static lean_object* _init_l_Coh_Crypto_rustChainDigestInputBytes___closed__1() {
_start:
{
lean_object* x_1; 
x_1 = lean_mk_string_unchecked("", 0, 0);
return x_1;
}
}
static lean_object* _init_l_Coh_Crypto_rustChainDigestInputBytes___closed__2() {
_start:
{
lean_object* x_1; lean_object* x_2; lean_object* x_3; 
x_1 = l_Coh_Crypto_rustChainDigestInputBytes___closed__1;
x_2 = l_Coh_Core_digestDomainTag;
x_3 = lean_string_append(x_1, x_2);
return x_3;
}
}
static lean_object* _init_l_Coh_Crypto_rustChainDigestInputBytes___closed__3() {
_start:
{
lean_object* x_1; 
x_1 = lean_mk_string_unchecked("|", 1, 1);
return x_1;
}
}
static lean_object* _init_l_Coh_Crypto_rustChainDigestInputBytes___closed__4() {
_start:
{
lean_object* x_1; lean_object* x_2; lean_object* x_3; 
x_1 = l_Coh_Crypto_rustChainDigestInputBytes___closed__2;
x_2 = l_Coh_Crypto_rustChainDigestInputBytes___closed__3;
x_3 = lean_string_append(x_1, x_2);
return x_3;
}
}
LEAN_EXPORT lean_object* l_Coh_Crypto_rustChainDigestInputBytes(lean_object* x_1, lean_object* x_2) {
_start:
{
lean_object* x_3; lean_object* x_4; lean_object* x_5; lean_object* x_6; lean_object* x_7; lean_object* x_8; lean_object* x_9; 
x_3 = l_Coh_Crypto_rustChainDigestInputBytes___closed__4;
x_4 = lean_string_append(x_3, x_1);
x_5 = l_Coh_Crypto_rustChainDigestInputBytes___closed__3;
x_6 = lean_string_append(x_4, x_5);
x_7 = lean_string_append(x_6, x_2);
x_8 = l_Coh_Crypto_rustChainDigestInputBytes___closed__1;
x_9 = lean_string_append(x_7, x_8);
return x_9;
}
}
LEAN_EXPORT lean_object* l_Coh_Crypto_rustChainDigestInputBytes___boxed(lean_object* x_1, lean_object* x_2) {
_start:
{
lean_object* x_3; 
x_3 = l_Coh_Crypto_rustChainDigestInputBytes(x_1, x_2);
lean_dec(x_2);
lean_dec(x_1);
return x_3;
}
}
LEAN_EXPORT lean_object* l_Coh_Crypto_canonicalDigestInputBytes(lean_object* x_1) {
_start:
{
lean_object* x_2; lean_object* x_3; lean_object* x_4; 
x_2 = lean_ctor_get(x_1, 8);
x_3 = l_Coh_Crypto_canonicalMicroJson(x_1);
x_4 = l_Coh_Crypto_rustChainDigestInputBytes(x_2, x_3);
lean_dec(x_3);
return x_4;
}
}
LEAN_EXPORT lean_object* l_Coh_Crypto_canonicalDigestInputBytes___boxed(lean_object* x_1) {
_start:
{
lean_object* x_2; 
x_2 = l_Coh_Crypto_canonicalDigestInputBytes(x_1);
lean_dec(x_1);
return x_2;
}
}
LEAN_EXPORT uint8_t l_Coh_Crypto_instDecidablePayloadMatchesCanonicalJson(lean_object* x_1) {
_start:
{
lean_object* x_2; lean_object* x_3; uint8_t x_4; 
x_2 = lean_ctor_get(x_1, 10);
x_3 = l_Coh_Crypto_canonicalMicroJson(x_1);
x_4 = lean_string_dec_eq(x_2, x_3);
lean_dec(x_3);
return x_4;
}
}
LEAN_EXPORT lean_object* l_Coh_Crypto_instDecidablePayloadMatchesCanonicalJson___boxed(lean_object* x_1) {
_start:
{
uint8_t x_2; lean_object* x_3; 
x_2 = l_Coh_Crypto_instDecidablePayloadMatchesCanonicalJson(x_1);
lean_dec(x_1);
x_3 = lean_box(x_2);
return x_3;
}
}
lean_object* initialize_Init(uint8_t builtin, lean_object*);
lean_object* initialize_Coh_Crypto_SHA256Spec(uint8_t builtin, lean_object*);
lean_object* initialize_Coh_Crypto_JCS(uint8_t builtin, lean_object*);
static bool _G_initialized = false;
LEAN_EXPORT lean_object* initialize_Coh_Crypto_HashBridge(uint8_t builtin, lean_object* w) {
lean_object * res;
if (_G_initialized) return lean_io_result_mk_ok(lean_box(0));
_G_initialized = true;
res = initialize_Init(builtin, lean_io_mk_world());
if (lean_io_result_is_error(res)) return res;
lean_dec_ref(res);
res = initialize_Coh_Crypto_SHA256Spec(builtin, lean_io_mk_world());
if (lean_io_result_is_error(res)) return res;
lean_dec_ref(res);
res = initialize_Coh_Crypto_JCS(builtin, lean_io_mk_world());
if (lean_io_result_is_error(res)) return res;
lean_dec_ref(res);
l_Coh_Crypto_rustChainDigestInputBytes___closed__1 = _init_l_Coh_Crypto_rustChainDigestInputBytes___closed__1();
lean_mark_persistent(l_Coh_Crypto_rustChainDigestInputBytes___closed__1);
l_Coh_Crypto_rustChainDigestInputBytes___closed__2 = _init_l_Coh_Crypto_rustChainDigestInputBytes___closed__2();
lean_mark_persistent(l_Coh_Crypto_rustChainDigestInputBytes___closed__2);
l_Coh_Crypto_rustChainDigestInputBytes___closed__3 = _init_l_Coh_Crypto_rustChainDigestInputBytes___closed__3();
lean_mark_persistent(l_Coh_Crypto_rustChainDigestInputBytes___closed__3);
l_Coh_Crypto_rustChainDigestInputBytes___closed__4 = _init_l_Coh_Crypto_rustChainDigestInputBytes___closed__4();
lean_mark_persistent(l_Coh_Crypto_rustChainDigestInputBytes___closed__4);
return lean_io_result_mk_ok(lean_box(0));
}
#ifdef __cplusplus
}
#endif
