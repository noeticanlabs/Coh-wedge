// Lean compiler output
// Module: Coh.Contract.Canon
// Imports: Init Coh.Crypto.HashBridge
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
lean_object* l_Coh_Crypto_rustChainDigestInputBytes(lean_object*, lean_object*);
lean_object* l_Coh_Crypto_canonicalDigestInputBytes(lean_object*);
LEAN_EXPORT lean_object* l_Coh_Contract_receiptProjectionOf___boxed(lean_object*);
LEAN_EXPORT lean_object* l_Coh_Contract_rustChainDigestInputBytes___boxed(lean_object*, lean_object*);
LEAN_EXPORT lean_object* l_Coh_Contract_jcsQuote(lean_object*);
LEAN_EXPORT lean_object* l_Coh_Contract_jcsQuote___boxed(lean_object*);
LEAN_EXPORT lean_object* l_Coh_Contract_canonicalDigestInputBytes___boxed(lean_object*);
LEAN_EXPORT lean_object* l_Coh_Contract_canonicalMicroJson___boxed(lean_object*);
LEAN_EXPORT lean_object* l_Coh_Contract_rustChainDigestInputBytes(lean_object*, lean_object*);
LEAN_EXPORT lean_object* l_Coh_Contract_receiptProjectionCanonicalJson(lean_object*);
LEAN_EXPORT lean_object* l_Coh_Contract_canonicalMetricsJson(lean_object*);
lean_object* l_Coh_Crypto_canonicalMicroJson(lean_object*);
lean_object* l_Coh_Crypto_receiptProjectionCanonicalJson(lean_object*);
LEAN_EXPORT lean_object* l_Coh_Contract_receiptProjectionOf(lean_object*);
lean_object* l_Coh_Crypto_receiptProjectionOf(lean_object*);
LEAN_EXPORT lean_object* l_Coh_Contract_canonicalMicroJson(lean_object*);
LEAN_EXPORT lean_object* l_Coh_Contract_canonicalDigestInputBytes(lean_object*);
lean_object* l_Coh_Crypto_canonicalMetricsJson(lean_object*);
lean_object* l_Coh_Crypto_jcsQuote(lean_object*);
LEAN_EXPORT lean_object* l_Coh_Contract_jcsQuote(lean_object* x_1) {
_start:
{
lean_object* x_2; 
x_2 = l_Coh_Crypto_jcsQuote(x_1);
return x_2;
}
}
LEAN_EXPORT lean_object* l_Coh_Contract_jcsQuote___boxed(lean_object* x_1) {
_start:
{
lean_object* x_2; 
x_2 = l_Coh_Contract_jcsQuote(x_1);
lean_dec(x_1);
return x_2;
}
}
LEAN_EXPORT lean_object* l_Coh_Contract_receiptProjectionOf(lean_object* x_1) {
_start:
{
lean_object* x_2; 
x_2 = l_Coh_Crypto_receiptProjectionOf(x_1);
return x_2;
}
}
LEAN_EXPORT lean_object* l_Coh_Contract_receiptProjectionOf___boxed(lean_object* x_1) {
_start:
{
lean_object* x_2; 
x_2 = l_Coh_Contract_receiptProjectionOf(x_1);
lean_dec(x_1);
return x_2;
}
}
LEAN_EXPORT lean_object* l_Coh_Contract_canonicalMetricsJson(lean_object* x_1) {
_start:
{
lean_object* x_2; 
x_2 = l_Coh_Crypto_canonicalMetricsJson(x_1);
return x_2;
}
}
LEAN_EXPORT lean_object* l_Coh_Contract_receiptProjectionCanonicalJson(lean_object* x_1) {
_start:
{
lean_object* x_2; 
x_2 = l_Coh_Crypto_receiptProjectionCanonicalJson(x_1);
return x_2;
}
}
LEAN_EXPORT lean_object* l_Coh_Contract_canonicalMicroJson(lean_object* x_1) {
_start:
{
lean_object* x_2; 
x_2 = l_Coh_Crypto_canonicalMicroJson(x_1);
return x_2;
}
}
LEAN_EXPORT lean_object* l_Coh_Contract_canonicalMicroJson___boxed(lean_object* x_1) {
_start:
{
lean_object* x_2; 
x_2 = l_Coh_Contract_canonicalMicroJson(x_1);
lean_dec(x_1);
return x_2;
}
}
LEAN_EXPORT lean_object* l_Coh_Contract_rustChainDigestInputBytes(lean_object* x_1, lean_object* x_2) {
_start:
{
lean_object* x_3; 
x_3 = l_Coh_Crypto_rustChainDigestInputBytes(x_1, x_2);
return x_3;
}
}
LEAN_EXPORT lean_object* l_Coh_Contract_rustChainDigestInputBytes___boxed(lean_object* x_1, lean_object* x_2) {
_start:
{
lean_object* x_3; 
x_3 = l_Coh_Contract_rustChainDigestInputBytes(x_1, x_2);
lean_dec(x_2);
lean_dec(x_1);
return x_3;
}
}
LEAN_EXPORT lean_object* l_Coh_Contract_canonicalDigestInputBytes(lean_object* x_1) {
_start:
{
lean_object* x_2; 
x_2 = l_Coh_Crypto_canonicalDigestInputBytes(x_1);
return x_2;
}
}
LEAN_EXPORT lean_object* l_Coh_Contract_canonicalDigestInputBytes___boxed(lean_object* x_1) {
_start:
{
lean_object* x_2; 
x_2 = l_Coh_Contract_canonicalDigestInputBytes(x_1);
lean_dec(x_1);
return x_2;
}
}
lean_object* initialize_Init(uint8_t builtin, lean_object*);
lean_object* initialize_Coh_Crypto_HashBridge(uint8_t builtin, lean_object*);
static bool _G_initialized = false;
LEAN_EXPORT lean_object* initialize_Coh_Contract_Canon(uint8_t builtin, lean_object* w) {
lean_object * res;
if (_G_initialized) return lean_io_result_mk_ok(lean_box(0));
_G_initialized = true;
res = initialize_Init(builtin, lean_io_mk_world());
if (lean_io_result_is_error(res)) return res;
lean_dec_ref(res);
res = initialize_Coh_Crypto_HashBridge(builtin, lean_io_mk_world());
if (lean_io_result_is_error(res)) return res;
lean_dec_ref(res);
return lean_io_result_mk_ok(lean_box(0));
}
#ifdef __cplusplus
}
#endif
