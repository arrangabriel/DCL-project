#include <stdio.h>
#include <runtime.h>
#include <payment.h>

import("__enter") void *__enter(uint32_t user_id, uint32_t to, uint32_t amount);
import("__get_balance") uint32_t __get_balance(uint32_t user_id);
import("__reset_user") void __reset_user(uint32_t user_id);

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
    if (PRINT) {
        printf(
            "Running transaction: { from: %u, to %u, amount: %u }\n",
            state->user, tx->to, tx->amount
        );
    }
	void *callsite = enter(tx, state);
	while (callsite != NULL) {
        if (PRINT) { print_utx(); }
		callsite = step(callsite);
	}

    if (PRINT) {
        print_balance(1);
        print_balance(2);
        printf("\n");
    }
}

int main(void)
{
	struct state state;
	struct tx tx;
    state.txlen = sizeof (struct tx);

    for (int i = 0; i < ITERATIONS; i++) {
        __reset_user(1);
        __reset_user(2);
        if (PRINT) { printf("i: %i\n", i); }

        state.user = MINTER;
        tx.to = 1;
        tx.amount = 1000;
        run_transaction(&tx, &state);

        state.user = 1;
        tx.to = 2;
        tx.amount = 300;
        run_transaction(&tx, &state);

        state.user = 2;
        tx.to = 1;
        tx.amount = 200;
        run_transaction(&tx, &state);

        state.user = 1;
        tx.to = 2;
        tx.amount = 500;
        run_transaction(&tx, &state);

        state.user = 1;
        tx.to = 2;
        tx.amount = 600;
        run_transaction(&tx, &state);
    }
	return 0;
}
