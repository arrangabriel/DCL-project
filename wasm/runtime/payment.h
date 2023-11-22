#ifndef _PAYMENT_H_
#define _PAYMENT_H_


#include <stdint.h>


#define NUSER  10000
#define MINTER 0


struct tx
{
	uint32_t to;
	uint32_t amount;
} __attribute__ ((packed));


struct state
{
	uint32_t user;
	uint16_t txlen;
} __attribute__ ((packed));


#endif
