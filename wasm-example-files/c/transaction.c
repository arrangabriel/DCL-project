#include <stdint.h>

 struct micro_tx
 {
 	uint64_t  addrs[7];
 	uint8_t   log2lens[7];
 	uint8_t   naddr;
 };

 struct tx
 {
 	uint64_t to;
 	uint32_t amount;
 };

 void *f_a(struct tx *tx, struct micro_tx *utx, void *state);
 void *f_b(struct tx *tx, struct micro_tx *utx, void *state);

 __attribute__((noinline)) void *f_a(struct tx *tx, struct micro_tx *utx, void *state)
 {
   uint64_t addr = 3;
   utx->addrs[0] = addr;
   return &f_b;
 }

 __attribute__((noinline)) void *f_b(struct tx *tx, struct micro_tx *utx, void *state)
 {
   uint64_t addr = utx->addrs[0];
   *(uint64_t*) state = addr;
   return 0;
 }

 __attribute__((noinline)) void *f_c(struct tx *tx, struct micro_tx *utx, void *state)
 {
   void* (*f_b_ptr)(struct tx *tx, struct micro_tx *utx, void *state) = f_a(tx, utx, state);
   ((void*(*)(struct tx *tx, struct micro_tx *utx, void *state))f_b_ptr)(tx, utx, state);
   return 0;
 }