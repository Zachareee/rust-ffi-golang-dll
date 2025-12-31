package main

// #include <stdlib.h>
import "C"
import (
	"runtime"
	"time"
	"unsafe"
)

var pinner runtime.Pinner

//export GetString
func GetString() *C.char {
	return C.CString(time.Now().Local().String())
}

//export FreeString
func FreeString(str *C.char) {
	C.free(unsafe.Pointer(str))
}

//export GetStruct
func GetStruct() (count int32, strings **C.char) {
	s := []*C.char{
		C.CString("Germany"),
		C.CString("Korea"),
		C.CString("Japan"),
	}
	count = int32(len(s))
	strings = unsafe.SliceData(s)
	pinner.Pin(strings)
	return
}

func FreeStruct() {
	pinner.Unpin()
}

func main() {}
