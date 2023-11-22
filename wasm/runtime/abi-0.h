#ifndef _ABI_0_H_
#define _ABI_0_H_


#include <stdint.h>


struct utx
{
	uint32_t addrs[7];
	uint8_t log2lens[7];
	uint8_t naddr;
};


#endif
