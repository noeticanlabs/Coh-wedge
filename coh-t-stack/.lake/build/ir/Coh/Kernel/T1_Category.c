// Lean compiler output
// Module: Coh.Kernel.T1_Category
// Imports: Init Coh.Kernel.Receipt Coh.Kernel.Verifier Mathlib.Tactic.Linarith
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
extern lean_object* l___private_Mathlib_Data_Real_Basic_0__Real_zero;
LEAN_EXPORT lean_object* l_Coh_Kernel_T1__StrictCoh__to__Category___rarg(lean_object*);
LEAN_EXPORT lean_object* l_Coh_Kernel_T1__StrictCoh__to__Category___elambda__1(lean_object*);
LEAN_EXPORT lean_object* l_Coh_Kernel_T1__StrictCoh__to__Category___elambda__2(lean_object*);
static lean_object* l_Coh_Kernel_transition__id___closed__1;
LEAN_EXPORT lean_object* l_Coh_Kernel_transition__comp(lean_object*, lean_object*);
LEAN_EXPORT lean_object* l_Coh_Kernel_transition__id(lean_object*, lean_object*);
LEAN_EXPORT lean_object* l_Coh_Kernel_T1__StrictCoh__to__Category___elambda__2___rarg(lean_object*, lean_object*);
lean_object* l_Real_definition____x40_Mathlib_Data_Real_Basic___hyg_657_(lean_object*, lean_object*);
LEAN_EXPORT lean_object* l_Coh_Kernel_transition__id___boxed(lean_object*, lean_object*);
LEAN_EXPORT lean_object* l_Coh_Kernel_T1__StrictCoh__to__Category(lean_object*);
LEAN_EXPORT lean_object* l_Coh_Kernel_T1__StrictCoh__to__Category___elambda__1___rarg(lean_object*, lean_object*, lean_object*, lean_object*, lean_object*, lean_object*);
static lean_object* _init_l_Coh_Kernel_transition__id___closed__1() {
_start:
{
lean_object* x_1; lean_object* x_2; 
x_1 = l___private_Mathlib_Data_Real_Basic_0__Real_zero;
x_2 = lean_alloc_ctor(0, 5, 0);
lean_ctor_set(x_2, 0, x_1);
lean_ctor_set(x_2, 1, x_1);
lean_ctor_set(x_2, 2, x_1);
lean_ctor_set(x_2, 3, x_1);
lean_ctor_set(x_2, 4, x_1);
return x_2;
}
}
LEAN_EXPORT lean_object* l_Coh_Kernel_transition__id(lean_object* x_1, lean_object* x_2) {
_start:
{
lean_object* x_3; 
x_3 = l_Coh_Kernel_transition__id___closed__1;
return x_3;
}
}
LEAN_EXPORT lean_object* l_Coh_Kernel_transition__id___boxed(lean_object* x_1, lean_object* x_2) {
_start:
{
lean_object* x_3; 
x_3 = l_Coh_Kernel_transition__id(x_1, x_2);
lean_dec(x_2);
return x_3;
}
}
LEAN_EXPORT lean_object* l_Coh_Kernel_transition__comp(lean_object* x_1, lean_object* x_2) {
_start:
{
lean_object* x_3; lean_object* x_4; lean_object* x_5; lean_object* x_6; uint8_t x_7; 
x_3 = lean_ctor_get(x_2, 0);
lean_inc(x_3);
x_4 = lean_ctor_get(x_2, 2);
lean_inc(x_4);
x_5 = lean_ctor_get(x_2, 3);
lean_inc(x_5);
x_6 = lean_ctor_get(x_2, 4);
lean_inc(x_6);
lean_dec(x_2);
x_7 = !lean_is_exclusive(x_1);
if (x_7 == 0)
{
lean_object* x_8; lean_object* x_9; lean_object* x_10; lean_object* x_11; lean_object* x_12; lean_object* x_13; lean_object* x_14; 
x_8 = lean_ctor_get(x_1, 2);
x_9 = lean_ctor_get(x_1, 3);
x_10 = lean_ctor_get(x_1, 4);
x_11 = lean_ctor_get(x_1, 0);
lean_dec(x_11);
x_12 = l_Real_definition____x40_Mathlib_Data_Real_Basic___hyg_657_(x_4, x_8);
x_13 = l_Real_definition____x40_Mathlib_Data_Real_Basic___hyg_657_(x_5, x_9);
x_14 = l_Real_definition____x40_Mathlib_Data_Real_Basic___hyg_657_(x_6, x_10);
lean_ctor_set(x_1, 4, x_14);
lean_ctor_set(x_1, 3, x_13);
lean_ctor_set(x_1, 2, x_12);
lean_ctor_set(x_1, 0, x_3);
return x_1;
}
else
{
lean_object* x_15; lean_object* x_16; lean_object* x_17; lean_object* x_18; lean_object* x_19; lean_object* x_20; lean_object* x_21; lean_object* x_22; 
x_15 = lean_ctor_get(x_1, 1);
x_16 = lean_ctor_get(x_1, 2);
x_17 = lean_ctor_get(x_1, 3);
x_18 = lean_ctor_get(x_1, 4);
lean_inc(x_18);
lean_inc(x_17);
lean_inc(x_16);
lean_inc(x_15);
lean_dec(x_1);
x_19 = l_Real_definition____x40_Mathlib_Data_Real_Basic___hyg_657_(x_4, x_16);
x_20 = l_Real_definition____x40_Mathlib_Data_Real_Basic___hyg_657_(x_5, x_17);
x_21 = l_Real_definition____x40_Mathlib_Data_Real_Basic___hyg_657_(x_6, x_18);
x_22 = lean_alloc_ctor(0, 5, 0);
lean_ctor_set(x_22, 0, x_3);
lean_ctor_set(x_22, 1, x_15);
lean_ctor_set(x_22, 2, x_19);
lean_ctor_set(x_22, 3, x_20);
lean_ctor_set(x_22, 4, x_21);
return x_22;
}
}
}
LEAN_EXPORT lean_object* l_Coh_Kernel_T1__StrictCoh__to__Category___elambda__1___rarg(lean_object* x_1, lean_object* x_2, lean_object* x_3, lean_object* x_4, lean_object* x_5, lean_object* x_6) {
_start:
{
lean_object* x_7; lean_object* x_8; 
x_7 = lean_ctor_get(x_1, 2);
lean_inc(x_7);
lean_dec(x_1);
x_8 = lean_apply_5(x_7, x_2, x_3, x_4, x_5, x_6);
return x_8;
}
}
LEAN_EXPORT lean_object* l_Coh_Kernel_T1__StrictCoh__to__Category___elambda__1(lean_object* x_1) {
_start:
{
lean_object* x_2; 
x_2 = lean_alloc_closure((void*)(l_Coh_Kernel_T1__StrictCoh__to__Category___elambda__1___rarg), 6, 0);
return x_2;
}
}
LEAN_EXPORT lean_object* l_Coh_Kernel_T1__StrictCoh__to__Category___elambda__2___rarg(lean_object* x_1, lean_object* x_2) {
_start:
{
lean_object* x_3; lean_object* x_4; 
x_3 = lean_ctor_get(x_1, 1);
lean_inc(x_3);
lean_dec(x_1);
x_4 = lean_apply_1(x_3, x_2);
return x_4;
}
}
LEAN_EXPORT lean_object* l_Coh_Kernel_T1__StrictCoh__to__Category___elambda__2(lean_object* x_1) {
_start:
{
lean_object* x_2; 
x_2 = lean_alloc_closure((void*)(l_Coh_Kernel_T1__StrictCoh__to__Category___elambda__2___rarg), 2, 0);
return x_2;
}
}
LEAN_EXPORT lean_object* l_Coh_Kernel_T1__StrictCoh__to__Category___rarg(lean_object* x_1) {
_start:
{
lean_object* x_2; lean_object* x_3; lean_object* x_4; 
lean_inc(x_1);
x_2 = lean_alloc_closure((void*)(l_Coh_Kernel_T1__StrictCoh__to__Category___elambda__2___rarg), 2, 1);
lean_closure_set(x_2, 0, x_1);
x_3 = lean_alloc_closure((void*)(l_Coh_Kernel_T1__StrictCoh__to__Category___elambda__1___rarg), 6, 1);
lean_closure_set(x_3, 0, x_1);
x_4 = lean_alloc_ctor(0, 2, 0);
lean_ctor_set(x_4, 0, x_2);
lean_ctor_set(x_4, 1, x_3);
return x_4;
}
}
LEAN_EXPORT lean_object* l_Coh_Kernel_T1__StrictCoh__to__Category(lean_object* x_1) {
_start:
{
lean_object* x_2; 
x_2 = lean_alloc_closure((void*)(l_Coh_Kernel_T1__StrictCoh__to__Category___rarg), 1, 0);
return x_2;
}
}
lean_object* initialize_Init(uint8_t builtin, lean_object*);
lean_object* initialize_Coh_Kernel_Receipt(uint8_t builtin, lean_object*);
lean_object* initialize_Coh_Kernel_Verifier(uint8_t builtin, lean_object*);
lean_object* initialize_Mathlib_Tactic_Linarith(uint8_t builtin, lean_object*);
static bool _G_initialized = false;
LEAN_EXPORT lean_object* initialize_Coh_Kernel_T1__Category(uint8_t builtin, lean_object* w) {
lean_object * res;
if (_G_initialized) return lean_io_result_mk_ok(lean_box(0));
_G_initialized = true;
res = initialize_Init(builtin, lean_io_mk_world());
if (lean_io_result_is_error(res)) return res;
lean_dec_ref(res);
res = initialize_Coh_Kernel_Receipt(builtin, lean_io_mk_world());
if (lean_io_result_is_error(res)) return res;
lean_dec_ref(res);
res = initialize_Coh_Kernel_Verifier(builtin, lean_io_mk_world());
if (lean_io_result_is_error(res)) return res;
lean_dec_ref(res);
res = initialize_Mathlib_Tactic_Linarith(builtin, lean_io_mk_world());
if (lean_io_result_is_error(res)) return res;
lean_dec_ref(res);
l_Coh_Kernel_transition__id___closed__1 = _init_l_Coh_Kernel_transition__id___closed__1();
lean_mark_persistent(l_Coh_Kernel_transition__id___closed__1);
return lean_io_result_mk_ok(lean_box(0));
}
#ifdef __cplusplus
}
#endif
