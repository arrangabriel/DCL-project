#include <inttypes.h>
#include <payment.h>
#include <stdio.h>


#ifdef __WASM__
# define import(name)							\
	__attribute__ ((import_module("contract"), import_name(name)))	\
	extern
#else
# define import(name) extern
#endif

import("__enter") void *__enter(uint32_t user, uint32_t to, uint32_t amount);
import("__step") void *__step(void *callsite);
import("__get_balance") uint64_t __get_balance(uint32_t user);
import("__get_utx_addrs") uint32_t __get_utx_addrs(uint8_t index);
import("__get_utx_log2lens") uint8_t __get_utx_log2lens(uint8_t index);
import("__get_utx_naddr") uint8_t __get_utx_naddr(void);

static void *enter(struct tx *tx, struct state *state)
{
	return __enter(state->user, tx->to, tx->amount);
}

static void *step(void *callsite)
{
	return __step(callsite);
}

static void print_utx(void)
{
	uint8_t naddr = __get_utx_naddr();
	uint8_t i;

	printf("utx {\n");
	printf("  naddr    = %hhu\n", naddr);

	for (i = 0; i < naddr; i++)
	    printf(
	        "  addrs[%i] = %u , log2lens[%i] = %u\n",
	        i, __get_utx_addrs(i), i, __get_utx_log2lens(i)
	    );

	printf("}\n");
}

static void print_balance(uint32_t user)
{
    printf("balances[%u] = %llu\n", user, __get_balance(user));
}


int main(void)
{
	void *callsite;
	struct state state;
	struct tx tx;

    printf("Transaction 1:\n");
	state.user = MINTER;
	state.txlen = sizeof (struct tx);
	tx.to = 1;
	tx.amount = 1000;

	callsite = enter(&tx, &state);
	while (callsite != NULL) {
		print_utx();
		callsite = step(callsite);
	}

    print_balance(1);

//	printf("\nTransaction 2:\n");
//
//	state.user = 1;
//	state.txlen = sizeof (struct tx);
//	tx.to = 2;
//	tx.amount = 300;
//
//	callsite = enter(&tx, &state);
//	while (callsite != NULL) {
//		print_utx();
//		callsite = step(callsite);
//	}
//
//    print_balance(1);
//    print_balance(2);

	return 0;
}
