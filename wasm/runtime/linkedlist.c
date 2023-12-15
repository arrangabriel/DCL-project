#include <abi.h>
#include <linkedlist.h>
#include <stddef.h>
#include <stdint.h>

static struct node list[LISTSIZE];
static uint32_t curr_len = 0;
const uint32_t base = 0;

static struct state __state;
static struct tx __tx;

void *walk(struct tx *tx, struct utx *utx, struct state *state)
{
    uint32_t addr = state->next_addr;
    struct node *node = &list[addr];
    // If node.val == 0 the node is free
    if (node->val == 0) {
        node->val = tx->to_insert;
        node->next = addr + 1;
        curr_len++;
        return NULL;
    } else {
        state->next_addr = node->next;
        return walk;
    }
}

void *enter(void *_tx, struct utx *utx, struct state *state)
{
    if (curr_len == LISTSIZE) return NULL;
	if (state->txlen != sizeof (struct tx)) return NULL;
    state->next_addr = base;
    return walk;
}

void *__enter(uint32_t val)
{
    __state.next_addr = 0;
	__state.txlen = sizeof (struct tx);
    __tx.to_insert = val;
    __utx = (struct utx) { 0 };

	return enter;
}

void *__step(void *callsite)
{
	utx_func_t utx_func = callsite;
	return utx_func(&__tx, &__utx, &__state);
}

uint32_t __get_node_val(uint32_t node_addr) {
    return list[node_addr].val;
}

uint32_t __get_node_next(uint32_t node_addr) {
    return list[node_addr].next;
}

void __reset_node(uint32_t node_addr) {
    list[node_addr] = (struct node) { 0 };
}