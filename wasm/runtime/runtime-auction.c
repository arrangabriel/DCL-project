#include <runtime.h>
#include <auction.h>

import("__enter") void *__enter(uint32_t user, uint32_t item, uint32_t amount);

import("__get_balance") uint32_t __get_balance(uint32_t user_id);
import("__get_bidder") uint32_t __get_bidder(uint32_t item_id);
import("__get_bid") uint32_t __get_bid(uint32_t item_id);

static void print_balance(uint32_t user_id)
{
    printf("balances[%u] = %u\n", user_id, __get_balance(user_id));
}

static void print_items(void)
{
    for (uint32_t item_id = 0; item_id < NITEMS; item_id++)
    {
        uint32_t bidder = __get_bidder(item_id);
        uint32_t bid = __get_bid(item_id);
        if (bid > 0)
        {
            printf(
                "items[%u] = { bidder: %u, bid: %u }\n",
                item_id, bidder, bid
            );
        }
        else
        {
            printf("items[%u] = { bidder: N/A, bid: N/A }\n", item_id);
        }
    }
}

static void *enter(struct tx *tx, struct state *state)
{
    return __enter(state->user, tx->item, tx->amount);
}

static void run_transaction(struct tx *tx, struct state *state)
{
    printf(
        "Running transaction: { bidder: %u, item %u, amount: %u }\n",
        state->user, tx->item, tx->amount
    );
    void *callsite = enter(tx, state);
	while (callsite != NULL) {
		print_utx();
		callsite = step(callsite);
	}
	print_balance(0);
	print_balance(1);
	print_items();
	printf("\n");
}

int main(void)
{
	struct state state;
	struct tx tx;
	state.txlen = sizeof (struct tx);

    printf("Initial state:\n");
	print_balance(0);
	print_balance(1);
	print_items();
	printf("\n");

    state.user = 0;
	tx.item = 0;
	tx.amount = 500;
    run_transaction(&tx, &state);

    state.user = 1;
	tx.item = 0;
	tx.amount = 600;
    run_transaction(&tx, &state);

    state.user = 0;
    tx.item = 2;
    tx.amount = 500;
    run_transaction(&tx, &state);

    state.user = 1;
    tx.item = 2;
    tx.amount = 600;
    run_transaction(&tx, &state);

    state.user = 1;
    tx.item = 1;
    tx.amount = 300;
    run_transaction(&tx, &state);

	return 0;
}