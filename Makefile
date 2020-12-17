CFLAGS=-std=c11 -g -static

tvcc: tvcc.c

test: tvcc
	./test.sh

clean:
	rm -f tvcc *.o *~ tmp*

.PHONY: test clean
