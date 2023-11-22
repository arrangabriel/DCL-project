#include <payment.h>
#include <stdint.h>

static uint64_t balances[NUSER];

static void mint(struct tx *tx, struct state *state)
{
	if ((balances[tx->to] + tx->amount) < balances[tx->to])
		return;

	balances[tx->to] += tx->amount;
}

static void pay(struct tx *tx, struct state *state)
{
	if (balances[state->user] < tx->amount)
		return;
	if ((balances[tx->to] + tx->amount) < balances[tx->to])
		return;

	balances[state->user] -= tx->amount;
	balances[tx->to] += tx->amount;
}

void enter(void *_tx, struct state *state)
{
	if (state->user >= NUSER)
		return;
	if (state->txlen != sizeof (struct tx))
		return;

	struct tx *tx = _tx;

	if (tx->to == state->user)
		return;
	if (tx->to >= NUSER)
		return;
	if (tx->amount == 0)
		return;

	if (state->user == MINTER)
		mint(tx, state);
	else
		pay(tx, state);
}


void __enter(uint32_t user, uint32_t to, uint32_t amount)
{
	struct state state;
	struct tx tx;

	state.user = user;
	state.txlen = sizeof (struct tx);
	tx.to = to;
	tx.amount = amount;

	enter(&tx, &state);
}

uint64_t __get_balance(uint32_t user)
{
	return balances[user];
}
