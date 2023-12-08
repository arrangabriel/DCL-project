#ifndef _AUCTION_H_
#define _AUCTION_H_

#include <stdint.h>

#define NUSERS 100
#define NITEMS 3

struct tx
{
    uint32_t item;
    uint32_t amount;
} __attribute__ ((packed));

struct state
{
    uint32_t user;
	uint16_t txlen;
	char transform_storage[128];
} __attribute__ ((packed));

#endif
