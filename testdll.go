package main

// #include <stdlib.h>
import "C"
import "unsafe"

//export GetString
func GetString() *C.char {
	return C.CString("Hello World")
}

//export FreeString
func FreeString(str *C.char) {
	C.free(unsafe.Pointer(str))
}

func main() {}
