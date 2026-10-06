#include <stdlib.h>

#ifdef LIBGARN_TARGET_STATIC
__attribute__((tls_model("local-exec")))
#elif defined(LIBGARN_TARGET_DYNAMIC)
__attribute__((tls_model("global-dynamic")))
#else
#error "No TLS model specified for this build target."
#endif
__thread int* buf = NULL;

__attribute__((always_inline))
inline int* tls_buf() {
	if (__builtin_expect(!buf, 0)) {
		buf = (int*) malloc(sizeof(int));
	}
	return buf;
}
