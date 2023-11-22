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


import("__enter") void __enter(uint32_t user, uint32_t to, uint32_t amount);

import("__get_balance") uint64_t __get_balance(uint32_t user);

static void enter(struct tx *tx, struct state *state)
{
	__enter(state->user, tx->to, tx->amount);
}


int main(void)
{
	struct state state;
	struct tx tx;

	state.user = MINTER;
	state.txlen = sizeof (struct tx);
	tx.to = 1;
	tx.amount = 1000;
	enter(&tx, &state);

	printf("balances[1] = %" PRIu64 "\n", __get_balance(1));

	printf("--\n");

	state.user = 1;
	state.txlen = sizeof (struct tx);
	tx.to = 2;
	tx.amount = 300;
	enter(&tx, &state);

	printf("balances[1] = %" PRIu64 "\n", __get_balance(1));
	printf("balances[2] = %" PRIu64 "\n", __get_balance(2));

	return 0;
}
