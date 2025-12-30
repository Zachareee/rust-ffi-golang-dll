package main

// #include <stdlib.h>
import "C"
import (
	"time"
	"unsafe"
)

//export GetString
func GetString() *C.char {
	return C.CString(time.Now().Local().String())
}

//export FreeString
func FreeString(str *C.char) {
	C.free(unsafe.Pointer(str))
}

func main() {}
