#ifndef _AUCTION_H_
#define _AUCTION_H_

#define NUSER 100

struct tx
{
} __attribute__ ((packed));

struct state
{
	uint16_t txlen;
	char transform_storage[128];
} __attribute__ ((packed));

#endif
