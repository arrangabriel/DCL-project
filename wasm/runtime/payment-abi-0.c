#include <abi-0.h>
#include <payment.h>
#include <stddef.h>
#include <stdint.h>

static uint64_t balances[NUSER];

static void *mint0(struct tx *tx, struct utx *utx, struct state *state)
{
	uint32_t *to = (uint32_t*) utx->addrs[0];

	if ((*to + tx->amount) < *to)
		return NULL;

	*to += tx->amount;

	return NULL;
}

static void *mint(struct tx *tx, struct utx *utx, struct state *state)
{
	utx->naddr = 1;
	utx->addrs[0] = (uint32_t) &balances[tx->to];
	utx->log2lens[0] = sizeof (balances[tx->to]) >> 6;

	return mint0;
}


static void *pay0(struct tx *tx, struct utx *utx, struct state *state)
{
	uint32_t *from = (uint32_t *) utx->addrs[0];
	uint32_t *to = (uint32_t *) utx->addrs[1];

	if (*from < tx->amount)
		return NULL;
	if ((*to + tx->amount) < *to)
		return NULL;

	*from -= tx->amount;
	*to += tx->amount;

	return NULL;
}

static void *pay(struct tx *tx, struct utx *utx, struct state *state)
{
	utx->naddr = 2;

	utx->addrs[0] = (uint32_t) &balances[state->user];
	utx->log2lens[0] = sizeof (balances[state->user]) >> 6;

	utx->addrs[1] = (uint32_t) &balances[tx->to];
	utx->log2lens[1] = sizeof (balances[tx->to]) >> 6;

	return pay0;
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


typedef void *(* utx_t)(void *, struct utx *, struct state *);

static struct state __state;
static struct utx __utx;
static struct tx __tx;

void *__enter(uint32_t user, uint32_t to, uint32_t amount)
{
	__state.user = user;
	__state.txlen = sizeof (struct tx);
	__tx.to = to;
	__tx.amount = amount;

	return enter;
}

void *__step(void *callsite)
{
	utx_t utx_func = callsite;

	return utx_func(&__tx, &__utx, &__state);
}

uint32_t __get_utx_addrs(uint8_t index)
{
	return __utx.addrs[index];
}

uint8_t __get_utx_log2lens(uint8_t index)
{
	return __utx.log2lens[index];
}

uint8_t __get_utx_naddr(void)
{
	return __utx.naddr;
}

uint64_t __get_balance(uint32_t user)
{
	return balances[user];
}
