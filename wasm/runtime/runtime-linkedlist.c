#include <runtime.h>
#include <linkedlist.h>

import("__enter") void *__enter(uint32_t val);

import("__get_node_val") uint32_t __get_node_val(uint32_t node_addr);
import("__get_node_next") uint32_t __get_node_next(uint32_t node_addr);
import("__reset_node") void __reset_node(uint32_t node_addr);

static void *enter(struct tx *tx, struct state *state)
{
    return __enter(tx->to_insert);
}

static void run_transaction(struct tx *tx, struct state *state)
{
    if (PRINT) {
        printf(
            "Running transaction: { to_insert: %u }\n",
            tx->to_insert
        );
    }
    void *callsite = enter(tx, state);
	while (callsite != NULL) {
        if (PRINT) { print_utx(); }
		callsite = step(callsite);
	}
    if (PRINT) {
        printf("list: [\n");
        for (int i = 0; i < LISTSIZE; i++) {
            uint32_t val = __get_node_val(i);
            uint32_t next = __get_node_next(i);
            if (val > 0) { 
                printf(
                    "  %u: { val: %u, next: %u }\n", 
                    i, val, next
                ); 
            }
        }
        printf("]\n");
    }
}

int main(void)
{
	struct state state;
	struct tx tx;
	state.txlen = sizeof (struct tx);

    for (int i = 0; i < ITERATIONS; i++) {
        __reset_node(0);
        __reset_node(1);
        __reset_node(2);
        __reset_node(3);
        __reset_node(4);
        if (PRINT) { printf("i: %i\n", i); }
        tx.to_insert = 10;
        run_transaction(&tx, &state);

        tx.to_insert = 20;
        run_transaction(&tx, &state);

        tx.to_insert = 30;
        run_transaction(&tx, &state);

        tx.to_insert = 40;
        run_transaction(&tx, &state);

        tx.to_insert = 50;
        run_transaction(&tx, &state);
    }

	return 0;
}


