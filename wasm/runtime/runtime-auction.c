#include <runtime.h>
#include <auction.h>

import("__enter") void *__enter();

static void *enter(struct tx* tx, struct state *state)
{
    return __enter();
}

int main(void)
{
    void *callsite;
	struct state state;
	struct tx tx;

	state.txlen = sizeof (struct tx);


	callsite = enter(&tx, &state);
	while (callsite != NULL) {
		print_utx();
		callsite = step(callsite);
	}

	return 0;
}