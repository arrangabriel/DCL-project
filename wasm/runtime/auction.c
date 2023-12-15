#include <abi.h>
#include <auction.h>
#include <stddef.h>

struct item {
    uint32_t current_bid;
    uint32_t current_bidder;
};

static uint32_t balances[NUSERS] = { 1000, 1000 }; // Start two users out with some money
static struct item items[NITEMS];

static struct state __state;
static struct tx __tx;

void *bid(struct tx *tx, struct utx *utx, struct state *state)
{
	uint32_t *from = &balances[state->user];
    struct item *item = &items[tx->item];

	if (tx->amount > *from)
	    return NULL;
	if (item->current_bid > tx->amount)
	    return NULL;

	*from -= tx->amount;
	balances[item->current_bidder] += item->current_bid;
	item->current_bid = tx->amount;
	item->current_bidder = state->user;

    return NULL;
}

void *enter(struct tx *tx, struct utx *utx, struct state *state)
{
	if (state->user >= NUSERS)
		return NULL;
	if (state->txlen != sizeof (struct tx))
		return NULL;

	if (tx->item >= NITEMS)
	    return NULL;
	if (tx->amount == 0)
	    return NULL;

	return bid(tx, utx, state);
}

void *__enter(uint32_t user, uint32_t item, uint32_t amount)
{
    __state.user = user;
    __state.txlen = sizeof (struct tx);
    __tx.item = item;
    __tx.amount = amount;
    __utx = (struct utx) {0};

    return enter;
}

void *__step(void *callsite)
{
    utx_func_t utx_func = callsite;
    return utx_func(&__tx, &__utx, &__state);
}

uint32_t __get_balance(uint32_t user_id)
{
	return balances[user_id];
}

uint32_t __get_bidder(uint32_t item_id)
{
    return items[item_id].current_bidder;
}

uint32_t __get_bid(uint32_t item_id)
{
    return items[item_id].current_bid;
}

void __reset_user(uint32_t user_id) 
{
    balances[user_id] = 1000;
}

void __reset_item(uint32_t item_id)
{
    items[item_id] = (struct item) {0};
}
