#ifndef _ABI_0_H_
#define _ABI_0_H_

#include <stdint.h>

struct utx
{
	uint32_t addrs[7];
	uint8_t log2lens[7];
	uint8_t naddr;
};

typedef void *(* utx_func_t)(void *, struct utx *, void *);

static struct utx __utx;

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

#endif
