#include <abi.h>
#include <auction.h>
#include <stddef.h>

static struct state __state;
static struct tx __tx;

void *enter(void *tx, struct utx *utx, struct state *state)
{
    return NULL;
}

void *__enter()
{
    return enter;
}

void *__step(void *callsite)
{
    utx_func_t utx_func = callsite;
    return utx_func(&__tx, &__utx, &__state);
}