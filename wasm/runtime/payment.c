#include <abi.h>
#include <payment.h>
#include <stddef.h>
#include <stdint.h>

static uint64_t balances[NUSER];

static struct state __state;
static struct tx __tx;

static void *mint(struct tx *tx, struct utx *utx, struct state *state)
{
	uint32_t *to = (uint32_t*) &balances[tx->to];

	if ((*to + tx->amount) < *to)
		return NULL;

	*to += tx->amount;

	return NULL;
}

static void *pay(struct tx *tx, struct utx *utx, struct state *state)
{
	uint32_t *from = (uint32_t *) &balances[state->user];
	uint32_t *to = (uint32_t *) &balances[tx->to];

	if (*from < tx->amount)
		return NULL;
	if ((*to + tx->amount) < *to)
		return NULL;

	*from -= tx->amount;
	*to += tx->amount;

	return NULL;
}

void *enter(void *_tx, struct utx *utx, struct state *state)
{
	if (state->user >= NUSER)
		return NULL;
	if (state->txlen != sizeof (struct tx))
		return NULL;

	struct tx *tx = _tx;

	if (tx->to == state->user)
		return NULL;
	if (tx->to >= NUSER)
		return NULL;
	if (tx->amount == 0)
		return NULL;

	if (state->user == MINTER)
	    return mint(tx, utx, state);
	else
		return pay(tx, utx, state);
}

void *__enter(uint32_t user, uint32_t to, uint32_t amount)
{
	__state.user = user;
	__state.txlen = sizeof (struct tx);
	__tx.to = to;
	__tx.amount = amount;
    struct utx init = {0};
    __utx = init;

	return enter;
}

void *__step(void *callsite)
{
	utx_func_t utx_func = callsite;

	return utx_func(&__tx, &__utx, &__state);
}

uint64_t __get_balance(uint32_t user)
{
	return balances[user];
}

uint32_t __get_user(void)
{
    return __state.user;
}

char __get_transform_storage(uint32_t index)
{
    return __state.transform_storage[index];
}
