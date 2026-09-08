/**
 * @file sc_cas.h
 * @brief Zero-overhead C/C++ Header Wrapper for SC-Substrate Sovereign Wasm Substrate
 */

#ifndef SC_CAS_H
#define SC_CAS_H

#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include <string.h>

#ifdef __cplusplus
extern "C" {
#endif

#define SC_CAS_PREAMBLE_CANARY 0x5343434153574153ULL  /* SCCASWAS */
#define SC_CAS_SEAL_CANARY     0x5343535F5345414CULL  /* SCS_SEAL */

typedef enum {
    SC_CAS_OP_EVAL_SYMBOLIC         = 1,
    SC_CAS_OP_INTEGRATE_MONODROMY   = 2,
    SC_CAS_OP_DIXON_SOLVE           = 3,
    SC_CAS_OP_DEEPPOLY_VERIFY       = 4,
    SC_CAS_OP_ARITHMETIC_SIMPLIFY   = 5,
    SC_CAS_OP_CLIFFORD_MULTIVECTOR  = 6,
    SC_CAS_OP_TENSOR_SLICE          = 7
} sc_cas_opcode_t;

typedef struct {
    uint32_t status_code;
    const char* payload;
    uint32_t payload_len;
    uint8_t attestation_digest[32];
} sc_cas_result_t;

/* Standard Flat C-ABI Function Signatures */
uint32_t sc_cas_wasm_alloc(uint32_t size);
void     sc_cas_wasm_free(uint32_t offset, uint32_t size);
uint32_t sc_cas_wasm_dispatch(uint32_t cmd_offset, uint32_t cmd_len, uint32_t out_offset, uint32_t out_cap);
uint32_t sc_cas_wasm_version(void);

#ifdef __cplusplus
}
#endif

#endif /* SC_CAS_H */
