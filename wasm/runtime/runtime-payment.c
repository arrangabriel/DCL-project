#include <stdio.h>
#include <runtime.h>
#include <payment.h>

import("__enter") void *__enter(uint32_t user_id, uint32_t to, uint32_t amount);
import("__get_balance") uint32_t __get_balance(uint32_t user_id);

static void print_balance(uint32_t user_id)
{
    printf("balances[%u] = %u\n", user_id, __get_balance(user_id));
}

static void *enter(struct tx *tx, struct state *state)
{
	return __enter(state->user, tx->to, tx->amount);
}

static void run_transaction(struct tx *tx, struct state *state)
{
	void *callsite = enter(tx, state);
	while (callsite != NULL) {
		print_utx();
		callsite = step(callsite);
	}

    print_balance(1);
    print_balance(2);
}

int main(void)
{
	struct state state;
	struct tx tx;

	state.user = MINTER;
	state.txlen = sizeof (struct tx);
	tx.to = 1;
	tx.amount = 1000;
    run_transaction(&tx, &state);

	state.user = 1;
	state.txlen = sizeof (struct tx);
	tx.to = 2;
	tx.amount = 300;
    run_transaction(&tx, &state);

	return 0;
}
