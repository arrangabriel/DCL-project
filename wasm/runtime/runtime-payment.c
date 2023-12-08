#include <stdio.h>
#include <runtime.h>
#include <payment.h>

import("__enter") void *__enter(uint32_t user, uint32_t to, uint32_t amount);
import("__get_balance") uint64_t __get_balance(uint32_t user);
import("__get_user") uint32_t __get_user(void);

static void *enter(struct tx *tx, struct state *state)
{
	return __enter(state->user, tx->to, tx->amount);
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

	state.user = 1;
	state.txlen = sizeof (struct tx);
	tx.to = 2;
	tx.amount = 300;

	callsite = enter(&tx, &state);
	while (callsite != NULL) {
		print_utx();
		callsite = step(callsite);
	}

    print_balance(1);
    print_balance(2);

	return 0;
}
