#include <stdio.h>
#include <stdint.h>

#ifdef PROFILE
    #define PRINT 0
    #define ITERATIONS 100000000
#else
    #define PRINT 1
    #define ITERATIONS 1
#endif

#ifdef __WASM__
# define import(name)							\
	__attribute__ ((import_module("contract"), import_name(name)))	\
	extern
#else
# define import(name) extern
#endif

import("__step") void *__step(void *callsite);
import("__get_utx_addrs") uint32_t __get_utx_addrs(uint8_t index);
import("__get_utx_log2lens") uint8_t __get_utx_log2lens(uint8_t index);
import("__get_utx_naddr") uint8_t __get_utx_naddr(void);

static void *step(void *callsite)
{
	return __step(callsite);
}

static void print_utx(void)
{
	uint8_t naddr = __get_utx_naddr();
	uint8_t i;

	printf("utx {\n");
	printf("  naddr    = %hhu\n", naddr);
	for (i = 0; i < naddr; i++)
	{
	    printf(
	        "  addrs[%i] = %u , log2lens[%i] = %u\n",
	        i, __get_utx_addrs(i), i, __get_utx_log2lens(i)
	    );
	}
	printf("}\n");
}
