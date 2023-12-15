#ifndef _LINKEDLIST_H_
#define _LINKEDLIST_H_

#include <stdint.h>

#define LISTSIZE 10

struct tx
{
    uint32_t to_insert;
} __attribute__ ((packed));

struct state
{
    uint32_t next_addr;
	uint16_t txlen;
	char transform_storage[128];
} __attribute__ ((packed));

struct node {
    uint32_t next;
    uint32_t val;
};

#endif
